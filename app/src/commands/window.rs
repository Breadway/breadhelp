//! Showing, hiding and steering the one help window.
//!
//! The window stays alive (hidden) for the whole session, so Super + / is
//! instant: closing it hides it instead. Every show sends the page an
//! `open` event naming where to land, which also makes it refetch keybinds
//! and guides so edits since the last open show up.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use breadhelp_core::services::breadd;

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

/// Closing the window hides it; the process stays for the session.
pub fn hide_on_close(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = w.hide();
        }
    });
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}
