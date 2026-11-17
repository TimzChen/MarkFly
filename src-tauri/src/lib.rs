// MarkFly - 跨平台 Markdown 编辑器
// 基于 Tauri + Vue + ByteMD 构建

mod file_watcher;
mod instance_guard;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::collections::HashSet;

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::webview::PageLoadEvent;
use tauri::{Emitter, Manager, State, Webview};
use tauri_plugin_fs::FsExt;
use tauri_plugin_opener::OpenerExt;

const PROJECT_HOME_URL: &str = "https://github.com/TimzChen/MarkFly";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PendingOpenFile {
    path: String,
    content: String,
}

struct PendingOpenFiles(Mutex<Vec<PendingOpenFile>>);

struct RecentMenuItems(Mutex<Vec<String>>);
/// 全屏时不要把原生菜单再画回去（例如刷新「最近打开」时）
struct HideAppMenu(Mutex<bool>);

#[cfg(windows)]
struct SavedWindowFrame {
    placement: windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT,
    style: i32,
}

#[cfg(windows)]
struct FullscreenFrame(Mutex<Option<SavedWindowFrame>>);

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

/// 刷新前重新从磁盘读取，避免 WebView Refresh 仍使用启动时的旧内容
fn refresh_pending_contents(pending: &mut Vec<PendingOpenFile>) {
    for item in pending.iter_mut() {
        if let Ok(content) = fs::read_to_string(&item.path) {
            item.content = content;
        }
    }
}

fn pending_boot_snapshot(webview: &Webview) -> Vec<PendingOpenFile> {
    let state = webview.app_handle().state::<PendingOpenFiles>();
    let mut pending = state.0.lock().unwrap();
    refresh_pending_contents(&mut pending);
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

fn recent_menu_label(path: &str) -> String {
    let path = std::path::Path::new(path);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("未命名.md");
    let parent = path
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let raw = if parent.is_empty() {
        name.to_string()
    } else {
        format!("{name} — {parent}")
    };
    let raw = raw.replace('&', "&&");
    let char_count = raw.chars().count();
    if char_count > 48 {
        format!("{}…", raw.chars().take(47).collect::<String>())
    } else {
        raw
    }
}

fn build_recent_submenu(
    app: &tauri::AppHandle,
) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    let state = app.state::<RecentMenuItems>();
    let recents = state.0.lock().unwrap().clone();
    let mut builder = SubmenuBuilder::new(app, "最近打开");

    if recents.is_empty() {
        let empty = MenuItemBuilder::with_id("recent-empty", "暂无记录")
            .enabled(false)
            .build(app)?;
        builder = builder.item(&empty);
    } else {
        let mut items = Vec::new();
        for (index, path) in recents.iter().enumerate() {
            items.push(
                MenuItemBuilder::with_id(format!("recent-{index}"), recent_menu_label(path))
                    .build(app)?,
            );
        }
        for item in &items {
            builder = builder.item(item);
        }
        builder = builder.separator().text("recent-clear", "清空最近打开");
    }

    builder.build()
}

fn apply_app_menus(app: &tauri::AppHandle) -> tauri::Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        if *app.state::<HideAppMenu>().0.lock().unwrap() {
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    let app_menu = SubmenuBuilder::new(app, "MarkFly")
        .text("about", "关于 MarkFly")
        .separator()
        .item(&PredefinedMenuItem::services(app, Some("服务"))?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("隐藏 MarkFly"))?)
        .item(&PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?)
        .item(&PredefinedMenuItem::show_all(app, Some("全部显示"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("退出"))?)
        .build()?;

    let recent_menu = build_recent_submenu(app)?;
    let file_menu = SubmenuBuilder::new(app, "文件")
        .text("new-file", "新建文件")
        .text("open-file", "打开文件...")
        .text("open-folder", "打开目录...")
        .text("save-file", "保存文件")
        .text("save-file-as", "另存为...")
        .separator()
        .item(&recent_menu)
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

    let toggle_fullscreen = MenuItemBuilder::with_id("toggle-fullscreen", "全屏\tF11")
        .build(app)?;

    let view_menu = SubmenuBuilder::new(app, "视图")
        .item(&toggle_sidebar)
        .separator()
        .text("toggle-preview", "切换预览模式")
        .text("toggle-theme", "切换主题")
        .separator()
        .item(&toggle_fullscreen)
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

    Ok(())
}

