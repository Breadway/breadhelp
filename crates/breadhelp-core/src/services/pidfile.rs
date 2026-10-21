//! Pid files in `$XDG_RUNTIME_DIR`, for "is that process up right now?"
//! checks that must not start anything: `breadhelp --tour-event` asks about
//! the tour, `breadhelp --suggest` about the help window. A stale file (a
//! crash) is caught by checking the pid still belongs to the named program.

use std::path::PathBuf;

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(std::env::temp_dir)
}

pub struct PidFile {
    path: PathBuf,
    /// `/proc/<pid>/comm` prefix the owner has (comm is cut to 15 bytes).
    comm: &'static str,
}

impl PidFile {
    pub const fn new_unchecked(path: PathBuf, comm: &'static str) -> Self {
        Self { path, comm }
    }

    /// `<runtime dir>/<name>.pid`, owned by a process whose comm starts with `comm`.
    pub fn named(name: &str, comm: &'static str) -> Self {
        Self { path: runtime_dir().join(format!("{name}.pid")), comm }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn write(&self) {
        if let Err(e) = std::fs::write(&self.path, std::process::id().to_string()) {
            eprintln!("breadhelp: couldn't write {}: {e}", self.path.display());
        }
    }

    /// Removes the file only if it still names this process, so a newer
    /// owner's file is left alone.
    pub fn remove(&self) {
        if self.pid() == Some(std::process::id()) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn pid(&self) -> Option<u32> {
        std::fs::read_to_string(&self.path).ok()?.trim().parse().ok()
    }

    pub fn alive(&self) -> bool {
        let Some(pid) = self.pid() else {
            return false;
        };
        std::fs::read_to_string(format!("/proc/{pid}/comm"))
            .map(|comm| comm.trim().starts_with(self.comm))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pid_owned_by_another_program_is_not_alive() {
        let dir = std::env::temp_dir().join(format!("breadhelp-pidfile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // This test process is not named "breadhelp-nope".
        let f = PidFile::new_unchecked(dir.join("x.pid"), "breadhelp-nope");
        assert!(!f.alive());
        f.write();
        assert!(!f.alive());
        f.remove();
        assert!(!f.path().exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
