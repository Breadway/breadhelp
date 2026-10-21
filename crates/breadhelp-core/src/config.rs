//! `~/.config/breadhelp/state.toml`: non-destructive TOML editing via
//! `bread_utils::tomlcfg`, the same `load_doc`/`save_doc` discipline
//! `bos-settings/src/config/mod.rs` uses: a missing file yields defaults, a
//! file that exists but fails to parse is backed up once before falling
//! back, so a bad edit is always recoverable. This module used to carry its
//! own byte-for-byte copy of that logic (introduced in the same fix pass
//! that added it to bos-settings); now both share one implementation.

use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut};

pub fn config_dir() -> PathBuf {
    bread_utils::xdg::config_home()
}

fn state_path() -> PathBuf {
    config_dir().join("breadhelp").join("state.toml")
}

fn load_doc(path: &Path) -> DocumentMut {
    bread_utils::tomlcfg::load_doc("breadhelp", path)
}

fn save_doc(path: &Path, doc: &DocumentMut) {
    if let Err(e) = bread_utils::tomlcfg::save_doc(path, doc) {
        eprintln!("breadhelp: couldn't write {}: {e}", path.display());
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Normal,
    Beginner,
    Dad,
    Compact,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Normal => "normal",
            Mode::Beginner => "beginner",
            Mode::Dad => "dad",
            Mode::Compact => "compact",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "beginner" => Mode::Beginner,
            "dad" => Mode::Dad,
            "compact" => Mode::Compact,
            _ => Mode::Normal,
        }
    }

    /// Beginner/Dad pick `content.beginner.md` over `content.md` when present.
    pub fn is_simplified(self) -> bool {
        matches!(self, Mode::Beginner | Mode::Dad)
    }
}

pub struct State {
    doc: DocumentMut,
    path: PathBuf,
}

impl State {
    pub fn load() -> Self {
        let path = state_path();
        let doc = load_doc(&path);
        Self { doc, path }
    }