fn register_menu_event_handler(app: &tauri::AppHandle) {
    app.on_menu_event(|app_handle, event| {
        let event_id = event.id().0.as_str();
        match event_id {
            "new-file" | "open-file" | "open-folder" | "save-file" | "save-file-as" | "toggle-sidebar"
            | "toggle-preview" | "toggle-theme" | "toggle-fullscreen" | "recent-clear" => {
                let payload = if event_id == "recent-clear" {
                    "clear-recents"
                } else {
                    event_id
                };
                let _ = app_handle.emit("menu", payload);
            }
            "exit" | "quit" => {
                app_handle.exit(0);
            }
            "about" => {
                let _ = app_handle
                    .opener()
                    .open_url(PROJECT_HOME_URL, None::<&str>);
            }
            other => {
                if let Some(index) = other
                    .strip_prefix("recent-")
                    .and_then(|value| value.parse::<usize>().ok())
                {
                    let state = app_handle.state::<RecentMenuItems>();
                    let path = state
                        .0
                        .lock()
                        .ok()
                        .and_then(|recents| recents.get(index).cloned());
                    if let Some(path) = path {
                        dispatch_open_files(app_handle, vec![path]);
                    }
                }
            }
        }
    });
}

fn setup_app_menus(app: &tauri::AppHandle) -> tauri::Result<()> {
    apply_app_menus(app)?;
    register_menu_event_handler(app);
    Ok(())
}

fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn handle_second_instance(app: &tauri::AppHandle, args: Vec<String>) {
    let paths = resolve_open_files(app, collect_paths_from_args(args.into_iter().skip(1)));
    dispatch_open_files(app, paths);
    focus_main_window(app);
}

// Tauri 命令：获取应用信息
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "MarkFly",
        "version": "0.1.4",
        "description": "跨平台 Markdown 编辑器"
    })
}

#[tauri::command]
fn get_pending_open_files(state: State<PendingOpenFiles>) -> Vec<PendingOpenFile> {
    let mut pending = state.0.lock().unwrap();
    refresh_pending_contents(&mut pending);
    pending.drain(..).collect()
}

#[tauri::command]
fn allow_preview_asset(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Ok(());
    }
    app.fs_scope()
        .allow_file(&path)
        .map_err(|error| error.to_string())
}

/// 打开本地文本文件：先纳入 FS scope，再用标准库读取（供预览链接跳转）
#[tauri::command]
fn read_local_text_file(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let path = PathBuf::from(&path);
    if !path.is_file() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    app.fs_scope()
        .allow_file(&path)
        .map_err(|error| error.to_string())?;
    fs::read_to_string(&path).map_err(|error| format!("读取失败: {error}"))
}

/// 预览区外链：用系统默认浏览器 / 邮件客户端打开
#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let trimmed = url.trim();
    let lower = trimmed.to_lowercase();
    let allowed = lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:");
    if !allowed {
        return Err("仅允许打开 http(s) 或 mailto 链接".to_string());
    }
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|error| error.to_string())
}

/// 在系统文件管理器中定位并选中该文件（类似 Reveal in File Explorer）
#[tauri::command]
fn reveal_in_file_manager(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        // 去掉 Windows canonicalize 产生的 \\?\ 前缀，避免 explorer 无法识别
        let absolute = fs::canonicalize(&path).unwrap_or(path);
        let display = absolute
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_string();
        std::process::Command::new("explorer")
            .arg(format!("/select,{display}"))
            .spawn()
            .map_err(|error| format!("打开资源管理器失败: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|error| format!("打开 Finder 失败: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let parent = path
            .parent()
            .ok_or_else(|| "无法获取所在目录".to_string())?;
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|error| format!("打开文件管理器失败: {error}"))?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("当前平台不支持打开所在目录".to_string())
    }
}

/// 启动时去掉已经不存在的最近打开记录，并把仍在的路径纳入 FS scope
#[tauri::command]
fn filter_existing_files(app: tauri::AppHandle, paths: Vec<String>) -> Vec<String> {
    let fs_scope = app.fs_scope();
    let mut existing = Vec::new();
    for path in paths {
        let buf = PathBuf::from(&path);
        if !buf.is_file() {
            continue;
        }
        let _ = fs_scope.allow_file(&buf);
        existing.push(path);
    }
    existing
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListedMarkdownFile {
    path: String,
    name: String,
    relative_dir: String,
}

struct MarkdownWalkEntry {
    path: PathBuf,
    name: String,
    relative_dir: String,
}

fn is_markdown_filename(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            lower == "md" || lower == "markdown" || lower == "txt"
        })
        .unwrap_or(false)
}

