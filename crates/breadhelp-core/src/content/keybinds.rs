//! Loads `~/.config/hypr/binds.json`: the SAME structured data
//! `hyprland.lua` itself loads (via `scripts/input/{binds,keybinds}.lua`) to
//! generate the real Hyprland binds. Reading this directly, rather than a
//! separately-maintained cheatsheet, means the keybind viewer can never
//! drift from what's actually bound.

use super::keys;

#[derive(Clone, Debug)]
pub struct Keybind {
    /// Human combo, lowercase keysym names: `"super + shift + slash"`.
    pub combo: String,
    /// Canonical comparison form, see [`keys::combo_id`].
    pub combo_id: String,
    /// Keycap text, one entry per key: `["Super", "Shift", "/"]`.
    pub caps: Vec<String>,
    /// Canonical modifiers and key, kept for grouping.
    pub mods: Vec<String>,
    pub key: String,
    /// The bind's `label`, or its raw `action` when it has none.
    pub action: String,
    /// binds.json's own action kind (`exec`, `focus`, `move`, ...).
    pub kind: String,
    pub category: String,
    /// What the onboarding tour's "Try it" button runs for this bind.
    pub demo_cmd: Option<String>,
    /// The `<dispatcher>,<args>` half of the `<mods>,<key>,<dispatcher>,<args>`
    /// bind string `services::hyprland::rebind_temp` takes (e.g.
    /// `"exec,breadbox"`), only synthesized for `action: "exec"`
    /// entries, since binds.json's other actions ("close", "fullscreen", ...)
    /// are this app's own loader abstraction, not real Hyprland dispatcher
    /// names, and there's no safe generic way to reconstruct one. Binds
    /// without this simply don't offer a rebind button.
    pub raw: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawEntry {
    action: String,
    #[serde(default)]
    command: Option<String>,
    key: String,
    #[serde(default)]
    mods: Option<Vec<String>>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    category: String,
    #[serde(default)]
    demo_cmd: Option<String>,
}

#[derive(serde::Deserialize)]
struct BindsFile {
    #[serde(default)]
    default_mods: Vec<String>,
    #[serde(default)]
    bindings: Vec<RawEntry>,
}

pub fn binds_json_path() -> std::path::PathBuf {
    bread_utils::xdg::config_home().join("hypr/binds.json")
}

pub fn load() -> Vec<Keybind> {
    let Ok(text) = std::fs::read_to_string(binds_json_path()) else {
        return Vec::new();
    };
    parse(&text)
}

pub fn parse(text: &str) -> Vec<Keybind> {
    let file: BindsFile = match serde_json::from_str(text) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("breadhelp: binds.json failed to parse: {e}");
            return Vec::new();
        }
    };
    let default_mods = if file.default_mods.is_empty() { vec!["SUPER".to_string()] } else { file.default_mods };

    file.bindings
        .into_iter()
        .map(|entry| {
            // An explicit "mods": [] (media keys) must NOT fall back to
            // default_mods: only an omitted `mods` field should, matching
            // scripts/input/keybinds.lua's own normalize_mods semantics.
            let raw_mods = entry.mods.unwrap_or_else(|| default_mods.clone());
            let mods = keys::sort_mods(&raw_mods);
            let key = keys::canonical(&entry.key);
            let combo = if mods.is_empty() { key.clone() } else { format!("{} + {key}", mods.join(" + ")) };
            let raw = (entry.action == "exec").then(|| entry.command.clone()).flatten().map(|c| format!("exec,{c}"));
            Keybind {
                combo_id: keys::combo_id(&mods, &key),
                caps: keys::display_combo(&mods, &key),
                combo,
                mods,
                key,
                action: entry.label.unwrap_or_else(|| entry.action.clone()),
                kind: entry.action,
                category: entry.category,
                demo_cmd: entry.demo_cmd,
                raw,
            }
        })
        .collect()
}

/// Lookup by combo for `[Show Keybind]` tags. Guides write what a person
/// types (`super + ,`) while binds.json stores keysym names (`comma`), so
/// both sides go through [`keys::parse_combo_id`]. `None` (rendered as the
/// guide's own text) rather than a panic when a guide names an unbound combo.
pub fn find<'a>(binds: &'a [Keybind], combo: &str) -> Option<&'a Keybind> {
    let id = keys::parse_combo_id(combo)?;
    binds.iter().find(|b| b.combo_id == id)
}

/// Best-effort conversion of a human combo ("super + shift + u") into the
/// `<mods>_<mods>,<key>` form ("SUPER_SHIFT,U", or ",U" with no modifier)
/// that `services::hyprland::rebind_temp` parses for the temporary rebind
/// feature. Symbols go through [`keys::canonical`] first, so `super + ,`
/// becomes `SUPER,COMMA`.
pub fn to_hypr_mods_key(combo: &str) -> String {
    let parts: Vec<String> = combo
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .flat_map(|s| s.split_whitespace())
        .map(|t| keys::canonical(t).to_uppercase())
        .collect();
    match parts.split_last() {
        None => String::new(),
        Some((key, [])) => format!(",{key}"),
        Some((key, mods)) => format!("{},{}", mods.join("_"), key),
    }
}

