//! How the help window and the tour overlay find each other.
//!
//! The tour is its own binary (`breadhelp-tour`, see the workspace manifest
//! for why). It holds a pid file in `$XDG_RUNTIME_DIR` while a tour is on
//! screen. `breadhelp-tour.lua` forwards *every* window, workspace and layer
//! event as `breadhelp --tour-event <id>`, so `breadhelp` checks that file
//! and only hands an event on when a tour is actually running; otherwise it
//! exits before starting a window, which keeps those calls cheap.

use std::path::PathBuf;
use std::process::Command;

use crate::config::State;
use crate::services::pidfile::PidFile;
use crate::services::{exec, hyprland};

const TOUR_BIN: &str = "breadhelp-tour";

/// Held by the tour process while a tour is on screen.
pub fn tour_pid() -> PidFile {
    PidFile::named("breadhelp-tour", TOUR_BIN)
}

/// Whether a tour is on screen right now.
pub fn tour_running() -> bool {
    tour_pid().alive()
}

/// The tour binary: next to the running executable when installed side by
/// side (bakery puts both in the same bin dir), else whatever is on PATH.
pub fn tour_binary() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(TOUR_BIN)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(TOUR_BIN))
}

/// Start (or forward to) the tour process with `args`.
pub fn spawn_tour(args: &[&str]) {
    let mut cmd = Command::new(tour_binary());
    cmd.args(args);
    if let Err(e) = exec::spawn_detached(cmd) {
        eprintln!("breadhelp: couldn't start {TOUR_BIN}: {e}");
    }
}

/// Reverts a keybind a tour step left temporarily rebound (see
/// `State::pending_rebind`). Safe to call any time, including when nothing
/// is pending; both binaries call it on start so a crashed tour can never
/// leave a stray chained bind behind.
pub fn revert_pending_rebind() {
    let mut cfg = State::load();
    // Nothing pending is the common case: don't rewrite state.toml (and its
    // .bak) on every launch for no change.
    let Some((_key, original)) = cfg.pending_rebind() else {
        return;
    };
    if let Err(e) = hyprland::rebind_temp(&original) {
        eprintln!("breadhelp: couldn't revert the tour's temporary rebind: {e}");
    }
    cfg.clear_pending_rebind();
}
