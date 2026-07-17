// MarkFly - 跨平台 Markdown 编辑器
// 基于 Tauri + Vue + ByteMD 构建

mod file_watcher;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashSet;

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::webview::PageLoadEvent;
use tauri::{Emitter, Manager, State, Webview};
use tauri_plugin_fs::FsExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PendingOpenFile {
    path: String,
    content: String,
}

struct PendingOpenFiles(Mutex<Vec<PendingOpenFile>>);

const BOOT_PREVIEW_MAX_BYTES: usize = 512 * 1024;

#[derive(serde::Serialize)]
struct BootPreviewFile {
    path: String,
    content: String,
    truncated: bool,
}

fn collect_paths_from_args<I>(args: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = String>,
{
    args.into_iter()
        .filter(|arg| !arg.starts_with('-') && arg != "--")
        .map(PathBuf::from)
        .collect()
}

fn resolve_open_files(app: &tauri::AppHandle, paths: Vec<PathBuf>) -> Vec<String> {
    let fs_scope = app.fs_scope();
    let mut valid_paths = Vec::new();

    for path in paths {
        if path.is_file() {
            let _ = fs_scope.allow_file(&path);
            if let Some(path_str) = path.to_str() {
                valid_paths.push(path_str.to_string());
            }
        }
    }

    valid_paths
}

fn store_pending_open_files(app: &tauri::AppHandle, paths: Vec<PathBuf>) -> Vec<String> {
    let fs_scope = app.fs_scope();
    let mut valid_paths = Vec::new();
    let mut pending_items = Vec::new();
    let existing_paths: HashSet<String> = app
        .state::<PendingOpenFiles>()
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|item| item.path.clone())
        .collect();

    for path in paths {
        if !path.is_file() {
            continue;
        }

        let _ = fs_scope.allow_file(&path);
        let Some(path_str) = path.to_str() else {
            continue;
        };

        if existing_paths.contains(path_str) {
            valid_paths.push(path_str.to_string());
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        pending_items.push(PendingOpenFile {
            path: path_str.to_string(),
            content,
        });
        valid_paths.push(path_str.to_string());
    }

    if !pending_items.is_empty() {
        app.state::<PendingOpenFiles>()
            .0
            .lock()
            .unwrap()
            .extend(pending_items);
    }

    valid_paths
}

fn dispatch_open_files(app: &tauri::AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }

    let _ = app.emit("open-file-path", paths);
}

fn read_initial_boot_files() -> Vec<PendingOpenFile> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        collect_paths_from_args(std::env::args().skip(1))
            .into_iter()
            .filter_map(|path| {
                if !path.is_file() {
                    return None;
                }
                let path_str = path.to_str()?.to_string();
                let content = fs::read_to_string(&path).ok()?;
                Some(PendingOpenFile {
                    path: path_str,
                    content,
                })
            })
            .collect()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        vec![]
    }
}

fn boot_preview_files(boot: &[PendingOpenFile]) -> Vec<BootPreviewFile> {
    boot
        .iter()
        .map(|item| {
            let truncated = item.content.len() > BOOT_PREVIEW_MAX_BYTES;
            let content = if truncated {
                item.content.chars().take(BOOT_PREVIEW_MAX_BYTES).collect()
            } else {
                item.content.clone()
            };
            BootPreviewFile {
                path: item.path.clone(),
                content,
                truncated,
            }
        })
        .collect()
}

fn pending_boot_snapshot(webview: &Webview) -> Vec<PendingOpenFile> {
    let state = webview.app_handle().state::<PendingOpenFiles>();
    let pending = state.0.lock().unwrap();
    pending.clone()
}

fn inject_boot_data(webview: &Webview) {
    let boot = pending_boot_snapshot(webview);
    if boot.is_empty() {
        return;
    }

    if let Ok(json) = serde_json::to_string(&boot_preview_files(&boot)) {
        let _ = webview.eval(format!(
            "(function(d){{window.__MARKFLY_BOOT__=d;window.__markflyApplyBoot&&window.__markflyApplyBoot(d);}})({json});"
        ));
    }
}

fn has_pending_boot(webview: &Webview) -> bool {
    !pending_boot_snapshot(webview).is_empty()
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }
}

