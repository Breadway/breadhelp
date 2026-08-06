//! Symptom-tree loader for the troubleshooting wizard (UI lands in M5;
//! this loader is defined now alongside the rest of the content system since
//! it's the same shape of problem as `keybinds.rs`/`meta.rs`).

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
    /// Not read by `ui::troubleshoot_wizard` yet — the "Run fix" button
    /// currently executes unconditionally regardless of this flag. Real
    /// symptom content already sets it, so kept rather than dropped; the
    /// wizard should gate on it before this is treated as safe.
    #[allow(dead_code)]
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

#[derive(serde::Deserialize)]
struct SymptomFile {
    #[serde(rename = "node", default)]
    nodes: Vec<SymptomNode>,
}

const SYSTEM_SYMPTOMS_DIR: &str = "/usr/share/breadhelp/content/troubleshooting/_symptoms";
const SYMPTOMS_SUBPATH: &str = "troubleshooting/_symptoms";

fn scan_symptoms_dir(dir: &Path, out: &mut std::collections::HashMap<String, Vec<SymptomNode>>) {
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
        match toml::from_str::<SymptomFile>(&text) {
            Ok(f) => {
                out.insert(stem.to_string(), f.nodes);
            }
            Err(e) => eprintln!("breadhelp: {} failed to parse: {e}", path.display()),
        }
    }
}

/// Loads every `*.toml` file in the symptoms directory, keyed by file stem so
/// a `SymptomOption::goto` of `"no-sound:check-mute"` can be resolved. Scans
/// the system path (a pacman package's location) then the user content root
/// (where bakery installs content — see `content::user_content_root`) —
/// same "user copy wins on collision" precedent as `ContentStore::load`.
pub fn load_all() -> std::collections::HashMap<String, Vec<SymptomNode>> {
    let mut out = std::collections::HashMap::new();
    scan_symptoms_dir(Path::new(SYSTEM_SYMPTOMS_DIR), &mut out);
    scan_symptoms_dir(&super::user_content_root().join(SYMPTOMS_SUBPATH), &mut out);
    out
}