fn should_skip_dir_name(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    matches!(
        name.to_ascii_lowercase().as_str(),
        "node_modules" | "dist" | "target" | "vendor" | "__pycache__"
    )
}

fn entry_is_dir(entry: &fs::DirEntry) -> bool {
    match entry.file_type() {
        Ok(file_type) if file_type.is_dir() => true,
        Ok(file_type) if file_type.is_symlink() => entry.path().is_dir(),
        Ok(_) => false,
        Err(_) => entry.path().is_dir(),
    }
}

fn relative_dir_of(root: &Path, file: &Path) -> String {
    let Some(parent) = file.parent() else {
        return String::new();
    };
    match parent.strip_prefix(root) {
        Ok(relative) if relative.as_os_str().is_empty() => String::new(),
        Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
        Err(_) => parent
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}

fn file_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("未命名.md")
        .to_string()
}

fn walk_markdown_files(root: &Path, dir: &Path, depth: u32, out: &mut Vec<MarkdownWalkEntry>) {
    const MAX_FILES: usize = 2000;
    const MAX_DEPTH: u32 = 12;
    if out.len() >= MAX_FILES || depth > MAX_DEPTH {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry_is_dir(&entry) {
            if should_skip_dir_name(&name) {
                continue;
            }
            dirs.push(path);
            continue;
        }
        if is_markdown_filename(&path) {
            files.push(path);
        }
    }
    files.sort();
    dirs.sort();

    for path in files {
        if out.len() >= MAX_FILES {
            return;
        }
        out.push(MarkdownWalkEntry {
            name: file_display_name(&path),
            relative_dir: relative_dir_of(root, &path),
            path,
        });
    }

    for child in dirs {
        walk_markdown_files(root, &child, depth + 1, out);
        if out.len() >= MAX_FILES {
            return;
        }
    }
}

/// 列出用户选中目录及其子目录下的 Markdown / 文本文件，供知识库「目录」页签浏览
#[tauri::command]
fn list_markdown_files(app: tauri::AppHandle, path: String) -> Result<Vec<ListedMarkdownFile>, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("不是有效文件夹: {}", dir.display()));
    }
    let _ = app.fs_scope().allow_directory(&dir, true);
    let mut found = Vec::new();
    walk_markdown_files(&dir, &dir, 0, &mut found);

    let fs_scope = app.fs_scope();
    let mut files = Vec::with_capacity(found.len());
    for entry in found {
        let _ = fs_scope.allow_file(&entry.path);
        let Some(path_str) = entry.path.to_str() else {
            continue;
        };
        files.push(ListedMarkdownFile {
            path: path_str.to_string(),
            name: entry.name,
            relative_dir: entry.relative_dir,
        });
    }
    Ok(files)
}

#[cfg(test)]
mod list_markdown_tests {
    use super::*;