fn setup_app_menus(app: &tauri::AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    let app_menu = SubmenuBuilder::new(app, "MarkFly")
        .item(&PredefinedMenuItem::about(app, Some("关于 MarkFly"), None)?)
        .separator()
        .item(&PredefinedMenuItem::services(app, Some("服务"))?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("隐藏 MarkFly"))?)
        .item(&PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?)
        .item(&PredefinedMenuItem::show_all(app, Some("全部显示"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("退出"))?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "文件")
        .text("new-file", "新建文件")
        .text("open-file", "打开文件...")
        .text("save-file", "保存文件")
        .text("save-file-as", "另存为...")
        .separator()
        .text("exit", "退出")
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "编辑")
        .item(&PredefinedMenuItem::undo(app, Some("撤销"))?)
        .item(&PredefinedMenuItem::redo(app, Some("重做"))?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, Some("剪切"))?)
        .item(&PredefinedMenuItem::copy(app, Some("复制"))?)
        .item(&PredefinedMenuItem::paste(app, Some("粘贴"))?)
        .build()?;

    let toggle_sidebar = MenuItemBuilder::with_id("toggle-sidebar", "显示/隐藏侧边栏")
        .accelerator("CmdOrCtrl+B")
        .build(app)?;

    let view_menu = SubmenuBuilder::new(app, "视图")
        .item(&toggle_sidebar)
        .separator()
        .text("toggle-preview", "切换预览模式")
        .text("toggle-theme", "切换主题")
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "帮助")
        .text("about", "关于 MarkFly")
        .build()?;

    #[cfg(target_os = "macos")]
    {
        let menu = MenuBuilder::new(app)
            .item(&app_menu)
            .item(&file_menu)
            .item(&edit_menu)
            .item(&view_menu)
            .item(&help_menu)
            .build()?;
        menu.set_as_app_menu()?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let menu = MenuBuilder::new(app)
            .item(&file_menu)
            .item(&edit_menu)
            .item(&view_menu)
            .item(&help_menu)
            .build()?;
        if let Some(window) = app.get_webview_window("main") {
            window.set_menu(menu)?;
        }
    }

    app.on_menu_event(|app_handle, event| {
        let event_id = event.id().0.as_str();
        match event_id {
            "new-file" | "open-file" | "save-file" | "save-file-as" | "toggle-sidebar"
            | "toggle-preview" | "toggle-theme" => {
                let _ = app_handle.emit("menu", event_id);
            }
            "exit" | "quit" => {
                app_handle.exit(0);
            }
            _ => {}
        }
    });

    Ok(())
}

fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// Tauri 命令：获取应用信息
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "MarkFly",
        "version": "0.1.0",
        "description": "跨平台 Markdown 编辑器"
    })
}

#[tauri::command]
fn get_pending_open_files(state: State<PendingOpenFiles>) -> Vec<PendingOpenFile> {
    state.0.lock().unwrap().drain(..).collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_boot = read_initial_boot_files();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(PendingOpenFiles(Mutex::new(initial_boot)))
        .manage(file_watcher::FileWatcherState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_pending_open_files,
            file_watcher::sync_file_watches
        ]);

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let paths = resolve_open_files(app, collect_paths_from_args(args.into_iter().skip(1)));
            dispatch_open_files(app, paths);
            focus_main_window(app);
        }));
    }

    builder
        .on_page_load(|webview: &Webview, payload| {
            match payload.event() {
                PageLoadEvent::Started => {
                    inject_boot_data(webview);
                    if has_pending_boot(webview) {
                        show_main_window(webview.app_handle());
                    }
                }
                PageLoadEvent::Finished => {
                    // 仅同步 boot 数据；渲染由 boot.js 根据 __MARKFLY_BOOT_DISMISSED__ 决定是否跳过
                    inject_boot_data(webview);
                    show_main_window(webview.app_handle());
                }
            }
        })
        .setup(|app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                let _ = store_pending_open_files(
                    app.handle(),
                    collect_paths_from_args(std::env::args().skip(1)),
                );
            }

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(80));
                let menu_handle = app_handle.clone();
                let _ = app_handle.run_on_main_thread(move || {
                    if let Err(error) = setup_app_menus(&menu_handle) {
                        eprintln!("菜单创建失败: {error}");
                    }
                });
            });

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.open_devtools();
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event {
                let paths = urls
                    .into_iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .collect::<Vec<_>>();
                let valid_paths = resolve_open_files(app, paths);
                dispatch_open_files(app, valid_paths);
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (app, event);
            }
        });
}
