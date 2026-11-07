//! Guide content, from highest precedence to lowest:
//!
//! 1. `~/.local/share/breadhelp/content/` (user overrides/additions)
//! 2. `/usr/local/share/breadhelp/content/` (bakery system install on BOS)
//! 3. `/usr/share/breadhelp/content/` (manual or distro system install)
//! 4. The `content/` tree compiled into the binary
//!
//! A guide directory name colliding between roots means the higher-precedence
//! copy wins wholesale: no field-level merge, same "replace the whole doc"
//! discipline `bos-settings/src/config/mod.rs` uses for TOML, just at the
//! file-tree level.
//!
//! Bakery installs a system package under its configured prefix's `share`
//! directory. BOS sets `prefix = "/usr/local"`, so the baked content lands at
//! `/usr/local/share/breadhelp/content`, not `/usr/share`. The embedded copy
//! is the fallback for an installed binary whose data archive is missing
//! (HELP-20); it also makes a source-built binary usable without a checkout.

pub mod keybinds;
pub mod keys;
pub mod markdown;
pub mod meta;
pub mod tour;
pub mod troubleshoot;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use include_dir::{include_dir, Dir};

use meta::GuideMeta;

/// System content roots in precedence order. Bakery on BOS uses the
/// `/usr/local` prefix; `/usr/share` is the documented distro/manual path.
const SYSTEM_ROOTS: &[&str] = &[
    "/usr/local/share/breadhelp/content",
    "/usr/share/breadhelp/content",
];

/// The shipped content tree embedded at compile time. Fallback only: an
/// on-disk system or user copy always wins for a colliding guide.
static EMBEDDED_CONTENT: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../content");

/// Categories are just top-level content directories; this only fixes the
/// *display order* of ones we know about; unrecognized directories still
/// show up (sorted alphabetically after these) instead of being hidden.
const CATEGORY_ORDER: &[&str] = &[
    "getting-started",
    "daily-use",
    "customization",
    "apps",
    "troubleshooting",
    "advanced",
];

/// Where a guide's files live: an extracted directory on disk or a directory
/// compiled into the binary.
enum GuideSource {
    Disk(PathBuf),
    Embedded(&'static Dir<'static>),
}

pub struct Guide {
    pub id: String,
    pub category: String,
    pub meta: GuideMeta,
    source: GuideSource,
}

/// `Dir::get_file` searches from the embedded root rather than the receiver,
/// so look a direct child up by filename instead.
fn embedded_child<'a>(dir: &'a Dir<'static>, name: &str) -> Option<&'a include_dir::File<'static>> {
    dir.files()
        .find(|file| file.path().file_name().and_then(|n| n.to_str()) == Some(name))
}

impl Guide {
    /// Whether the guide directory contains `name`, for callers deciding
    /// whether to offer the simplified body or a guide file at all.
    pub fn has_file(&self, name: &str) -> bool {
        match &self.source {
            GuideSource::Disk(dir) => dir.join(name).is_file(),
            GuideSource::Embedded(dir) => embedded_child(dir, name).is_some(),
        }
    }

    /// Read a file from this guide's directory. Returns `None` when the file
    /// is absent or cannot be read as UTF-8.
    pub fn read_file(&self, name: &str) -> Option<String> {
        match &self.source {
            GuideSource::Disk(dir) => std::fs::read_to_string(dir.join(name)).ok(),
            GuideSource::Embedded(dir) => embedded_child(dir, name)
                .and_then(|f| f.contents_utf8())
                .map(str::to_owned),
        }
    }
}

pub struct ContentStore {
    guides: Vec<Guide>,
}

pub(crate) fn user_content_root() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg);
        if p.is_absolute() {
            return p.join("breadhelp/content");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".local/share/breadhelp/content")
}

