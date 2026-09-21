use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::event::{EventKind, ModifyKind};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager, State};

const DEBOUNCE: Duration = Duration::from_millis(400);

pub struct FileWatcherState {
    watcher: Mutex<Option<RecommendedWatcher>>,
    watched: Mutex<WatchedState>,
}

struct WatchedState {
    files: HashSet<PathBuf>,
    dirs: HashSet<PathBuf>,
    last_emit: HashMap<PathBuf, Instant>,
}

impl FileWatcherState {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
            watched: Mutex::new(WatchedState {
                files: HashSet::new(),
                dirs: HashSet::new(),
                last_emit: HashMap::new(),
            }),
        }
    }
}

fn normalize_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn should_emit(watched: &mut WatchedState, path: &Path) -> bool {
    let now = Instant::now();
    if let Some(last) = watched.last_emit.get(path) {
        if now.duration_since(*last) < DEBOUNCE {
            return false;
        }
    }
    watched.last_emit.insert(path.to_path_buf(), now);
    true
}

fn watch_dir_diff(
    current: &HashSet<PathBuf>,
    next: &HashSet<PathBuf>,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let unwatch = current.difference(next).cloned().collect();
    let watch = next.difference(current).cloned().collect();
    (unwatch, watch)
}

#[cfg(test)]
fn dirs_after_apply(
    current: &HashSet<PathBuf>,
    unwatched: &[PathBuf],
    watched_ok: &[PathBuf],
) -> HashSet<PathBuf> {
    let mut dirs = current.clone();
    for dir in unwatched {
        dirs.remove(dir);
    }
    for dir in watched_ok {
        dirs.insert(dir.clone());
    }
    dirs
}

fn create_watcher(app: &AppHandle) -> Result<RecommendedWatcher, String> {
    let app_handle = app.clone();
    RecommendedWatcher::new(
        move |result| {
            if let Ok(event) = result {
                handle_notify_event(&app_handle, event);
            }
        },
        Config::default(),
    )
    .map_err(|error| error.to_string())
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
    let mut watched = match state.watched.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let mut to_emit = Vec::new();
    for path in event.paths {
        let matched = watched.files.iter().any(|watched_file| watched_file == &path);
        if !matched {
            continue;
        }

        if !should_emit(&mut watched, &path) {
            continue;
        }

        if let Some(path_str) = path.to_str() {
            to_emit.push(path_str.to_string());
        }
    }
    drop(watched);

    for path in to_emit {
        let _ = app.emit("file-changed", path);
    }
}

#[tauri::command]
pub fn sync_file_watches(
    app: AppHandle,
    state: State<'_, FileWatcherState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let next_files: HashSet<PathBuf> = paths
        .into_iter()
        .map(|path| normalize_path(&path))
        .filter(|path| path.is_file())
        .collect();

    let next_dirs: HashSet<PathBuf> = next_files
        .iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();

    // 先拿 watcher 锁，再短时间拿 watched 锁算目录差。
    // watch/unwatch 绝不能和回调抢同一把锁：notify 在 unwatch 时会等回调结束，
    // 回调若再锁同一把 Mutex，Windows 上会把整个 UI 卡死。
    let mut watcher_guard = state
        .watcher
        .lock()
        .map_err(|_| "watcher lock poisoned".to_string())?;
    if watcher_guard.is_none() {
        *watcher_guard = Some(create_watcher(&app)?);
    }

    let (dirs_to_unwatch, dirs_to_watch) = {
        let mut watched = state
            .watched
            .lock()
            .map_err(|_| "watch list lock poisoned".to_string())?;
        let diff = watch_dir_diff(&watched.dirs, &next_dirs);
        watched.files = next_files;
        diff
    };

    if let Some(watcher) = watcher_guard.as_mut() {
        for dir in dirs_to_unwatch {
            if let Err(error) = watcher.unwatch(&dir) {
                eprintln!("MarkFly: stop watching {} failed: {error}", dir.display());
            }
            if let Ok(mut watched) = state.watched.lock() {
                watched.dirs.remove(&dir);
            }
        }

        for dir in dirs_to_watch {
            watcher
                .watch(&dir, RecursiveMode::NonRecursive)
                .map_err(|error| error.to_string())?;
            if let Ok(mut watched) = state.watched.lock() {
                watched.dirs.insert(dir);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_suppresses_immediate_repeat() {
        let mut watched = WatchedState {
            files: HashSet::new(),
            dirs: HashSet::new(),
            last_emit: HashMap::new(),
        };
        let path = Path::new("C:\\notes\\a.md");
        assert!(should_emit(&mut watched, path));
        assert!(!should_emit(&mut watched, path));
    }

    #[test]
    fn watch_dir_diff_adds_and_removes() {
        let current = HashSet::from([PathBuf::from("C:\\old"), PathBuf::from("C:\\keep")]);
        let next = HashSet::from([PathBuf::from("C:\\keep"), PathBuf::from("C:\\new")]);
        let (unwatch, watch) = watch_dir_diff(&current, &next);
        assert_eq!(unwatch.into_iter().collect::<HashSet<_>>(), HashSet::from([PathBuf::from("C:\\old")]));
        assert_eq!(watch.into_iter().collect::<HashSet<_>>(), HashSet::from([PathBuf::from("C:\\new")]));
    }

    #[test]
    fn failed_watch_is_not_committed() {
        let current = HashSet::from([PathBuf::from("C:\\keep"), PathBuf::from("C:\\old")]);
        let after = dirs_after_apply(
            &current,
            &[PathBuf::from("C:\\old")],
            &[],
        );
        assert_eq!(after, HashSet::from([PathBuf::from("C:\\keep")]));
    }

    #[test]
    fn successful_watch_is_committed() {
        let current = HashSet::from([PathBuf::from("C:\\keep")]);
        let after = dirs_after_apply(
            &current,
            &[],
            &[PathBuf::from("C:\\new")],
        );
        assert_eq!(
            after,
            HashSet::from([PathBuf::from("C:\\keep"), PathBuf::from("C:\\new")])
        );
    }

    #[test]
    fn debounce_allows_emit_after_window() {
        let mut watched = WatchedState {
            files: HashSet::new(),
            dirs: HashSet::new(),
            last_emit: HashMap::new(),
        };
        let path = Path::new("C:\\notes\\a.md");
        assert!(should_emit(&mut watched, path));
        std::thread::sleep(DEBOUNCE + Duration::from_millis(20));
        assert!(should_emit(&mut watched, path));
    }
}
