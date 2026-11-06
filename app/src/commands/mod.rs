pub mod actions;
pub mod content;
pub mod setup;
pub mod theme;
pub mod window;

use std::time::{Duration, Instant};

use serde::Serialize;

use breadhelp_core::tourctl;

/// How long the help window keeps itself on screen waiting for the tour
/// process to announce it is up. Long enough for a cold GTK4 start, short
/// enough that a broken install reports a reason instead of hanging.
pub const TOUR_START_TIMEOUT: Duration = Duration::from_secs(4);

/// What a "start the tour" command tells the page. The page shows `message`
/// while the command is pending and keeps it on screen when `ok` is false,
/// so the window never closes silently into an empty desktop.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TourOutcome {
    pub ok: bool,
    pub message: String,
    pub tour_started: bool,
}

/// Block (on a worker thread) until `pid` names a live process, or the
/// timeout expires.
pub fn wait_for_tour_until(pid: &breadhelp_core::services::pidfile::PidFile, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if pid.alive() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Block (on a worker thread) until the tour pid file names a live tour
/// process, or [`TOUR_START_TIMEOUT`] expires.
pub fn wait_for_tour() -> bool {
    wait_for_tour_until(&tourctl::tour_pid(), TOUR_START_TIMEOUT)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("breadhelp-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_missing_tour_pid_times_out_quickly() {
        let dir = scratch("tour-wait-missing");
        let pid = breadhelp_core::services::pidfile::PidFile::new_unchecked(dir.join("tour.pid"), "breadhelp-tour");
        assert!(!wait_for_tour_until(&pid, Duration::from_millis(120)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_live_process_named_breadhelp_tour_is_seen() {
        let dir = scratch("tour-wait-live");
        // comm is the executable's file name, so a copy of sleep under the
        // tour's name is what PidFile::alive checks against.
        let exe = dir.join("breadhelp-tour");
        std::fs::copy("/bin/sleep", &exe).unwrap();
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut child = Command::new(&exe).arg("5").spawn().unwrap();
        let pid = breadhelp_core::services::pidfile::PidFile::new_unchecked(dir.join("tour.pid"), "breadhelp-tour");
        std::fs::write(pid.path(), child.id().to_string()).unwrap();
        assert!(wait_for_tour_until(&pid, Duration::from_secs(2)));
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