fn scan_root(root: &Path, out: &mut BTreeMap<(String, String), Guide>) {
    let Ok(categories) = std::fs::read_dir(root) else {
        return;
    };
    for cat_entry in categories.flatten() {
        let cat_path = cat_entry.path();
        if !cat_path.is_dir() {
            continue;
        }
        let category = cat_entry.file_name().to_string_lossy().into_owned();
        let Ok(guide_dirs) = std::fs::read_dir(&cat_path) else {
            continue;
        };
        for guide_entry in guide_dirs.flatten() {
            let guide_dir = guide_entry.path();
            if !guide_dir.is_dir() {
                continue;
            }
            let id = guide_entry.file_name().to_string_lossy().into_owned();
            // troubleshooting/_symptoms holds wizard data, not guides.
            if id == "_symptoms" {
                continue;
            }
            let meta_path = guide_dir.join("meta.toml");
            let Ok(meta_text) = std::fs::read_to_string(&meta_path) else {
                continue;
            };
            let Ok(meta) = toml::from_str::<GuideMeta>(&meta_text) else {
                eprintln!(
                    "breadhelp: {} failed to parse, skipping",
                    meta_path.display()
                );
                continue;
            };
            let effective_category = meta.category.clone().unwrap_or_else(|| category.clone());
            out.insert(
                (effective_category.clone(), id.clone()),
                Guide {
                    id,
                    category: effective_category,
                    meta,
                    source: GuideSource::Disk(guide_dir),
                },
            );
        }
    }
}

fn scan_embedded(root: &'static Dir<'static>, out: &mut BTreeMap<(String, String), Guide>) {
    for cat_dir in root.dirs() {
        let Some(category) = cat_dir.path().file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        for guide_dir in cat_dir.dirs() {
            let Some(id) = guide_dir.path().file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            // troubleshooting/_symptoms holds wizard data, not guides.
            if id == "_symptoms" {
                continue;
            }
            let Some(meta_text) =
                embedded_child(guide_dir, "meta.toml").and_then(|f| f.contents_utf8())
            else {
                continue;
            };
            let Ok(meta) = toml::from_str::<GuideMeta>(meta_text) else {
                eprintln!(
                    "breadhelp: embedded {}/meta.toml failed to parse, skipping",
                    guide_dir.path().display()
                );
                continue;
            };
            let effective_category = meta.category.clone().unwrap_or_else(|| category.to_owned());
            out.insert(
                (effective_category.clone(), id.to_owned()),
                Guide {
                    id: id.to_owned(),
                    category: effective_category,
                    meta,
                    source: GuideSource::Embedded(guide_dir),
                },
            );
        }
    }
}

fn category_rank(category: &str) -> usize {
    CATEGORY_ORDER
        .iter()
        .position(|c| *c == category)
        .unwrap_or(CATEGORY_ORDER.len())
}

/// Build a store from explicit roots. `system_roots` is in precedence order
/// (highest first); `user_root` always wins over them, and any embedded guide
/// not overridden by a filesystem copy is kept.
fn collect_guides(
    system_roots: &[PathBuf],
    user_root: Option<&Path>,
    embedded: Option<&'static Dir<'static>>,
) -> Vec<Guide> {
    let mut by_key = BTreeMap::new();
    if let Some(dir) = embedded {
        scan_embedded(dir, &mut by_key);
    }
    // Insert lowest-precedence system root first so the highest one overwrites.
    for root in system_roots.iter().rev() {
        scan_root(root, &mut by_key);
    }
    if let Some(root) = user_root {
        scan_root(root, &mut by_key);
    }

    let mut guides: Vec<Guide> = by_key.into_values().collect();
    guides.sort_by(|a, b| {
        (category_rank(&a.category), &a.category, &a.id).cmp(&(
            category_rank(&b.category),
            &b.category,
            &b.id,
        ))
    });
    guides
}

/// Resolve a content file such as `tours/onboarding.toml` using the same
/// precedence as [`ContentStore`]: user, then system roots, then embedded.
pub(crate) fn content_file(relative: &str) -> Option<String> {
    let user = user_content_root().join(relative);
    if let Ok(text) = std::fs::read_to_string(&user) {
        return Some(text);
    }
    for root in SYSTEM_ROOTS {
        if let Ok(text) = std::fs::read_to_string(Path::new(root).join(relative)) {
            return Some(text);
        }
    }
    embedded_file(relative).map(str::to_owned)
}

