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
use crate::services::{exec, hyprland};

const TOUR_BIN: &str = "breadhelp-tour";

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(std::env::temp_dir)
}

pub fn pid_path() -> PathBuf {
    runtime_dir().join("breadhelp-tour.pid")
}

/// Called by the tour when it puts a tour on screen.
pub fn write_pid() {
    if let Err(e) = std::fs::write(pid_path(), std::process::id().to_string()) {
        eprintln!("breadhelp-tour: couldn't write {}: {e}", pid_path().display());
    }
}

/// Called by the tour when the tour ends. Only removes a file that still
/// names this process, so a newer tour's file is left alone.
pub fn remove_pid() {
    let path = pid_path();
    if read_pid(&path) == Some(std::process::id()) {
        let _ = std::fs::remove_file(path);
    }
}

fn read_pid(path: &std::path::Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Whether a tour process is alive. A stale file (crash, reboot of the
/// runtime dir is fine) is detected by checking the pid is still the tour.
pub fn tour_running() -> bool {
    let Some(pid) = read_pid(&pid_path()) else {
        return false;
    };
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|comm| comm.trim().starts_with(TOUR_BIN))
        .unwrap_or(false)
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
    if let Some((_key, original)) = cfg.pending_rebind() {
        if let Err(e) = hyprland::rebind_temp(&original) {
            eprintln!("breadhelp: couldn't revert the tour's temporary rebind: {e}");
        }
    }
    cfg.clear_pending_rebind();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pid_file_for_another_program_is_not_a_running_tour() {
        let dir = std::env::temp_dir().join(format!("breadhelp-tourctl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("breadhelp-tour.pid");
        // This test process is not named breadhelp-tour.
        std::fs::write(&path, std::process::id().to_string()).unwrap();
        let pid = read_pid(&path).unwrap();
        let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).unwrap();
        assert!(!comm.trim().starts_with(TOUR_BIN));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
