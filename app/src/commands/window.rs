//! Showing, hiding and steering the one help window, and telling the rest
//! of the desktop which of those it is doing.
//!
//! By default closing the window quits the app. With the opt-in
//! `general.resident` setting it hides instead and stays loaded, so the next
//! Super + / is a quick hand-off rather than a cold start. Every show sends
//! the page an `open` event naming where to land, which also makes it
//! refetch keybinds and guides.
//!
//! Presence, for tools like breadram that manage memory: while the process
//! is up it keeps `$XDG_RUNTIME_DIR/breadhelp-window.json` current
//! (`{ "pid", "visible", "resident", "since" }`, `since` in unix seconds of
//! the last show or hide) and publishes `bread.help.opened` /
//! `bread.help.hidden`. `breadhelp --unload` (or the bus command
//! `bread.command.help.unload`) quits a hidden resident window and does
//! nothing to a visible one. See EVENTS.md.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use breadhelp_core::config::State;
use breadhelp_core::services::breadd;
use breadhelp_core::services::pidfile::PidFile;

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Open {
    /// Page id to show (`home`, `keybinds`, `guides`, `fix`, `setup`).
    pub page: String,
    /// The breadd suggestion banner, if one is pending.
    pub suggestion: Option<String>,
}

/// What the page should show on its first paint. Read with a command, not
/// an event, since an event sent before the page's listener is attached is
/// lost.
#[derive(Default)]
pub struct UiState(pub Mutex<Open>);

#[tauri::command]
pub fn get_open_state(state: tauri::State<UiState>) -> Open {
    state.0.lock().map(|o| o.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn dismiss_suggestion(state: tauri::State<UiState>) {
    if let Ok(mut o) = state.0.lock() {
        o.suggestion = None;
    }
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(std::env::temp_dir)
}

/// Held while the help window process is up (visible or hidden).
pub fn window_pid() -> PidFile {
    PidFile::named("breadhelp-window", "breadhelp")
}

fn presence_path() -> PathBuf {
    runtime_dir().join("breadhelp-window.json")
}

fn write_presence(visible: bool) {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let body = serde_json::json!({
        "pid": std::process::id(),
        "visible": visible,
        "resident": State::load().resident(),
        "since": since,
    });
    if let Err(e) = std::fs::write(presence_path(), body.to_string()) {
        eprintln!("breadhelp: couldn't write {}: {e}", presence_path().display());
    }
}

/// Called on start and on exit.
pub fn presence_started() {
    window_pid().write();
    write_presence(false);
}

/// Removes the presence file only if it is this process's, so a stray run
/// (a capture, a stale instance) never deletes the real window's.
pub fn presence_ended() {
    window_pid().remove();
    let path = presence_path();
    let ours = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("pid").and_then(|p| p.as_u64()))
        == Some(u64::from(std::process::id()));
    if ours {
        let _ = std::fs::remove_file(path);
    }
}

/// A suggestion that arrived while no window was up waits here for the
/// next open, rather than keeping a hidden process alive just to hold it.
fn pending_suggestion_path() -> PathBuf {
    runtime_dir().join("breadhelp-suggestion")
}

pub fn save_pending_suggestion(id: &str) {
    let _ = std::fs::write(pending_suggestion_path(), id);
}

/// Moves a saved suggestion into the page's first-paint state.
pub fn take_pending_suggestion(app: &AppHandle) {
    let path = pending_suggestion_path();
    let Ok(id) = std::fs::read_to_string(&path) else {
        return;
    };
    let _ = std::fs::remove_file(path);
    if let Some(s) = breadd::resolve(id.trim()) {
        if let Ok(mut guard) = app.state::<UiState>().0.lock() {
            guard.suggestion = Some(s.text);
        }
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

/// Show the window on `page` and publish `bread.help.opened`.
pub fn show(app: &AppHandle, page: &str, autostart: bool) {
    let open = {
        let state = app.state::<UiState>();
        let mut guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
        guard.page = page.to_string();
        guard.clone()
    };
    let _ = app.emit("open", open);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    write_presence(true);
    breadd::emit_opened(autostart);
}

/// Record a suggestion banner and tell an open page, without showing or
/// focusing anything: this comes from a background daemon event.
pub fn suggest(app: &AppHandle, text: String) {
    let state = app.state::<UiState>();
    if let Ok(mut guard) = state.0.lock() {
        guard.suggestion = Some(text.clone());
    }
    let _ = app.emit("suggestion", text);
}

/// The window is done with for now: hide it when resident, quit otherwise.
pub fn dismiss(app: &AppHandle) {
    if State::load().resident() {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
        write_presence(false);
        breadd::emit_hidden(true);
    } else {
        quit(app);
    }
}

pub fn quit(app: &AppHandle) {
    breadd::emit_hidden(false);
    presence_ended();
    app.exit(0);
}

/// `--unload`: drop a hidden resident window to free its memory. A visible
/// window is someone using it, so it is left alone.
pub fn unload(app: &AppHandle) {
    if !is_visible(app) {
        quit(app);
    }
}

/// Closing the window (its close button, Hyprland's kill bind) goes through
/// [`dismiss`].
pub fn route_close(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            dismiss(&handle);
        }
    });
}

/// Esc, and the page's own "done" paths (starting the tour, finishing setup).
#[tauri::command]
pub fn hide_window(app: AppHandle) {
    dismiss(&app);
}

#[tauri::command]
pub fn get_resident() -> bool {
    State::load().resident()
}

#[tauri::command]
pub fn set_resident(resident: bool) {
    State::load().set_resident(resident);
    write_presence(true);
}
