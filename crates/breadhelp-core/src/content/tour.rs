//! Live tour step loader: same shape of problem as `troubleshoot.rs`'s
//! symptom trees: TOML data, not hardcoded Rust, so the tour can be edited
//! without a rebuild. Fields are flat optionals rather than a tagged enum in
//! the TOML itself (mirroring `SymptomOption::goto`'s plain-string encoding)
//! to keep hand-authored `onboarding.toml` simple; `target()`/`success()`
//! turn that into the richer shape `ui::tour` actually wants to match on.

#[derive(Clone)]
pub enum Target {
    /// A `gtk4-layer-shell` surface identified by namespace, e.g. "breadbox".
    Namespace(String),
    /// A plain toplevel window identified by its Wayland app-id/class.
    WindowClass(String),
    /// Pure-concept step: full-screen dim, centered callout, no spotlight.
    None,
}

#[derive(Clone)]
pub enum Success {
    /// Auto-advance when a `--tour-event <id>` matching this arrives.
    Event(String),
    /// Auto-advance after this many seconds if no event arrives first.
    Timeout(u64),
    /// Next button only: no event can signal this step's completion.
    Manual,
}

#[derive(serde::Deserialize, Clone)]
pub struct Step {
    /// Not read by any Rust code yet: steps are matched by index, not id.
    /// Real tour content already sets it (content authors use it to keep
    /// track of steps), so kept rather than dropped.
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub target_namespace: Option<String>,
    #[serde(default)]
    pub target_window_class: Option<String>,
    /// Shell command run when the step starts, e.g. launching breadbox so
    /// there's something to highlight. Skipped if the target is already
    /// resolvable; several bread apps toggle-close on a second invocation
    /// of the same command, so firing this unconditionally could close an
    /// already-open instance instead of opening one.
    #[serde(default)]
    pub launch: Option<String>,
    #[serde(default)]
    pub success_event: Option<String>,
    #[serde(default)]
    pub success_timeout_seconds: Option<u64>,
    /// A keybind combo (e.g. "super + shift + s") to temporarily rebind so
    /// its original action still runs but is chained with a
    /// `--tour-event <success_event>` ping: for steps whose action has no
    /// compositor-observable signal at all (e.g. taking a screenshot).
    /// Reverted the moment this step is left, and self-healed on the next
    /// launch if breadhelp crashes mid-step, see `ui::tour`.
    #[serde(default)]
    pub rebind_combo: Option<String>,
    /// The shortcut a must-do step is waiting for (e.g. "super + space"),
    /// shown as keycaps on the callout's "waiting for you" row.
    #[serde(default)]
    pub keybind: Option<String>,
    /// What the callout says once `success_event` confirms the user did it.
    /// Defaults to [`DEFAULT_CONFIRMED_TEXT`].
    #[serde(default)]
    pub confirmed_text: Option<String>,
    /// Keycap reminders listed on the callout (the closing step uses this).
    #[serde(default)]
    pub shortcuts: Vec<Shortcut>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Shortcut {
    pub label: String,
    pub keybind: String,
}

pub const DEFAULT_CONFIRMED_TEXT: &str = "Nice work. That was the real thing, and you just did it.";

impl Step {
    pub fn target(&self) -> Target {
        if let Some(ns) = &self.target_namespace {
            Target::Namespace(ns.clone())
        } else if let Some(class) = &self.target_window_class {
            Target::WindowClass(class.clone())
        } else {
            Target::None
        }
    }

    pub fn success(&self) -> Success {
        if let Some(id) = &self.success_event {
            Success::Event(id.clone())
        } else if let Some(secs) = self.success_timeout_seconds {
            Success::Timeout(secs)
        } else {
            Success::Manual
        }
    }
}

#[derive(serde::Deserialize)]
struct TourFile {
    #[serde(rename = "step", default)]
    steps: Vec<Step>,
}

const TOUR_SUBPATH: &str = "tours/onboarding.toml";

pub fn parse(text: &str) -> Result<Vec<Step>, toml::de::Error> {
    toml::from_str::<TourFile>(text).map(|f| f.steps)
}

/// Resolves the tour through the same precedence as `ContentStore::load`:
/// user root, system roots (including bakery's `/usr/local` prefix), then the
/// copy embedded in the binary. Same "user copy wins" precedent.
pub fn load() -> Vec<Step> {
    let Some(text) = super::content_file(TOUR_SUBPATH) else {
        return Vec::new();
    };
    match parse(&text) {
        Ok(steps) => steps,
        Err(e) => {
            eprintln!("breadhelp: {} failed to parse: {e}", TOUR_SUBPATH);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn embedded_tour_parses_without_a_content_checkout() {
        let text = super::super::embedded_file(TOUR_SUBPATH).expect("embedded onboarding.toml");
        let steps = parse(text).unwrap();
        assert!(!steps.is_empty());
    }

    #[test]
    fn shipped_tour_parses_and_ends_on_the_shortcuts_step() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/tours/onboarding.toml");
        let steps = parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["welcome", "breadbar", "breadbox", "finish"]);
        let launcher = &steps[2];
        assert!(
            matches!(launcher.success(), Success::Event(ref e) if e == "layer-closed:breadbox")
        );
        assert_eq!(launcher.keybind.as_deref(), Some("super + space"));
        let last = steps.last().unwrap();
        assert!(matches!(last.target(), Target::None));
        assert_eq!(last.shortcuts.len(), 3);
    }
}
