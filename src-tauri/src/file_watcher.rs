use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::event::{EventKind, ModifyKind};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager, State};

const DEBOUNCE: Duration = Duration::from_millis(400);

pub struct FileWatcherState {
    inner: Mutex<FileWatcherInner>,
}

struct FileWatcherInner {
    watcher: Option<RecommendedWatcher>,
    watched_files: HashSet<PathBuf>,
    watched_dirs: HashSet<PathBuf>,
    last_emit: HashMap<PathBuf, Instant>,
}

impl FileWatcherState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(FileWatcherInner {
                watcher: None,
                watched_files: HashSet::new(),
                watched_dirs: HashSet::new(),
                last_emit: HashMap::new(),
            }),
        }
    }
}

fn normalize_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn should_emit(inner: &mut FileWatcherInner, path: &Path) -> bool {
    let now = Instant::now();
    if let Some(last) = inner.last_emit.get(path) {
        if now.duration_since(*last) < DEBOUNCE {
            return false;
        }
    }
    inner.last_emit.insert(path.to_path_buf(), now);
    true
}

fn ensure_watcher(app: &AppHandle, inner: &mut FileWatcherInner) -> Result<(), String> {
    if inner.watcher.is_some() {
        return Ok(());
    }

    let app_handle = app.clone();
    let watcher = RecommendedWatcher::new(
        move |result| {
            if let Ok(event) = result {
                handle_notify_event(&app_handle, event);
            }
        },
        Config::default(),
    )
    .map_err(|error| error.to_string())?;

    inner.watcher = Some(watcher);
    Ok(())
}

fn handle_notify_event(app: &AppHandle, event: notify::Event) {
    let is_content_change = matches!(
        event.kind,
        EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(_))
            | EventKind::Modify(ModifyKind::Any)
            | EventKind::Create(_)
    );

    if !is_content_change {
        return;
    }

    let state = app.state::<FileWatcherState>();
    let mut inner = match state.inner.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    for path in event.paths {
        let matched = inner.watched_files.iter().any(|watched| watched == &path);
        if !matched {
            continue;
        }

        if !should_emit(&mut inner, &path) {
            continue;
        }

        if let Some(path_str) = path.to_str() {
            let _ = app.emit("file-changed", path_str.to_string());
        }
    }
}

#[tauri::command]
pub fn sync_file_watches(
    app: AppHandle,
    state: State<'_, FileWatcherState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|_| "lock poisoned".to_string())?;
    ensure_watcher(&app, &mut inner)?;

    let next_files: HashSet<PathBuf> = paths
        .into_iter()
        .map(|path| normalize_path(&path))
        .filter(|path| path.is_file())
        .collect();

    let next_dirs: HashSet<PathBuf> = next_files
        .iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();

    let dirs_to_unwatch: Vec<PathBuf> = inner
        .watched_dirs
        .difference(&next_dirs)
        .cloned()
        .collect();
    let dirs_to_watch: Vec<PathBuf> = next_dirs
        .difference(&inner.watched_dirs)
        .cloned()
        .collect();

    if let Some(watcher) = inner.watcher.as_mut() {
        for dir in dirs_to_unwatch {
            let _ = watcher.unwatch(&dir);
        }

        for dir in dirs_to_watch {
            watcher
                .watch(&dir, RecursiveMode::NonRecursive)
                .map_err(|error| error.to_string())?;
        }
    }

    inner.watched_files = next_files;
    inner.watched_dirs = next_dirs;
    Ok(())
}