/// A file from the embedded content tree, used as the final fallback.
pub(crate) fn embedded_file(relative: &str) -> Option<&'static str> {
    EMBEDDED_CONTENT
        .get_file(relative)
        .and_then(|f| f.contents_utf8())
}

/// A directory from the embedded content tree, used as the final fallback.
pub(crate) fn embedded_dir(relative: &str) -> Option<&'static Dir<'static>> {
    if relative.is_empty() {
        Some(&EMBEDDED_CONTENT)
    } else {
        EMBEDDED_CONTENT.get_dir(relative)
    }
}

impl ContentStore {
    pub fn load() -> Self {
        let system_roots: Vec<PathBuf> = SYSTEM_ROOTS.iter().map(PathBuf::from).collect();
        let user_root = user_content_root();
        Self {
            guides: collect_guides(&system_roots, Some(&user_root), Some(&EMBEDDED_CONTENT)),
        }
    }

    pub fn categories(&self) -> Vec<String> {
        let mut seen = Vec::new();
        for g in &self.guides {
            if !seen.contains(&g.category) {
                seen.push(g.category.clone());
            }
        }
        seen
    }

    pub fn guides_in(&self, category: &str) -> Vec<&Guide> {
        self.guides
            .iter()
            .filter(|g| g.category == category)
            .collect()
    }

    pub fn guide(&self, category: &str, id: &str) -> Option<&Guide> {
        self.guides
            .iter()
            .find(|g| g.category == category && g.id == id)
    }

    pub fn all(&self) -> &[Guide] {
        &self.guides
    }

    /// The guide body for the given mode: `content.beginner.md` when in a
    /// simplified mode and present, otherwise `content.md`. Never errors: a
    /// missing file just renders as an empty guide rather than crashing.
    pub fn body(&self, guide: &Guide, simplified: bool) -> String {
        if simplified {
            if let Some(text) = guide.read_file("content.beginner.md") {
                return text;
            }
        }
        guide.read_file("content.md").unwrap_or_default()
    }
}

#[cfg(test)]
mod shipped_content_tests {
    use super::*;

