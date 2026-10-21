//! Guide content: `/usr/share/breadhelp/content/` (system, shipped by the
//! package) merged with `~/.local/share/breadhelp/content/` (user overrides/
//! additions). A guide directory name colliding between the two roots means
//! the user copy wins wholesale: no field-level merge, same "replace the
//! whole doc" discipline `bos-settings/src/config/mod.rs` uses for TOML, just
//! at the file-tree level.

pub mod keybinds;
pub mod keys;
pub mod markdown;
pub mod meta;
pub mod tour;
pub mod troubleshoot;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use meta::GuideMeta;

const SYSTEM_ROOT: &str = "/usr/share/breadhelp/content";

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

pub struct Guide {
    pub id: String,
    pub category: String,
    pub meta: GuideMeta,
    pub dir: PathBuf,
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
                eprintln!("breadhelp: {} failed to parse, skipping", meta_path.display());
                continue;
            };
            let effective_category = meta.category.clone().unwrap_or_else(|| category.clone());
            out.insert(
                (effective_category.clone(), id.clone()),
                Guide { id, category: effective_category, meta, dir: guide_dir },
            );
        }
    }
}

fn category_rank(category: &str) -> usize {
    CATEGORY_ORDER.iter().position(|c| *c == category).unwrap_or(CATEGORY_ORDER.len())
}

impl ContentStore {
    pub fn load() -> Self {
        let mut by_key = BTreeMap::new();
        scan_root(Path::new(SYSTEM_ROOT), &mut by_key);
        scan_root(&user_content_root(), &mut by_key);

        let mut guides: Vec<Guide> = by_key.into_values().collect();
        guides.sort_by(|a, b| {
            (category_rank(&a.category), &a.category, &a.id).cmp(&(category_rank(&b.category), &b.category, &b.id))
        });
        Self { guides }
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
        self.guides.iter().filter(|g| g.category == category).collect()
    }

    pub fn guide(&self, category: &str, id: &str) -> Option<&Guide> {
        self.guides.iter().find(|g| g.category == category && g.id == id)
    }

    pub fn all(&self) -> &[Guide] {
        &self.guides
    }

    /// The guide body for the given mode: `content.beginner.md` when in a
    /// simplified mode and present, otherwise `content.md`. Never errors: a
    /// missing file just renders as an empty guide rather than crashing.
    pub fn body(&self, guide: &Guide, simplified: bool) -> String {
        if simplified {
            let beginner = guide.dir.join("content.beginner.md");
            if let Ok(text) = std::fs::read_to_string(&beginner) {
                return text;
            }
        }
        std::fs::read_to_string(guide.dir.join("content.md")).unwrap_or_default()
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
            let body = std::fs::read_to_string(g.dir.join("content.md"))
                .unwrap_or_else(|_| panic!("{cat}/{id} has no content.md"));
            assert!(!markdown::parse(&body).is_empty(), "{cat}/{id} parses to nothing");
            assert!(!g.meta.title.trim().is_empty(), "{cat}/{id} has no title");
        }
    }

    #[test]
    fn related_guides_resolve() {
        let guides = shipped();
        let ids: Vec<&str> = guides.keys().map(|(_, id)| id.as_str()).collect();
        for ((cat, id), g) in &guides {
            for rel in &g.meta.related {
                assert!(ids.contains(&rel.as_str()), "{cat}/{id} relates to missing guide '{rel}'");
            }
        }
    }

    #[test]
    fn the_breadbug_guides_match_shipped_behavior_and_track_planned_work() {
        let guides = shipped();
        let report = guides.get(&("apps".into(), "02-reporting-a-problem".into())).expect("how-to guide");
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let report_body = std::fs::read_to_string(report.dir.join("content.md")).unwrap();
        let privacy_body = std::fs::read_to_string(privacy.dir.join("content.md")).unwrap();

        // The reporter is terminal-only, so the guide must launch it through
        // the BOS terminal, never as a bare shell run.
        assert!(report_body.contains("[Terminal]breadbug|Report a problem[/Terminal]"));
        assert!(!report_body.contains("[Run]breadbug"));
        // Executable today: list, preview, save.
        for flow in ["breadbug list", "--dry-run", "--save"] {
            assert!(report_body.contains(flow), "reporting guide lacks shipped flow: {flow}");
        }
        // Unmerged bread work is labeled planned, with the branch gates named.
        assert!(report_body.contains("## Planned"));
        for branch in ["feature/breadbug-upload", "feature/breadbug-status", "feature/breadbug-gui"] {
            assert!(report_body.contains(branch), "reporting guide lacks gate: {branch}");
        }
        // Server-side commands must not read as runnable outside Planned.
        let (shipped_text, planned_text) =
            report_body.split_once("## Planned").expect("planned section");
        for cmd in ["breadbug status", "breadbug delete", "breadbug queue"] {
            assert!(
                !shipped_text.contains(cmd),
                "reporting guide presents {cmd} as shipped"
            );
            assert!(planned_text.contains(cmd), "planned section lacks {cmd}");
        }

        // Privacy matches the shipped schema sections and states the limits.
        for section in [
            "journal",
            "runtime",
            "markers",
            "system",
            "installed",
            "embeddings",
            "redactions",
            "90 days",
            "backup",
            "cluster metadata",
        ] {
            assert!(
                privacy_body.to_lowercase().contains(&section.to_lowercase()),
                "privacy guide lacks: {section}"
            );
        }
        // No rotation promise: the rotating install id is unmerged.
        assert!(!privacy_body.contains("30 days"));
        assert!(privacy_body.contains("no such promise applies"));
        // Scrub limits: best effort, journal direct-read caveat.
        assert!(privacy_body.contains("best effort"));
        assert!(privacy_body.contains("journalctl"));
    }

    #[test]
    fn the_privacy_guide_quotes_the_shared_consent_source() {
        let root = repo_root();
        let shared = std::fs::read_to_string(root.join("content/shared/breadbug-consent.md"))
            .expect("shared consent source");
        let (_, notice) = shared.split_once("## Proposed notice").expect("notice section");
        let lines: Vec<&str> = notice
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        assert!(lines.len() > 5, "shared notice is unexpectedly short");
        // Provenance: the file records which unmerged branch it came from.
        assert!(shared.contains("feature/breadbug-consent"));
        assert!(shared.contains("NOT merged"));

        let mut guides = BTreeMap::new();
        scan_root(&root.join("content"), &mut guides);
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let privacy_body = std::fs::read_to_string(privacy.dir.join("content.md")).unwrap();
        for line in &lines {
            assert!(privacy_body.contains(line), "privacy guide drifts from shared source: {line}");
        }
    }
}
