//! Bridges `bread-theme`'s wallpaper-derived palette into the webview as CSS
//! custom properties and keeps it live, the same way bos-settings does
//! (`bos-settings/src/src/commands/theme.rs`).
//!
//! `bread-theme` rewrites its files with a temp-then-rename, which kills a
//! direct file watch (inotify reports `DELETE_SELF` and never re-arms), so
//! this watches the generated-theme *directory* and filters by path.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub fn get_theme_css() -> String {
    render(&bread_theme::load_palette())
}

fn render(palette: &bread_theme::Palette) -> String {
    format!(
        "{}\n{}\n:root {{ --font-family: 'Varela Round', sans-serif; --font-display: 'Outfit', 'Varela Round', sans-serif; }}",
        bread_theme::css_custom_properties(palette),
        bread_theme::css_tokens()
    )
}

struct WatcherHandle(#[allow(dead_code)] RecommendedWatcher);

/// Emit `theme-changed` with fresh CSS whenever the shared stylesheet or a
/// per-output palette is rewritten. Call once from `setup`.
pub fn watch_and_emit(app: &AppHandle) {
    let shared = bread_theme::shared_css_path();
    let Some(dir) = shared.parent().map(Path::to_path_buf) else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let handle = app.clone();
    let watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            let Ok(event) = res else { return };
            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
                return;
            }
            let relevant = event.paths.iter().any(|p| {
                p == &shared || matches!(p.extension().and_then(|e| e.to_str()), Some("css") | Some("json"))
            });
            if relevant {
                let _ = handle.emit("theme-changed", get_theme_css());
            }
        },
        notify::Config::default(),
    );
    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            eprintln!("breadhelp: theme watcher: {e}");
            return;
        }
    };
    if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
        eprintln!("breadhelp: theme watcher: couldn't watch {}: {e}", dir.display());
        return;
    }
    app.manage(WatcherHandle(watcher));
}
