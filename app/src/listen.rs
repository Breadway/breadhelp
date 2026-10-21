//! Long-running command subscription for `bread.command.help.*`.
//!
//! `breadhelp listen` is the optional persistent process that honors bus
//! commands. See `EVENTS.md`.

use bread_utils::bread_client::{BreadClient, BreadEvent};

use breadhelp_core::services::breadd::APP_ID;

/// Subscribe to `bread.command.help.**` and block until the process is killed.
///
/// breadd being absent is not an error: [`BreadClient::subscribe`] reconnects
/// with backoff, and `on_event` simply isn't called until the daemon is up.
pub fn run() {
    let client = BreadClient::connect(APP_ID);
    if client.health().is_none() {
        eprintln!(
            "breadhelp: breadd unreachable; command subscription will connect when it comes back"
        );
    }

    let _commands = client.subscribe("bread.command.help.**", |event| {
        handle_command(&event);
    });

    eprintln!("breadhelp: listening for bread.command.help.**");
    loop {
        std::thread::park();
    }
}

/// Reacts to `bread.command.help.*` verbs: `open` and `unload`;
/// other verbs are ignored, not stubbed as no-ops that pretend to succeed.
fn handle_command(event: &BreadEvent) {
    let Some(verb) = command_verb(&event.event) else {
        return;
    };
    match verb {
        "open" => handle_open(),
        "unload" => handle_unload(),
        other => {
            eprintln!("breadhelp: ignoring unrecognized bread.command.help.{other}");
        }
    }
}

fn handle_open() {
    // Same as launching from the app menu: the single-instance plugin hands
    // this to an already-running help window, else it starts one.
    let result = spawn_self();
    let client = BreadClient::connect(APP_ID);
    match result {
        Ok(_) => client.emit("bread.help.open.done", serde_json::json!({})),
        Err(e) => {
            eprintln!("breadhelp: bread.command.help.open failed: {e}");
            client.emit(
                "bread.help.open.failed",
                serde_json::json!({ "error": e.to_string() }),
            );
        }
    }
}

/// Free a hidden resident window's memory (breadram's lever). A no-op when
/// no window is up or it is on screen; see `commands::window::unload`.
fn handle_unload() {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("breadhelp"));
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--unload");
    let client = BreadClient::connect(APP_ID);
    match breadhelp_core::services::exec::spawn_detached(cmd) {
        Ok(()) => client.emit("bread.help.unload.done", serde_json::json!({})),
        Err(e) => client.emit("bread.help.unload.failed", serde_json::json!({ "error": e.to_string() })),
    }
}

fn spawn_self() -> std::io::Result<()> {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("breadhelp"));
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["--page", "home"]);
    breadhelp_core::services::exec::spawn_detached(cmd)
}

fn command_verb(event_name: &str) -> Option<&str> {
    event_name.strip_prefix("bread.command.help.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_verb_strips_help_prefix() {
        assert_eq!(command_verb("bread.command.help.open"), Some("open"));
        assert_eq!(command_verb("bread.command.help.onboard"), Some("onboard"));
        assert_eq!(command_verb("bread.command.box.open"), None);
        assert_eq!(command_verb("bread.help.opened"), None);
    }
}