    fn write_md(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn listed_keys(entries: &[MarkdownWalkEntry]) -> Vec<String> {
        entries
            .iter()
            .map(|item| {
                if item.relative_dir.is_empty() {
                    item.name.clone()
                } else {
                    format!("{}/{}", item.relative_dir, item.name)
                }
            })
            .collect()
    }

    #[test]
    fn walks_nested_markdown_and_skips_junk() {
        let root = std::env::temp_dir().join(format!(
            "markfly-md-walk-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        write_md(&root.join("root.md"), "# root");
        write_md(&root.join("sub").join("child.md"), "# child");
        write_md(&root.join("sub").join("deep").join("nested.md"), "# nested");
        write_md(&root.join(".hidden").join("secret.md"), "# hidden");
        write_md(&root.join("node_modules").join("pkg.md"), "# pkg");
        write_md(&root.join("notes.txt"), "txt");

        let mut out = Vec::new();
        walk_markdown_files(&root, &root, 0, &mut out);
        let keys = listed_keys(&out);
        let _ = fs::remove_dir_all(&root);

        assert!(keys.contains(&"root.md".to_string()));
        assert!(keys.contains(&"notes.txt".to_string()));
        assert!(keys.contains(&"sub/child.md".to_string()));
        assert!(keys.contains(&"sub/deep/nested.md".to_string()));
        assert!(!keys.iter().any(|item| item.contains("secret")));
        assert!(!keys.iter().any(|item| item.contains("pkg")));
    }
}

fn is_case_only_rename(from: &PathBuf, to: &PathBuf) -> bool {
    let (Some(from_name), Some(to_name)) = (from.file_name(), to.file_name()) else {
        return false;
    };
    from.parent() == to.parent() && from_name.eq_ignore_ascii_case(to_name) && from_name != to_name
}

/// 重命名磁盘文件，返回新路径。Windows 上仅改大小写时走临时名，避免 NTFS 判成同名冲突。
#[tauri::command]
fn rename_local_file(app: tauri::AppHandle, from: String, to: String) -> Result<String, String> {
    let from_path = PathBuf::from(&from);
    let to_path = PathBuf::from(&to);
    if !from_path.is_file() {
        return Err(format!("文件不存在: {}", from_path.display()));
    }
    if from_path == to_path {
        return Ok(from);
    }
    let case_only = is_case_only_rename(&from_path, &to_path);
    if to_path.exists() && !case_only {
        return Err(format!("已存在同名文件: {}", to_path.display()));
    }
    if case_only {
        let parent = from_path.parent().ok_or_else(|| "无法确定所在目录".to_string())?;
        let tmp = parent.join(format!(
            ".{}-{}.markfly-tmp",
            from_path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("file"),
            std::process::id()
        ));
        fs::rename(&from_path, &tmp).map_err(|error| format!("重命名失败: {error}"))?;
        if let Err(error) = fs::rename(&tmp, &to_path) {
            let _ = fs::rename(&tmp, &from_path);
            return Err(format!("重命名失败: {error}"));
        }
    } else {
        fs::rename(&from_path, &to_path).map_err(|error| format!("重命名失败: {error}"))?;
    }
    let _ = app.fs_scope().allow_file(&to_path);
    to_path
        .to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| "新路径无法表示".to_string())
}

/// 全屏时收起原生菜单，退出全屏后再挂回去。
/// 只允许从后台线程调用 `remove_menu`/`set_menu`：它们自己会 marshal 到 UI 线程。
/// 若再包一层 `run_on_main_thread`，Windows 上会在 UI 线程里等自己，按一两次 F11 就卡死。
#[tauri::command]
async fn set_app_menu_visible(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = (app, visible);
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        *app.state::<HideAppMenu>().0.lock().unwrap() = !visible;
        if visible {
            apply_app_menus(&app).map_err(|error| format!("恢复菜单失败: {error}"))
        } else if let Some(window) = app.get_webview_window("main") {
            window
                .remove_menu()
                .map(|_| ())
                .map_err(|error| format!("隐藏菜单失败: {error}"))
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
fn apply_borderless_fullscreen(app: &tauri::AppHandle, fullscreen: bool) -> Result<bool, String> {
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, GetWindowPlacement, SetWindowLongW, SetWindowPlacement, SetWindowPos,
        GWL_STYLE, HWND_TOP, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOOWNERZORDER,
        SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WINDOWPLACEMENT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    let window = app.get_webview_window("main").ok_or("没有主窗口")?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0 as _;
    let saved = app.state::<FullscreenFrame>();
    let mut slot = saved.0.lock().map_err(|_| "全屏状态锁失败")?;

    unsafe {
        if fullscreen {
            if slot.is_some() {
                return Ok(true);
            }
            let mut placement = std::mem::zeroed::<WINDOWPLACEMENT>();
            placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
            if GetWindowPlacement(hwnd, &mut placement) == 0 {
                return Err("读取窗口位置失败".into());
            }
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            let mut info = std::mem::zeroed::<MONITORINFO>();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut info) == 0 {
                return Err("读取屏幕信息失败".into());
            }
            *slot = Some(SavedWindowFrame { placement, style });
            drop(slot);
            SetWindowLongW(
                hwnd,
                GWL_STYLE,
                (style & !WS_OVERLAPPEDWINDOW as i32) | WS_VISIBLE as i32,
            );
            let rect = info.rcMonitor;
            SetWindowPos(
                hwnd,
                HWND_TOP,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_FRAMECHANGED | SWP_NOOWNERZORDER | SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
            );
            Ok(true)
        } else {
            let Some(frame) = slot.take() else {
                return Ok(false);
            };
            drop(slot);
            SetWindowLongW(hwnd, GWL_STYLE, frame.style);
            SetWindowPlacement(hwnd, &frame.placement);
            SetWindowPos(
                hwnd,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE
                    | SWP_NOSIZE
                    | SWP_NOZORDER
                    | SWP_FRAMECHANGED
                    | SWP_ASYNCWINDOWPOS
                    | SWP_SHOWWINDOW,
            );
            Ok(false)
        }
    }
}

/// Windows 不用 tao 的 `setFullscreen` / muda 拆菜单：那两条都会在 UI 线程里重入卡死。
/// 这里只改窗口样式铺满屏幕，菜单随标题栏一起隐掉，退出时按保存的 placement 还原。
#[tauri::command]
async fn set_window_fullscreen(app: tauri::AppHandle, fullscreen: bool) -> Result<bool, String> {
    *app.state::<HideAppMenu>().0.lock().unwrap() = fullscreen;

    #[cfg(windows)]
    {
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let app_clone = app.clone();
        app.run_on_main_thread(move || {
            let result = apply_borderless_fullscreen(&app_clone, fullscreen);
            let _ = done_tx.send(result);
        })
        .map_err(|error| error.to_string())?;
        let applied = tauri::async_runtime::spawn_blocking(move || {
            done_rx.recv_timeout(std::time::Duration::from_millis(800))
        })
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or(Ok(fullscreen));
        if let Some(window) = app.get_webview_window("main") {
            let menu_result = if fullscreen {
                window.hide_menu()
            } else {
                window.show_menu()
            };
            if let Err(error) = menu_result {
                eprintln!("切换菜单显示失败: {error}");
            }
        }
        return applied;
    }

    #[cfg(not(windows))]
    {
        let window = app.get_webview_window("main").ok_or("没有主窗口")?;
        window
            .set_fullscreen(fullscreen)
            .map_err(|error| error.to_string())?;
        window.is_fullscreen().map_err(|error| error.to_string())
    }
}

/// 同步「文件 → 最近打开」子菜单，与知识库列表保持一致
#[tauri::command]
fn update_recent_menu(app: tauri::AppHandle, paths: Vec<String>) -> Result<(), String> {
    {
        let state = app.state::<RecentMenuItems>();
        let mut recents = state.0.lock().unwrap();
        *recents = paths;
    }
    let app_clone = app.clone();
    app.run_on_main_thread(move || {
        if let Err(error) = apply_app_menus(&app_clone) {
            eprintln!("更新最近打开菜单失败: {error}");
        }
    })
    .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_boot = read_initial_boot_files();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(PendingOpenFiles(Mutex::new(initial_boot)))
        .manage(RecentMenuItems(Mutex::new(Vec::new())))
        .manage(HideAppMenu(Mutex::new(false)))
        .manage(file_watcher::FileWatcherState::new())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_pending_open_files,
            allow_preview_asset,
            read_local_text_file,
            open_external_url,
            reveal_in_file_manager,
            filter_existing_files,
            list_markdown_files,
            rename_local_file,
            set_app_menu_visible,
            set_window_fullscreen,
            update_recent_menu,
            file_watcher::sync_file_watches
        ]);

    #[cfg(windows)]
    {
        builder = builder.manage(FullscreenFrame(Mutex::new(None)));
    }

    #[cfg(windows)]
    {
        builder = builder.plugin(instance_guard::init(|app, args, _cwd| {
            handle_second_instance(app, args);
        }));
    }

    #[cfg(all(desktop, not(windows)))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            handle_second_instance(app, args);
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
                    #[cfg(windows)]
                    {
                        let app = webview.app_handle().clone();
                        let _ = app.clone().run_on_main_thread(move || {
                            instance_guard::cache_main_window_hwnd(&app);
                        });
                    }
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

            #[cfg(windows)]
            {
                instance_guard::cache_main_window_hwnd(app.handle());
                instance_guard::spawn_self_watchdog();
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
            match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    instance_guard::schedule_force_exit();
                }
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Opened { urls } => {
                    let paths = urls
                        .into_iter()
                        .filter_map(|url| url.to_file_path().ok())
                        .collect::<Vec<_>>();
                    let valid_paths = resolve_open_files(app, paths);
                    dispatch_open_files(app, valid_paths);
                }
                _ => {}
            }
            #[cfg(not(target_os = "macos"))]
            let _ = app;
        });
}
