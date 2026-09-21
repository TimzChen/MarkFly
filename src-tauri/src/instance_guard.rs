//! 单实例握手与卡死接管。
//!
//! Windows 上官方 `tauri-plugin-single-instance` 用同步 `SendMessageW` 通知已有实例。
//! 已有实例一旦 UI 线程卡死，新进程会永久堵在这条系统调用上。
//! 这里改成带超时的握手：窗口已挂死才强杀并接管，忙但未挂死则不抢。

use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager, RunEvent, Runtime};

const FORCE_EXIT_GRACE: Duration = Duration::from_secs(4);
const HANDOFF_TIMEOUT_MS: u32 = 4000;
const FIND_WINDOW_RETRIES: u32 = 10;
const FIND_WINDOW_GAP: Duration = Duration::from_millis(50);
const MUTEX_WAIT_AFTER_KILL_MS: u32 = 3000;
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(5);
const WATCHDOG_STARTUP_GRACE: Duration = Duration::from_secs(10);
const WATCHDOG_MISS_LIMIT: u8 = 3;

static CACHED_MAIN_HWND: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

pub fn schedule_force_exit() {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        std::thread::sleep(FORCE_EXIT_GRACE);
        eprintln!("MarkFly: graceful exit timed out, forcing process tree kill");
        if !force_kill_pid_tree(std::process::id()) {
            std::process::exit(1);
        }
    });
}

#[cfg(windows)]
pub fn cache_main_window_hwnd<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(hwnd) = window.hwnd() {
            CACHED_MAIN_HWND.store(hwnd.0, Ordering::SeqCst);
        }
    }
}

#[cfg(not(windows))]
pub fn cache_main_window_hwnd<R: Runtime>(_app: &AppHandle<R>) {}

pub fn spawn_self_watchdog() {
    #[cfg(windows)]
    {
        std::thread::spawn(|| {
            std::thread::sleep(WATCHDOG_STARTUP_GRACE);
            let mut misses = 0_u8;
            loop {
                std::thread::sleep(WATCHDOG_INTERVAL);
                let hwnd = CACHED_MAIN_HWND.load(Ordering::SeqCst);
                if hwnd.is_null() {
                    continue;
                }
                let (next_misses, kill) =
                    watchdog_tick(misses, !is_hwnd_unresponsive(hwnd), WATCHDOG_MISS_LIMIT);
                misses = next_misses;
                if kill {
                    eprintln!(
                        "MarkFly: main window unresponsive for {:?}, forcing exit",
                        WATCHDOG_INTERVAL * u32::from(WATCHDOG_MISS_LIMIT)
                    );
                    if !force_kill_pid_tree(std::process::id()) {
                        std::process::exit(1);
                    }
                    return;
                }
            }
        });
    }
}

pub(crate) fn watchdog_tick(misses: u8, responsive: bool, limit: u8) -> (u8, bool) {
    if responsive {
        (0, false)
    } else {
        let next = misses.saturating_add(1);
        (next, next >= limit)
    }
}

fn force_kill_pid_tree(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        if pid == std::process::id() {
            std::process::exit(1);
        }
        false
    }
}