    /// The repo root: this crate lives at `crates/breadhelp-core`.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn shipped() -> BTreeMap<(String, String), Guide> {
        let mut out = BTreeMap::new();
        scan_root(&repo_root().join("content"), &mut out);
        out
    }

    #[test]
    fn every_shipped_guide_loads_and_has_a_body() {
        let guides = shipped();
        assert!(!guides.is_empty());
        for ((cat, id), g) in &guides {
            let body = g
                .read_file("content.md")
                .unwrap_or_else(|| panic!("{cat}/{id} has no content.md"));
            assert!(
                !markdown::parse(&body).is_empty(),
                "{cat}/{id} parses to nothing"
            );
            assert!(!g.meta.title.trim().is_empty(), "{cat}/{id} has no title");
        }
    }

    #[test]
    fn related_guides_resolve() {
        let guides = shipped();
        let ids: Vec<&str> = guides.keys().map(|(_, id)| id.as_str()).collect();
        for ((cat, id), g) in &guides {
            for rel in &g.meta.related {
                assert!(
                    ids.contains(&rel.as_str()),
                    "{cat}/{id} relates to missing guide '{rel}'"
                );
            }
        }
    }

    #[test]
    fn the_breadbug_guides_match_shipped_behavior_and_track_pending_work() {
        let guides = shipped();
        let report = guides
            .get(&("apps".into(), "02-reporting-a-problem".into()))
            .expect("how-to guide");
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let report_body = report.read_file("content.md").unwrap();
        let privacy_body = privacy.read_file("content.md").unwrap();

        // The reporter is terminal-only, so the guide must launch it through
        // the BOS terminal, never as a bare shell run.
        assert!(report_body.contains("[Terminal]breadbug|Report a problem[/Terminal]"));
        assert!(!report_body.contains("[Run]breadbug"));
        // Every shipped command the guide teaches must appear in it.
        for flow in [
            "breadbug list",
            "--dry-run",
            "--save",
            "breadbug status",
            "breadbug delete <receipt>",
            "breadbug history",
            "breadbug queue",
            "breadbug config upload on",
        ] {
            assert!(
                report_body.contains(flow),
                "reporting guide lacks shipped flow: {flow}"
            );
        }
        // Nothing in the guide may still call merged work unshipped.
        assert!(!report_body.contains("## Planned"));
        assert!(!report_body.contains("sending is not available"));
        assert!(!report_body.contains("feature/breadbug-upload"));

        // M2: the dev track already ships breadbug, and the daily status call
        // only happens when a receipt is held.
        assert!(report_body.contains("dev track"));
        assert!(report_body.contains("v0.10.0"));
        assert!(report_body.contains("at least one receipt"));

        // Privacy matches the shipped schema sections and states the limits.
        for section in [
            "journal",
            "runtime",
            "markers",
            "system",
            "installed",
            "redactions",
            "90 days",
            "backup",
            "cluster metadata",
            "never collected",
            "Screenshots",
            "Process memory",
            "Core dumps",
            "Sending is off by default",
            "HTTPS",
            "install id",
            "30 days",
        ] {
            assert!(
                privacy_body
                    .to_lowercase()
                    .contains(&section.to_lowercase()),
                "privacy guide lacks: {section}"
            );
        }
        // M1: a freeze without a live stack is titled from the note, not from
        // a crash message or frame.
        assert!(privacy_body.contains("freeze without a live stack"));
        assert!(privacy_body.contains("crash with no frames and no location"));
        assert!(privacy_body.contains("first line of your note"));
        // Stopped markers: the client removes them before building a report,
        // and the server accepts an older client's marker without clustering.
        assert!(privacy_body.contains("removes stopped markers before building the report"));
        assert!(privacy_body.contains("never builds or joins a cluster"));
        assert!(!privacy_body.contains("server rejects it, so that report is dropped"));
        // The provenance sentence appears once.
        assert_eq!(privacy_body.matches("Anything not in place yet").count(), 1);
        // The shortcut is described as dev-track only, matching release status.
        assert!(report_body.contains("not available on a stable BOS image yet"));
        assert!(report_body.contains("On a machine with breadbug from the bakery dev track"));
        assert!(!report_body.contains("On BOS you can also press"));
        assert!(report_body.contains("a stopped marker is not sent"));
        assert!(!report_body.contains("the server does not accept"));
        // Delete now rebuilds or neutralises cluster metadata.
        assert!(privacy_body.contains("neutral title"));
        assert!(privacy_body.contains("What it does not remove"));
        // Local-only embedding and no third-party model calls.
        assert!(privacy_body.contains("private IPv4"));
        assert!(privacy_body.contains("makes no calls to third-party model APIs"));
        // Queue: nothing retries on its own.
        assert!(privacy_body.contains("Nothing retries a queued report by itself"));
        // Stale claims that no longer match the shipped client or server.
        assert!(!privacy_body.contains("existing permissions"));
        assert!(!privacy_body.contains("BREAD-55"));
        assert!(!privacy_body.contains("prefix test"));
        // Scrub limits: best effort, journal direct-read caveat.
        assert!(privacy_body.contains("best effort"));
        assert!(privacy_body.contains("journalctl"));
        // The remaining cross-repo gate is named.
        assert!(privacy_body.contains("BOS site publication"));
    }

    #[test]
    fn the_privacy_guide_quotes_the_shared_consent_source() {
        let root = repo_root();
        let shared = std::fs::read_to_string(root.join("content/shared/breadbug-consent.md"))
            .expect("shared consent source");
        let (_, notice) = shared.split_once("## Notice").expect("notice section");
        let lines: Vec<&str> = notice
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        assert!(lines.len() > 5, "shared notice is unexpectedly short");
        // Provenance: the file names the merged bread constant and the
        // consumers that read it.
        assert!(shared.contains("CONSENT_TEXT"));
        assert!(shared.contains("bread-bug/src/consent.rs"));
        assert!(shared.contains("bos-settings"));
        assert!(!shared.contains("NOT merged"));

        let mut guides = BTreeMap::new();
        scan_root(&root.join("content"), &mut guides);
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let privacy_body = privacy.read_file("content.md").unwrap();
        for line in &lines {
            assert!(
                privacy_body.contains(line),
                "privacy guide drifts from shared source: {line}"
            );
        }
        // The guide's copy must also keep the paragraph order of the shared notice.
        let notice_text = lines.join(" ");
        let start = privacy_body
            .find("Reporting a problem")
            .expect("consent heading in privacy guide");
        let rest = &privacy_body[start..];
        let guide_block = rest
            .split_once("## Not yet shipped")
            .map_or(rest, |(a, _)| a);
        let guide_text = guide_block
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            guide_text.contains(&notice_text),
            "privacy guide consent block differs from shared notice"
        );
    }

    /// A throwaway content root for the precedence tests.
    fn temp_root(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "breadhelp-content-{}-{tag}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_guide(root: &Path, category: &str, id: &str, title: &str, body: &str) {
        let dir = root.join(category).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("meta.toml"), format!("title = \"{title}\"\n")).unwrap();
        std::fs::write(dir.join("content.md"), body).unwrap();
    }

    /// HELP-20: the embedded fallback alone must list the shipped guides and
    /// every one of them must open. This never touches the repo source tree.
    #[test]
    fn embedded_content_lists_every_guide_with_a_body() {
        let guides = collect_guides(&[], None, Some(&EMBEDDED_CONTENT));
        assert!(!guides.is_empty(), "embedded content has no guides");
        for g in &guides {
            let body = g
                .read_file("content.md")
                .unwrap_or_else(|| panic!("{}/{} has no embedded body", g.category, g.id));
            assert!(
                !markdown::parse(&body).is_empty(),
                "{}/{} parses to nothing",
                g.category,
                g.id
            );
            assert!(!g.meta.title.trim().is_empty());
        }
    }

    /// Precedence: user override beats system content, system content beats
    /// the embedded copy, and unoverridden embedded guides still appear.
    #[test]
    fn user_beats_system_and_system_beats_embedded() {
        let system = temp_root("system");
        let user = temp_root("user");
        write_guide(
            &system,
            "getting-started",
            "01-what-is-bos",
            "System title",
            "system body",
        );
        write_guide(
            &system,
            "getting-started",
            "99-system-only",
            "System only",
            "system only body",
        );
        // A system copy of an embedded guide, with no user override.
        write_guide(
            &system,
            "apps",
            "01-notes-and-tasks",
            "System notes",
            "system notes body",
        );
        write_guide(
            &user,
            "getting-started",
            "01-what-is-bos",
            "User title",
            "user body",
        );

        let guides = collect_guides(
            &[system.clone(), PathBuf::from("/nonexistent-breadhelp-root")],
            Some(&user),
            Some(&EMBEDDED_CONTENT),
        );
        let store = ContentStore { guides };

        let overridden = store
            .guide("getting-started", "01-what-is-bos")
            .expect("overridden guide");
        assert_eq!(overridden.meta.title, "User title");
        assert_eq!(store.body(overridden, false), "user body");

        let system_only = store
            .guide("getting-started", "99-system-only")
            .expect("system-only guide");
        assert_eq!(system_only.meta.title, "System only");
        assert_eq!(store.body(system_only, false), "system only body");

        let system_over_embedded = store
            .guide("apps", "01-notes-and-tasks")
            .expect("system copy of an embedded guide");
        assert_eq!(system_over_embedded.meta.title, "System notes");
        assert_eq!(store.body(system_over_embedded, false), "system notes body");

        assert!(
            store
                .guide("advanced", "02-bug-reports-and-your-privacy")
                .is_some(),
            "embedded guides must still fill gaps"
        );

        std::fs::remove_dir_all(&system).ok();
        std::fs::remove_dir_all(&user).ok();
    }
}
