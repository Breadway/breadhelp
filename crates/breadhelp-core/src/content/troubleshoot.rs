//! Symptom trees for Fix a problem: one TOML file per symptom, each a small
//! question tree ending in a fix.

use std::path::Path;

#[derive(serde::Deserialize, Clone)]
pub struct SymptomOption {
    pub label: String,
    /// Another node id, or "<file-stem>:<id>" to jump to a different file.
    pub goto: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Fix {
    pub description: String,
    pub command: String,
    /// Fix a problem asks before running a fix with this set.
    #[serde(default)]
    pub requires_confirm: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct SymptomNode {
    pub id: String,
    pub prompt: String,
    #[serde(default)]
    pub options: Vec<SymptomOption>,
    #[serde(default)]
    pub fix: Option<Fix>,
}

#[derive(serde::Deserialize, Clone)]
pub struct SymptomTree {
    /// Card title on Fix a problem ("Sound"). Falls back to the file stem.
    #[serde(default)]
    pub title: String,
    /// One line under the title ("No sound, wrong speakers").
    #[serde(default)]
    pub summary: String,
    #[serde(rename = "node", default)]
    pub nodes: Vec<SymptomNode>,
}

impl SymptomTree {
    pub fn node(&self, id: &str) -> Option<&SymptomNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

const SYMPTOMS_SUBPATH: &str = "troubleshooting/_symptoms";

fn insert_tree(
    stem: &str,
    text: &str,
    origin: &str,
    out: &mut std::collections::BTreeMap<String, SymptomTree>,
) {
    match toml::from_str::<SymptomTree>(text) {
        Ok(mut tree) => {
            if tree.title.is_empty() {
                // Content older than the title field: "no-sound" -> "No sound".
                let words = stem.replace('-', " ");
                let mut c = words.chars();
                tree.title = c
                    .next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default();
            }
            out.insert(stem.to_string(), tree);
        }
        Err(e) => eprintln!("breadhelp: {origin} failed to parse: {e}"),
    }
}

fn scan_symptoms_dir(dir: &Path, out: &mut std::collections::BTreeMap<String, SymptomTree>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        insert_tree(stem, &text, &path.display().to_string(), out);
    }
}

fn scan_symptoms_embedded(
    dir: &'static include_dir::Dir<'static>,
    out: &mut std::collections::BTreeMap<String, SymptomTree>,
) {
    for file in dir.files() {
        if file.path().extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(stem) = file.path().file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(text) = file.contents_utf8() else {
            continue;
        };
        insert_tree(stem, text, &file.path().display().to_string(), out);
    }
}

/// Loads every `*.toml` file in the symptoms directory, keyed by file stem so
/// a `SymptomOption::goto` of `"no-sound:check-mute"` can be resolved. Uses
/// the same precedence as `ContentStore::load`: embedded fallback first, then
/// the system roots in reverse (bakery's `/usr/local` install wins over
/// `/usr/share`), then the user content root last.
pub fn load_all() -> std::collections::BTreeMap<String, SymptomTree> {
    let mut out = std::collections::BTreeMap::new();
    if let Some(dir) = super::embedded_dir(SYMPTOMS_SUBPATH) {
        scan_symptoms_embedded(dir, &mut out);
    }
    for root in super::SYSTEM_ROOTS.iter().rev() {
        scan_symptoms_dir(&Path::new(root).join(SYMPTOMS_SUBPATH), &mut out);
    }
    scan_symptoms_dir(&super::user_content_root().join(SYMPTOMS_SUBPATH), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_trees_exist_without_a_content_checkout() {
        let mut trees = std::collections::BTreeMap::new();
        let dir = super::super::embedded_dir(SYMPTOMS_SUBPATH).expect("embedded symptoms dir");
        scan_symptoms_embedded(dir, &mut trees);
        assert!(trees.len() >= 2);
        for (stem, tree) in &trees {
            assert!(tree.node("start").is_some(), "{stem} has no start node");
        }
    }

    #[test]
    fn shipped_trees_parse_and_every_goto_resolves() {
        let mut trees = std::collections::BTreeMap::new();
        scan_symptoms_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/troubleshooting/_symptoms"),
            &mut trees,
        );
        assert!(trees.len() >= 2);
        for (stem, tree) in &trees {
            assert!(!tree.summary.is_empty(), "{stem} has no summary");
            assert!(tree.node("start").is_some(), "{stem} has no start node");
            for node in &tree.nodes {
                for opt in &node.options {
                    assert!(
                        tree.node(&opt.goto).is_some(),
                        "{stem}:{} goes to missing {}",
                        node.id,
                        opt.goto
                    );
                }
                assert!(
                    node.fix.is_some() || !node.options.is_empty(),
                    "{stem}:{} is a dead end",
                    node.id
                );
            }
        }
    }
}