#[cfg(windows)]
fn is_hwnd_unresponsive(hwnd: *mut core::ffi::c_void) -> bool {
    windows_impl::is_hwnd_unresponsive(hwnd)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandoffDecision {
    ExitAsSecondary,
    BecomePrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstanceProbe {
    Delivered,
    Hung,
    BusyTimeout,
    Missing,
}

pub(crate) fn decide_after_probe(probe: InstanceProbe) -> HandoffDecision {
    match probe {
        InstanceProbe::Delivered | InstanceProbe::BusyTimeout => HandoffDecision::ExitAsSecondary,
        InstanceProbe::Hung | InstanceProbe::Missing => HandoffDecision::BecomePrimary,
    }
}

pub(crate) fn should_retry_find_window(attempt: u32, max_attempts: u32, found: bool) -> bool {
    !found && attempt + 1 < max_attempts
}

pub(crate) fn single_instance_names(identifier: &str) -> (String, String, String) {
    (
        format!("{identifier}-sic"),
        format!("{identifier}-siw"),
        format!("{identifier}-sim"),
    )
}

const WMCOPYDATA_SINGLE_INSTANCE_DATA: usize = 1542;

pub(crate) fn encode_copydata_payload(cwd: &str, args: &[String]) -> String {
    format!("{cwd}|{}\0", args.join("|"))
}

pub(crate) fn parse_copydata_payload(data: &str) -> (String, Vec<String>) {
    let data = data.trim_end_matches('\0');
    let mut parts = data.split('|');
    let cwd = parts.next().unwrap_or_default().to_string();
    let args = parts
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    (cwd, args)
}

#[cfg(windows)]
pub fn init<R: Runtime, F>(callback: F) -> tauri::plugin::TauriPlugin<R>
where
    F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static,
{
    windows_impl::init(Box::new(callback))
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::ffi::CStr;
    use tauri::plugin::{self, TauriPlugin};
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WAIT_ABANDONED,
        WAIT_OBJECT_0, WPARAM,
    };
    use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW, GetWindowThreadProcessId,
        IsHungAppWindow, RegisterClassExW, SendMessageTimeoutW, CREATESTRUCTW, GWLP_USERDATA,
        GWL_STYLE, SMTO_ABORTIFHUNG, SMTO_BLOCK, WINDOW_LONG_PTR_INDEX, WM_COPYDATA, WM_CREATE,
        WM_DESTROY, WM_NULL, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT, WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
    };

    struct MutexHandle(isize);
    struct TargetWindowHandle(isize);

    struct UserData<R: Runtime> {
        app: AppHandle<R>,
        callback: Box<dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>,
    }

    impl<R: Runtime> UserData<R> {
        unsafe fn from_hwnd_raw(hwnd: HWND) -> *mut Self {
            GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Self
        }

        unsafe fn from_hwnd<'a>(hwnd: HWND) -> &'a mut Self {
            &mut *Self::from_hwnd_raw(hwnd)
        }
    }

    pub fn init<R: Runtime>(
        callback: Box<dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>,
    ) -> TauriPlugin<R> {
        plugin::Builder::new("markfly-instance-guard")
            .setup(|app, _api| {
                attach_or_handoff(app, callback);
                Ok(())
            })
            .on_event(|app, event| {
                if let RunEvent::ExitRequested { .. } = event {
                    schedule_force_exit();
                }
                if let RunEvent::Exit = event {
                    destroy(app);
                }
            })
            .build()
    }

    fn attach_or_handoff<R: Runtime>(
        app: &AppHandle<R>,
        callback: Box<dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>,
    ) {
        let id = app.config().identifier.clone();
        let (class_name, window_name, mutex_name) = single_instance_names(&id);
        let class_name = encode_wide(&class_name);
        let window_name = encode_wide(&window_name);
        let mutex_name = encode_wide(&mutex_name);

        let hmutex = unsafe { CreateMutexW(std::ptr::null(), 1, mutex_name.as_ptr()) };
        if hmutex.is_null() {
            eprintln!("MarkFly: failed to create single-instance mutex");
            become_primary(app, callback, 0, &class_name, &window_name);
            return;
        }

        let already_exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        if already_exists {
            match handoff_existing(app, hmutex, &class_name, &window_name) {
                HandoffDecision::ExitAsSecondary => return,
                HandoffDecision::BecomePrimary => {}
            }
        }

        become_primary(app, callback, hmutex as isize, &class_name, &window_name);
    }

    fn handoff_existing<R: Runtime>(
        app: &AppHandle<R>,
        hmutex: *mut core::ffi::c_void,
        class_name: &[u16],
        window_name: &[u16],
    ) -> HandoffDecision {
        let hwnd = find_instance_window(class_name, window_name);
        let probe = probe_instance(hwnd);
        match decide_after_probe(probe) {
            HandoffDecision::ExitAsSecondary => {
                unsafe {
                    CloseHandle(hmutex);
                }
                app.cleanup_before_exit();
                std::process::exit(0);
            }
            HandoffDecision::BecomePrimary => {
                if !hwnd.is_null() && probe == InstanceProbe::Hung {
                    let mut pid = 0_u32;
                    unsafe {
                        GetWindowThreadProcessId(hwnd, &mut pid);
                    }
                    if pid != 0 && pid != std::process::id() {
                        eprintln!("MarkFly: existing instance {pid} is hung, taking over");
                        if !force_kill_pid_tree(pid)
                            || !wait_mutex(hmutex, MUTEX_WAIT_AFTER_KILL_MS)
                        {
                            eprintln!("MarkFly: failed to take over hung instance {pid}");
                            unsafe {
                                CloseHandle(hmutex);
                            }
                            app.cleanup_before_exit();
                            std::process::exit(0);
                        }
                        return HandoffDecision::BecomePrimary;
                    }
                }
                if !wait_mutex(hmutex, MUTEX_WAIT_AFTER_KILL_MS) {
                    let retry_hwnd = find_instance_window(class_name, window_name);
                    if decide_after_probe(probe_instance(retry_hwnd))
                        == HandoffDecision::ExitAsSecondary
                    {
                        unsafe {
                            CloseHandle(hmutex);
                        }
                        app.cleanup_before_exit();
                        std::process::exit(0);
                    }
                    eprintln!("MarkFly: another instance owns the session lock, not taking over");
                    unsafe {
                        CloseHandle(hmutex);
                    }
                    app.cleanup_before_exit();
                    std::process::exit(0);
                }
                HandoffDecision::BecomePrimary
            }
        }
    }

    fn become_primary<R: Runtime>(
        app: &AppHandle<R>,
        callback: Box<dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>,
        hmutex: isize,
        class_name: &[u16],
        window_name: &[u16],
    ) {
        if hmutex != 0 {
            app.manage(MutexHandle(hmutex));
        }

        let userdata = Box::into_raw(Box::new(UserData {
            app: app.clone(),
            callback,
        }));
        let hwnd = create_event_target_window::<R>(class_name, window_name, userdata);
        if hwnd.is_null() {
            eprintln!("MarkFly: failed to create single-instance window");
            unsafe {
                drop(Box::from_raw(userdata));
            }
            return;
        }
        app.manage(TargetWindowHandle(hwnd as isize));
    }

    fn find_instance_window(class_name: &[u16], window_name: &[u16]) -> HWND {
        for attempt in 0..FIND_WINDOW_RETRIES {
            let hwnd = unsafe { FindWindowW(class_name.as_ptr(), window_name.as_ptr()) };
            if !should_retry_find_window(attempt, FIND_WINDOW_RETRIES, !hwnd.is_null()) {
                return hwnd;
            }
            std::thread::sleep(FIND_WINDOW_GAP);
        }
        std::ptr::null_mut()
    }

    fn probe_instance(hwnd: HWND) -> InstanceProbe {
        if hwnd.is_null() {
            return InstanceProbe::Missing;
        }
        if unsafe { IsHungAppWindow(hwnd) } != 0 {
            return InstanceProbe::Hung;
        }
        if send_copydata(hwnd) {
            InstanceProbe::Delivered
        } else {
            InstanceProbe::BusyTimeout
        }
    }

    fn wait_mutex(hmutex: *mut core::ffi::c_void, timeout_ms: u32) -> bool {
        let result = unsafe { WaitForSingleObject(hmutex, timeout_ms) };
        result == WAIT_OBJECT_0 || result == WAIT_ABANDONED
    }

    fn destroy<R: Runtime>(app: &AppHandle<R>) {
        if let Some(hmutex) = app.try_state::<MutexHandle>() {
            unsafe {
                ReleaseMutex(hmutex.0 as _);
                CloseHandle(hmutex.0 as _);
            }
        }
        if let Some(hwnd) = app.try_state::<TargetWindowHandle>() {
            unsafe {
                DestroyWindow(hwnd.0 as HWND);
            }
        }
    }

    fn send_copydata(hwnd: HWND) -> bool {
        let cwd = std::env::current_dir().unwrap_or_default();
        let cwd = cwd.to_string_lossy();
        let args: Vec<String> = std::env::args().collect();
        let data = encode_copydata_payload(&cwd, &args);
        let bytes = data.into_bytes();
        let cds = COPYDATASTRUCT {
            dwData: WMCOPYDATA_SINGLE_INSTANCE_DATA,
            cbData: bytes.len() as u32,
            lpData: bytes.as_ptr() as *mut _,
        };

        let mut result = 0_usize;
        let sent = unsafe {
            SendMessageTimeoutW(
                hwnd,
                WM_COPYDATA,
                0,
                &cds as *const COPYDATASTRUCT as LPARAM,
                SMTO_ABORTIFHUNG | SMTO_BLOCK,
                HANDOFF_TIMEOUT_MS,
                &mut result,
            )
        };
        sent != 0
    }

    pub(super) fn is_hwnd_unresponsive(hwnd: HWND) -> bool {
        if hwnd.is_null() {
            return false;
        }
        if unsafe { IsHungAppWindow(hwnd) } != 0 {
            return true;
        }
        let mut result = 0_usize;
        let sent = unsafe {
            SendMessageTimeoutW(
                hwnd,
                WM_NULL,
                0,
                0,
                SMTO_ABORTIFHUNG | SMTO_BLOCK,
                2000,
                &mut result,
            )
        };
        sent == 0
    }

    fn encode_wide(value: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        std::ffi::OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    unsafe extern "system" fn single_instance_window_proc<R: Runtime>(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam as *const CREATESTRUCTW);
                let userdata = create_struct.lpCreateParams as *const UserData<R>;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, userdata as isize);
                0
            }
            WM_COPYDATA => {
                let cds_ptr = lparam as *const COPYDATASTRUCT;
                if (*cds_ptr).dwData == WMCOPYDATA_SINGLE_INSTANCE_DATA {
                    let userdata = UserData::<R>::from_hwnd(hwnd);
                    let raw = CStr::from_ptr((*cds_ptr).lpData as *const i8).to_string_lossy();
                    let (cwd, args) = parse_copydata_payload(&raw);
                    (userdata.callback)(&userdata.app, args, cwd);
                }
                1
            }
            WM_DESTROY => {
                let userdata = UserData::<R>::from_hwnd_raw(hwnd);
                drop(Box::from_raw(userdata));
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn create_event_target_window<R: Runtime>(
        class_name: &[u16],
        window_name: &[u16],
        userdata: *const UserData<R>,
    ) -> HWND {
        unsafe {
            let class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(single_instance_window_proc::<R>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(std::ptr::null()),
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            RegisterClassExW(&class);
            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()),
                userdata as _,
            );
            if hwnd.is_null() {
                return hwnd;
            }
            SetWindowLongPtrW(hwnd, GWL_STYLE, (WS_VISIBLE | WS_POPUP) as isize);
            hwnd
        }
    }

    #[cfg(target_pointer_width = "32")]
    #[allow(non_snake_case)]
    unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongW(hwnd, index, value as i32)
            as isize
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(non_snake_case)]
    unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(hwnd, index, value)
    }

    #[cfg(target_pointer_width = "32")]
    #[allow(non_snake_case)]
    unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongW(hwnd, index) as isize
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(non_snake_case)]
    unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivered_or_busy_instance_stays_secondary() {
        assert_eq!(
            decide_after_probe(InstanceProbe::Delivered),
            HandoffDecision::ExitAsSecondary
        );
        assert_eq!(
            decide_after_probe(InstanceProbe::BusyTimeout),
            HandoffDecision::ExitAsSecondary
        );
    }

    #[test]
    fn hung_or_missing_instance_is_taken_over() {
        assert_eq!(
            decide_after_probe(InstanceProbe::Hung),
            HandoffDecision::BecomePrimary
        );
        assert_eq!(
            decide_after_probe(InstanceProbe::Missing),
            HandoffDecision::BecomePrimary
        );
    }

    #[test]
    fn find_window_retries_until_last_attempt() {
        assert!(should_retry_find_window(0, 10, false));
        assert!(!should_retry_find_window(9, 10, false));
        assert!(!should_retry_find_window(0, 10, true));
    }

    #[test]
    fn watchdog_kills_after_consecutive_misses() {
        assert_eq!(watchdog_tick(0, true, 3), (0, false));
        assert_eq!(watchdog_tick(2, false, 3), (3, true));
        assert_eq!(watchdog_tick(1, true, 3), (0, false));
        assert_eq!(watchdog_tick(1, false, 3), (2, false));
    }

    #[test]
    fn single_instance_names_match_tauri_plugin_suffixes() {
        let (class, window, mutex) = single_instance_names("com.jwangkun.markfly");
        assert_eq!(class, "com.jwangkun.markfly-sic");
        assert_eq!(window, "com.jwangkun.markfly-siw");
        assert_eq!(mutex, "com.jwangkun.markfly-sim");
    }

    #[test]
    fn copydata_payload_roundtrip() {
        let payload = encode_copydata_payload(
            "C:\\docs",
            &["markfly.exe".into(), "note.md".into()],
        );
        let (cwd, args) = parse_copydata_payload(&payload);
        assert_eq!(cwd, "C:\\docs");
        assert_eq!(args, vec!["markfly.exe".to_string(), "note.md".to_string()]);
    }

    #[test]
    fn copydata_payload_ignores_empty_segments() {
        let (cwd, args) = parse_copydata_payload("C:\\docs|markfly.exe||note.md|\0");
        assert_eq!(cwd, "C:\\docs");
        assert_eq!(args, vec!["markfly.exe".to_string(), "note.md".to_string()]);
    }
}