/// One row of the keybind reference: a single bind, or a family of related
/// binds shown once (`Super` `1 – 0` for ten workspace binds).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub caps: Vec<String>,
    pub label: String,
    pub category: String,
    /// Indexes into the bind list this row stands for.
    pub members: Vec<usize>,
}

/// Key families that collapse into one row when every member shares the
/// same modifiers, action kind and category.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Digits,
    Arrows,
    Vim,
    Brackets,
    Scroll,
    Volume,
    Brightness,
    Tracks,
}

fn family(key: &str) -> Option<Family> {
    Some(match key {
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0" => Family::Digits,
        "left" | "right" | "up" | "down" => Family::Arrows,
        "h" | "j" | "k" | "l" => Family::Vim,
        "bracketleft" | "bracketright" => Family::Brackets,
        "mouse_up" | "mouse_down" => Family::Scroll,
        "xf86audiolowervolume" | "xf86audioraisevolume" => Family::Volume,
        "xf86monbrightnessdown" | "xf86monbrightnessup" => Family::Brightness,
        "xf86audioprev" | "xf86audionext" => Family::Tracks,
        _ => return None,
    })
}

/// Display order of a key inside its family.
fn family_rank(key: &str) -> usize {
    const ORDER: &[&str] = &[
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "left", "up", "down", "right", "h", "j", "k", "l",
        "bracketleft", "bracketright", "mouse_up", "mouse_down", "xf86audiolowervolume", "xf86audioraisevolume",
        "xf86monbrightnessdown", "xf86monbrightnessup", "xf86audioprev", "xf86audionext",
    ];
    ORDER.iter().position(|k| *k == key).unwrap_or(usize::MAX)
}

/// Collapse binds.json's flat list into reference rows, keeping the file's
/// own order (a family appears where its first member does).
pub fn rows(binds: &[Keybind]) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut used = vec![false; binds.len()];
    for (i, bind) in binds.iter().enumerate() {
        if used[i] {
            continue;
        }
        let Some(fam) = family(&bind.key) else {
            used[i] = true;
            rows.push(Row {
                caps: bind.caps.clone(),
                label: bind.action.clone(),
                category: bind.category.clone(),
                members: vec![i],
            });
            continue;
        };
        let mut members: Vec<usize> = binds
            .iter()
            .enumerate()
            .filter(|(j, b)| {
                !used[*j]
                    && family(&b.key) == Some(fam)
                    && b.mods == bind.mods
                    && b.kind == bind.kind
                    && b.category == bind.category
            })
            .map(|(j, _)| j)
            .collect();
        if members.len() < 2 {
            members = vec![i];
        }
        members.sort_by_key(|j| family_rank(&binds[*j].key));
        for j in &members {
            used[*j] = true;
        }
        rows.push(family_row(binds, fam, members));
    }
    rows
}

fn family_row(binds: &[Keybind], fam: Family, members: Vec<usize>) -> Row {
    let first = &binds[members[0]];
    if members.len() == 1 {
        return Row { caps: first.caps.clone(), label: first.action.clone(), category: first.category.clone(), members };
    }
    let mut caps: Vec<String> = first.mods.iter().map(|m| keys::display(m)).collect();
    let member_keys: Vec<&str> = members.iter().map(|j| binds[*j].key.as_str()).collect();
    match fam {
        Family::Digits => {
            let (a, b) = (member_keys[0], member_keys[member_keys.len() - 1]);
            caps.push(format!("{a} – {b}"));
        }
        Family::Arrows | Family::Vim => {
            caps.push(member_keys.iter().map(|k| keys::display(k)).collect::<Vec<_>>().join(" "));
        }
        Family::Scroll => caps.push("Scroll".to_string()),
        Family::Brackets | Family::Volume | Family::Brightness | Family::Tracks => {
            caps.extend(member_keys.iter().map(|k| keys::display(k)));
        }
    }
    let labels: Vec<&str> = members.iter().map(|j| binds[*j].action.as_str()).collect();
    Row { caps, label: merge_labels(&labels, fam), category: first.category.clone(), members }
}

/// One label for a family, from its members' labels: the shared leading
/// words, plus a range (`1 to 10`) or the differing tails (`up / down`).
fn merge_labels(labels: &[&str], fam: Family) -> String {
    let cleaned: Vec<String> = labels.iter().map(|l| strip_parens(l)).collect();
    let words: Vec<Vec<&str>> = cleaned.iter().map(|l| l.split_whitespace().collect()).collect();
    let shortest = words.iter().map(Vec::len).min().unwrap_or(0);
    let mut common = 0;
    while common < shortest && words.iter().all(|w| w[common].eq_ignore_ascii_case(words[0][common])) {
        common += 1;
    }
    let prefix = words[0][..common].join(" ");
    let tails: Vec<String> = words.iter().map(|w| w[common..].join(" ")).collect();

    if matches!(fam, Family::Arrows | Family::Vim) || tails.iter().all(String::is_empty) {
        return if prefix.is_empty() { cleaned[0].clone() } else { prefix };
    }
    let numeric = tails.iter().all(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()));
    let tail = if numeric {
        format!("{} to {}", tails[0], tails[tails.len() - 1])
    } else {
        tails.join(" / ")
    };
    if prefix.is_empty() {
        tail
    } else {
        format!("{prefix} {tail}")
    }
}