    pub fn onboarding_completed(&self) -> bool {
        self.doc
            .get("onboarding")
            .and_then(|t| t.get("completed"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set_onboarding_completed(&mut self, completed: bool) {
        self.doc["onboarding"]["completed"] = value(completed);
        self.save();
    }

    pub fn onboarding_step(&self) -> i64 {
        self.doc
            .get("onboarding")
            .and_then(|t| t.get("step"))
            .and_then(|v| v.as_integer())
            .unwrap_or(0)
    }

    pub fn set_onboarding_step(&mut self, step: i64) {
        self.doc["onboarding"]["step"] = value(step);
        self.save();
    }

    /// First-boot day-zero wizard (timezone / network hint / updates /
    /// snapshot). Independent of the live desktop tour so an existing user
    /// who already finished onboarding is never pulled back into setup.
    pub fn day_zero_completed(&self) -> bool {
        self.doc
            .get("day_zero")
            .and_then(|t| t.get("completed"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set_day_zero_completed(&mut self, completed: bool) {
        self.doc["day_zero"]["completed"] = value(completed);
        self.save();
    }

    pub fn day_zero_step(&self) -> i64 {
        self.doc
            .get("day_zero")
            .and_then(|t| t.get("step"))
            .and_then(|v| v.as_integer())
            .unwrap_or(0)
    }

    pub fn set_day_zero_step(&mut self, step: i64) {
        self.doc["day_zero"]["step"] = value(step);
        self.save();
    }

    pub fn day_zero_snapshot_attempted(&self) -> bool {
        self.doc
            .get("day_zero")
            .and_then(|t| t.get("snapshot_attempted"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set_day_zero_snapshot_attempted(&mut self, attempted: bool) {
        self.doc["day_zero"]["snapshot_attempted"] = value(attempted);
        self.save();
    }

    /// Set the instant before `services::hyprland::rebind_temp` is called for
    /// a tour step with no compositor-observable signal (e.g. the screenshot
    /// step), cleared the instant after reverting. If breadhelp is killed
    /// mid-step, this is how the next launch knows a keybind was left
    /// pointing at a chained `--tour-event` ping and self-heals it before
    /// the user can be surprised by a stray tour popup; see `ui::tour`.
    pub fn pending_rebind(&self) -> Option<(String, String)> {
        let key = self
            .doc
            .get("tour")?
            .get("pending_rebind_key")?
            .as_str()?
            .to_string();
        let original = self
            .doc
            .get("tour")?
            .get("pending_rebind_original")?
            .as_str()?
            .to_string();
        Some((key, original))
    }

    pub fn set_pending_rebind(&mut self, key: &str, original_bind_value: &str) {
        self.doc["tour"]["pending_rebind_key"] = value(key);
        self.doc["tour"]["pending_rebind_original"] = value(original_bind_value);
        self.save();
    }

    pub fn clear_pending_rebind(&mut self) {
        if let Some(table) = self.doc.get_mut("tour").and_then(|t| t.as_table_mut()) {
            table.remove("pending_rebind_key");
            table.remove("pending_rebind_original");
        }
        self.save();
    }

    pub fn mode(&self) -> Mode {
        let s = self
            .doc
            .get("general")
            .and_then(|t| t.get("mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        Mode::from_str(s)
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.doc["general"]["mode"] = value(mode.as_str());
        self.save();
    }

    /// Keep the help window loaded (hidden) after it is closed, so the next
    /// Super + / paints in about 170 ms instead of about 620 ms, at the cost
    /// of about 200 MiB of webview memory for the session. Off by default.
    pub fn resident(&self) -> bool {
        self.doc
            .get("general")
            .and_then(|t| t.get("resident"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn set_resident(&mut self, resident: bool) {
        self.doc["general"]["resident"] = value(resident);
        self.save();
    }

    fn save(&self) {
        save_doc(&self.path, &self.doc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_toml_write_backs_up_previous_contents_and_no_tmp_file_left_behind() {
        // Exercises the same bread_utils::atomic::write_atomic_backed_up
        // path save_doc uses (via tomlcfg::save_doc), at the file-layout
        // breadhelp actually writes to: the deeper backup/no-leftover-tmp
        // behavior itself is covered by bread-utils' own test suite.
        let dir = std::env::temp_dir().join(format!(
            "breadhelp-atomic-write-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.toml");
        let backup = dir.join("state.toml.bak");

        bread_utils::atomic::write_atomic_backed_up(&path, "first").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first");
        assert!(
            !backup.exists(),
            "no backup should be made when there's nothing to back up yet"
        );

        bread_utils::atomic::write_atomic_backed_up(&path, "second").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "first");

        let leftover_tmp: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp."))
            .collect();
        assert!(
            leftover_tmp.is_empty(),
            "temp file should be renamed away, not left behind: {leftover_tmp:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_doc_then_load_doc_round_trips_state() {
        let dir = std::env::temp_dir().join(format!(
            "breadhelp-state-roundtrip-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.toml");

        let mut doc = DocumentMut::default();
        doc["onboarding"]["completed"] = value(true);
        doc["general"]["mode"] = value("dad");
        save_doc(&path, &doc);

        let loaded = load_doc(&path);
        assert_eq!(
            loaded
                .get("onboarding")
                .and_then(|t| t.get("completed"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            loaded
                .get("general")
                .and_then(|t| t.get("mode"))
                .and_then(|v| v.as_str()),
            Some("dad")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn day_zero_defaults_incomplete_and_round_trips() {
        let dir =
            std::env::temp_dir().join(format!("breadhelp-day-zero-state-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.toml");

        let mut state = State {
            doc: DocumentMut::default(),
            path: path.clone(),
        };
        assert!(!state.day_zero_completed());
        assert_eq!(state.day_zero_step(), 0);
        assert!(!state.day_zero_snapshot_attempted());

        assert!(!state.resident(), "resident is opt-in");
        state.set_resident(true);
        state.set_day_zero_step(3);
        state.set_day_zero_snapshot_attempted(true);
        state.set_day_zero_completed(true);

        let loaded = State {
            doc: load_doc(&path),
            path,
        };
        assert!(loaded.day_zero_completed());
        assert_eq!(loaded.day_zero_step(), 3);
        assert!(loaded.day_zero_snapshot_attempted());
        assert!(loaded.resident());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
