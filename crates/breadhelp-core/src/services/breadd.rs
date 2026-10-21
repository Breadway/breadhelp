//! Bread bus integration. `--suggest` banners stay local; `bread.help.*`
//! emits are optional and fail-silent. See `EVENTS.md`.
//!
//! Every emit runs on its own short-lived thread, never the caller's. The
//! help window calls these from its UI thread, and connecting a
//! `BreadClient` does real work the first time (it also starts breadbug
//! crash reporting); once that work blocked forever (ECO-100) and the window
//! never mapped. A missing, slow or broken breadd must not affect the help
//! center itself.

use std::time::Duration;

use bread_utils::bread_client::BreadClient;

/// How long an emit sent just before the process exits may hold up that
/// exit, so it still reaches breadd.
const FLUSH_BEFORE_EXIT: Duration = Duration::from_millis(300);

/// Emit on a background thread. With `wait`, give it up to that long to
/// finish (for an event sent right before exiting); without, return at once.
fn emit(event: &'static str, data: serde_json::Value, wait: Option<Duration>) {
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let spawned = std::thread::Builder::new().name("bread-emit".into()).spawn(move || {
        BreadClient::connect(APP_ID).emit(event, data);
        let _ = done_tx.send(());
    });
    if let (Ok(_), Some(wait)) = (spawned, wait) {
        let _ = done_rx.recv_timeout(wait);
    }
}

/// Sibling-app id in `bread_shared::apps::KNOWN_APPS`. Events publish as
/// `bread.help.*`. Command verbs are handled by `breadhelp listen`.
pub const APP_ID: &str = "help";

/// Fire-and-forget `bread.help.opened` after the main window is actually
/// shown. Silent autostart (window built, not presented) does not call this.
pub fn emit_opened(autostart: bool) {
    emit("bread.help.opened", serde_json::json!({ "autostart": autostart }), None);
}

/// Fire-and-forget `bread.help.hidden` when the help window leaves the
/// screen. `resident` is true when the process stays loaded (hidden) and
/// false when it is quitting, so a memory manager knows whether there is
/// anything left to unload.
pub fn emit_hidden(resident: bool) {
    // Not resident means the process exits right after this.
    let wait = (!resident).then_some(FLUSH_BEFORE_EXIT);
    emit("bread.help.hidden", serde_json::json!({ "resident": resident }), wait);
}

pub struct Suggestion {
    pub text: String,
}

pub fn resolve(id: &str) -> Option<Suggestion> {
    match id {
        "monitor-setup" => Some(Suggestion {
            text: "New monitor detected: want to set up your display layout?".to_string(),
        }),
        _ => None,
    }
}