fn strip_parens(label: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in label.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "default_mods": ["SUPER"],
      "bindings": [
        { "key": "SPACE", "action": "exec", "command": "breadbox", "label": "Open the app launcher (breadbox)", "category": "apps", "demo_cmd": "breadbox" },
        { "key": "comma", "action": "exec", "command": "bos-settings", "label": "Open BOS Settings", "category": "apps" },
        { "key": "Escape", "mods": ["SUPER", "SHIFT"], "action": "exit", "label": "Exit Hyprland (log out)", "category": "windows" },
        { "key": "left", "action": "focus", "label": "Move focus left", "category": "focus" },
        { "key": "right", "action": "focus", "label": "Move focus right", "category": "focus" },
        { "key": "up", "action": "focus", "label": "Move focus up", "category": "focus" },
        { "key": "down", "action": "focus", "label": "Move focus down", "category": "focus" },
        { "key": "1", "action": "focus", "label": "Switch to workspace 1", "category": "workspaces" },
        { "key": "2", "action": "focus", "label": "Switch to workspace 2", "category": "workspaces" },
        { "key": "0", "action": "focus", "label": "Switch to workspace 10", "category": "workspaces" },
        { "key": "bracketright", "action": "focus", "label": "Next workspace", "category": "workspaces" },
        { "key": "bracketleft", "action": "focus", "label": "Previous workspace", "category": "workspaces" },
        { "key": "XF86AudioRaiseVolume", "mods": [], "action": "exec", "command": "wpctl up", "label": "Volume up", "category": "media" },
        { "key": "XF86AudioLowerVolume", "mods": [], "action": "exec", "command": "wpctl down", "label": "Volume down", "category": "media" },
        { "key": "XF86AudioMute", "mods": [], "action": "exec", "command": "wpctl mute", "label": "Mute", "category": "media" }
      ]
    }"#;

    #[test]
    fn find_matches_symbols_against_keysym_names() {
        let binds = parse(SAMPLE);
        assert_eq!(find(&binds, "super + ,").map(|b| b.action.as_str()), Some("Open BOS Settings"));
        assert_eq!(find(&binds, "super + space").map(|b| b.combo_id.as_str()), Some("super+space"));
        assert_eq!(find(&binds, "super + shift + esc").map(|b| b.kind.as_str()), Some("exit"));
        assert!(find(&binds, "super + q").is_none());
    }

    #[test]
    fn explicit_empty_mods_do_not_fall_back_to_defaults() {
        let binds = parse(SAMPLE);
        let mute = binds.iter().find(|b| b.key == "xf86audiomute").unwrap();
        assert!(mute.mods.is_empty());
        assert_eq!(mute.caps, vec!["Mute"]);
    }

    #[test]
    fn exec_binds_get_a_raw_rebind_value() {
        let binds = parse(SAMPLE);
        assert_eq!(binds[0].raw.as_deref(), Some("exec,breadbox"));
        assert_eq!(binds[2].raw, None);
    }

    #[test]
    fn families_collapse_into_one_row() {
        let binds = parse(SAMPLE);
        let rows = rows(&binds);
        let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "Open the app launcher (breadbox)",
                "Open BOS Settings",
                "Exit Hyprland (log out)",
                "Move focus",
                "Switch to workspace 1 to 10",
                "Previous workspace / Next workspace",
                "Volume down / up",
                "Mute",
            ]
        );
        let ws = &rows[4];
        assert_eq!(ws.caps, vec!["Super", "1 – 0"]);
        assert_eq!(ws.members.len(), 3);
        assert_eq!(rows[3].caps, vec!["Super", "← ↑ ↓ →"]);
        assert_eq!(rows[5].caps, vec!["Super", "[", "]"]);
        assert_eq!(rows[6].caps, vec!["Volume down", "Volume up"]);
    }

    #[test]
    fn a_lone_family_member_stays_a_plain_row() {
        let binds = parse(
            r#"{"bindings":[{"key":"1","action":"focus","label":"Switch to workspace 1","category":"workspaces"}]}"#,
        );
        let rows = rows(&binds);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Switch to workspace 1");
        assert_eq!(rows[0].caps, vec!["Super", "1"]);
    }

    #[test]
    fn rebind_combo_text_uses_keysym_names() {
        assert_eq!(to_hypr_mods_key("super + shift + u"), "SUPER_SHIFT,U");
        assert_eq!(to_hypr_mods_key("super + ,"), "SUPER,COMMA");
        assert_eq!(to_hypr_mods_key("f5"), ",F5");
        assert_eq!(to_hypr_mods_key("super + enter"), "SUPER,RETURN");
    }
}
