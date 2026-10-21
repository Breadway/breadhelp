//! Bread bus integration. `--suggest` banners stay local; `bread.help.*`
//! emits are optional and fail-silent. See `EVENTS.md`.
//!
//! `BreadClient::emit` never blocks or errors this process: a missing or
//! restarting breadd must not affect the help center itself.

use bread_utils::bread_client::BreadClient;

/// Sibling-app id in `bread_shared::apps::KNOWN_APPS`. Events publish as
/// `bread.help.*`. Command verbs are handled by `breadhelp listen`.
pub const APP_ID: &str = "help";

/// Fire-and-forget `bread.help.opened` after the main window is actually
/// shown. Silent autostart (window built, not presented) does not call this.
pub fn emit_opened(autostart: bool) {
    BreadClient::connect(APP_ID).emit(
        "bread.help.opened",
        serde_json::json!({ "autostart": autostart }),
    );
}

/// Fire-and-forget `bread.help.hidden` when the help window leaves the
/// screen. `resident` is true when the process stays loaded (hidden) and
/// false when it is quitting, so a memory manager knows whether there is
/// anything left to unload.
pub fn emit_hidden(resident: bool) {
    BreadClient::connect(APP_ID).emit("bread.help.hidden", serde_json::json!({ "resident": resident }));
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
