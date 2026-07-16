// MarkFly - 跨平台 Markdown 编辑器
// 基于 Tauri + Vue + ByteMD 构建

mod file_watcher;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Emitter, Manager, State};
use tauri_plugin_fs::FsExt;

struct PendingOpenFiles(Mutex<Vec<String>>);

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
    let valid_paths = resolve_open_files(app, paths);

    if !valid_paths.is_empty() {
        app.state::<PendingOpenFiles>()
            .0
            .lock()
            .unwrap()
            .extend(valid_paths.clone());
    }

    valid_paths
}

fn dispatch_open_files(app: &tauri::AppHandle, paths: Vec<String>) {
    if paths.is_empty() {
        return;
    }

    let _ = app.emit("open-file-path", paths);
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
fn get_pending_open_files(state: State<PendingOpenFiles>) -> Vec<String> {
    state.0.lock().unwrap().drain(..).collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(PendingOpenFiles(Mutex::new(Vec::new())))
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
        .setup(|app| {
            println!("开始创建菜单...");

            // 在 macOS 上，第一个菜单会自动成为应用菜单，所以我们创建一个空的应用菜单
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

            // 创建文件菜单
            let file_menu = SubmenuBuilder::new(app, "文件")
                .text("new-file", "新建文件")
                .text("open-file", "打开文件...")
                .text("save-file", "保存文件")
                .text("save-file-as", "另存为...")
                .separator()
                .text("exit", "退出")
                .build()?;
            println!("文件菜单创建完成");

            // 创建编辑菜单，使用预定义项
            let edit_menu = SubmenuBuilder::new(app, "编辑")
                .item(&PredefinedMenuItem::undo(app, Some("撤销"))?)
                .item(&PredefinedMenuItem::redo(app, Some("重做"))?)
                .separator()
                .item(&PredefinedMenuItem::cut(app, Some("剪切"))?)
                .item(&PredefinedMenuItem::copy(app, Some("复制"))?)
                .item(&PredefinedMenuItem::paste(app, Some("粘贴"))?)
                .build()?;
            println!("编辑菜单创建完成");

            let toggle_sidebar = MenuItemBuilder::with_id("toggle-sidebar", "显示/隐藏侧边栏")
                .accelerator("CmdOrCtrl+B")
                .build(app)?;

            let view_menu = SubmenuBuilder::new(app, "视图")
                .item(&toggle_sidebar)
                .separator()
                .text("toggle-preview", "切换预览模式")
                .text("toggle-theme", "切换主题")
                .build()?;
            println!("视图菜单创建完成");

            let help_menu = SubmenuBuilder::new(app, "帮助")
                .text("about", "关于 MarkFly")
                .build()?;
            println!("帮助菜单创建完成");

            // 在 macOS 上使用 set_as_app_menu，在其他平台上使用 window.set_menu
            #[cfg(target_os = "macos")]
            {
                let menu = MenuBuilder::new(app)
                    .item(&app_menu) // 应用菜单作为第一个菜单
                    .item(&file_menu)
                    .item(&edit_menu)
                    .item(&view_menu)
                    .item(&help_menu)
                    .build()?;
                menu.set_as_app_menu()?;
                println!("macOS 应用菜单设置完成");
            }

            #[cfg(not(target_os = "macos"))]
            {
                let menu = MenuBuilder::new(app)
                    .item(&file_menu)
                    .item(&edit_menu)
                    .item(&view_menu)
                    .item(&help_menu)
                    .build()?;
                let window = app.get_webview_window("main").unwrap();
                window.set_menu(menu)?;
                println!("窗口菜单设置完成");
            }

            // 监听菜单事件
            app.on_menu_event(move |app_handle, event| {
                let event_id = event.id().0.as_str();
                println!("收到菜单事件: {}", event_id);
                match event_id {
                    "new-file" | "open-file" | "save-file" | "save-file-as" | "toggle-sidebar"
                    | "toggle-preview" | "toggle-theme" => {
                        // 发送事件到前端
                        let _ = app_handle.emit("menu", event_id);
                        println!("发送菜单事件到前端: {}", event_id);
                    }
                    "exit" | "quit" => {
                        println!("退出应用");
                        app_handle.exit(0);
                    }
                    "about" => {
                        println!("关于 MarkFly");
                    }
                    _ => {
                        println!("未知菜单事件: {}", event_id);
                    }
                }
            });

            // Windows / Linux: 从命令行参数读取右键打开的文件
            #[cfg(any(windows, target_os = "linux"))]
            {
                let _ = store_pending_open_files(
                    app.handle(),
                    collect_paths_from_args(std::env::args().skip(1)),
                );
            }

            // 应用启动时的初始化逻辑
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            println!("应用初始化完成");
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
