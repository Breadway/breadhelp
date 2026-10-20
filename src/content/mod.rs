//! Guide content: `/usr/share/breadhelp/content/` (system, shipped by the
//! package) merged with `~/.local/share/breadhelp/content/` (user overrides/
//! additions). A guide directory name colliding between the two roots means
//! the user copy wins wholesale: no field-level merge, same "replace the
//! whole doc" discipline `bos-settings/src/config/mod.rs` uses for TOML, just
//! at the file-tree level.

pub mod keybinds;
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
                    dir: guide_dir,
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

impl ContentStore {
    pub fn load() -> Self {
        let mut by_key = BTreeMap::new();
        scan_root(Path::new(SYSTEM_ROOT), &mut by_key);
        scan_root(&user_content_root(), &mut by_key);

        let mut guides: Vec<Guide> = by_key.into_values().collect();
        guides.sort_by(|a, b| {
            (category_rank(&a.category), &a.category, &a.id).cmp(&(
                category_rank(&b.category),
                &b.category,
                &b.id,
            ))
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

    fn shipped() -> BTreeMap<(String, String), Guide> {
        let mut out = BTreeMap::new();
        scan_root(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("content"),
            &mut out,
        );
        out
    }

    #[test]
    fn every_shipped_guide_loads_and_has_a_body() {
        let guides = shipped();
        assert!(!guides.is_empty());
        for ((cat, id), g) in &guides {
            let body = std::fs::read_to_string(g.dir.join("content.md"))
                .unwrap_or_else(|_| panic!("{cat}/{id} has no content.md"));
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
    fn the_breadbug_guides_match_shipped_behavior_and_track_planned_work() {
        let guides = shipped();
        let report = guides
            .get(&("apps".into(), "02-reporting-a-problem".into()))
            .expect("how-to guide");
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let report_body = std::fs::read_to_string(report.dir.join("content.md")).unwrap();
        let privacy_body = std::fs::read_to_string(privacy.dir.join("content.md")).unwrap();

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
        assert!(!report_body.contains("feature/breadbug-upload"));
        assert!(!report_body.contains("feature/breadbug-status"));

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
        // The install id is random, stored on this machine, and replaced once it is older than 30 days.
        assert!(privacy_body.contains("replaced when it is older than 30 days"));
        assert!(!privacy_body.contains("no such promise applies"));
        // Retention and deletion claims must be the ones the server code makes.
        assert!(privacy_body.contains("BREADBUG_RETENTION_DAYS"));
        assert!(privacy_body.contains("Breadboard issues"));
        // Scrub limits: best effort, journal direct-read caveat.
        assert!(privacy_body.contains("best effort"));
        assert!(privacy_body.contains("journalctl"));
        // Corrected claims: each sentence below was checked against source.
        // Token formats are replaced with <SECRET>; <TOKEN> is only the random-string rule.
        assert!(privacy_body.contains("`<SECRET>`"));
        assert!(!privacy_body.contains("(`<TOKEN>`)"));
        // Local-only embedding and no third-party model calls (card requirement).
        assert!(privacy_body.contains("literal loopback IP address"));
        assert!(privacy_body.contains("makes no calls to third-party model APIs"));
        // Cluster metadata: title, vector and install-hash membership survive a delete.
        assert!(privacy_body.contains("a title taken from report text"));
        assert!(privacy_body.contains("an embedding vector"));
        assert!(privacy_body.contains("BUG-36"));
        // Memory: the guarantee is what is kept or sent, not that memory is never read.
        assert!(
            privacy_body.contains("No memory contents, registers or environment are kept or sent")
        );
        assert!(!privacy_body.contains("It never reads memory"));
        // Queue: nothing retries on its own.
        assert!(privacy_body.contains("Nothing retries a queued report by itself"));
        assert!(!privacy_body.contains("retries with increasing waits"));
        // Daily status poll and the release status section.
        assert!(privacy_body.contains("once a day"));
        assert!(privacy_body.contains("## Release status"));
        // Plain HTTP: the check is a prefix test today; the tracked bug is named.
        assert!(privacy_body.contains("BREAD-55"));
        // Stopped markers are rejected by the server; the tracked bug is named.
        assert!(privacy_body.contains("BREAD-57"));
    }

    #[test]
    fn the_privacy_guide_quotes_the_shared_consent_source() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let shared = std::fs::read_to_string(root.join("content/shared/breadbug-consent.md"))
            .expect("shared consent source");
        let (_, notice) = shared.split_once("## Notice").expect("notice section");
        let lines: Vec<&str> = notice
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        assert!(lines.len() > 5, "shared notice is unexpectedly short");
        // Provenance: the file records that it copies the merged bread constant.
        assert!(shared.contains("CONSENT_TEXT"));
        assert!(shared.contains("bread-bug/src/consent.rs"));
        // Why this test compares against breadhelp's own copy and not against
        // bread's constant: breadhelp cannot depend on bread's Rust crate, so CI
        // has no bread source to read. The shared file is therefore checked
        // against bread by scripts/check-consent-sync.py, which needs a bread
        // checkout and is run by hand whenever the notice text changes. This
        // test then checks that the privacy guide quotes the shared file.

        let mut guides = BTreeMap::new();
        scan_root(&root.join("content"), &mut guides);
        let privacy = guides
            .get(&("advanced".into(), "02-bug-reports-and-your-privacy".into()))
            .expect("privacy guide");
        let privacy_body = std::fs::read_to_string(privacy.dir.join("content.md")).unwrap();
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
}
