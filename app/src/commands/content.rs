//! Read-only data for the page: guides, keybinds, fixes, symptom trees.
//! Everything is re-read on each call, so edits to binds.json or guide files
//! show up the next time the window opens (it refetches on every show).

use serde::Serialize;

use breadhelp_core::config::State;
use breadhelp_core::content::keybinds::{self, Keybind};
use breadhelp_core::content::markdown::{self, Block, Span};
use breadhelp_core::content::{keys, troubleshoot, ContentStore};
use breadhelp_core::fixes;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideSummary {
    category: String,
    category_title: String,
    id: String,
    title: String,
    tags: Vec<String>,
    has_simple: bool,
}

#[tauri::command]
pub fn list_guides() -> Vec<GuideSummary> {
    let store = ContentStore::load();
    store
        .all()
        .iter()
        .map(|g| GuideSummary {
            category: g.category.clone(),
            category_title: category_title(&g.category),
            id: g.id.clone(),
            title: g.meta.title.clone(),
            tags: g.meta.tags.clone(),
            has_simple: g.dir.join("content.beginner.md").is_file(),
        })
        .collect()
}

/// `getting-started` -> `Getting started`.
pub fn category_title(id: &str) -> String {
    let words = id.replace('-', " ");
    let mut c = words.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

#[derive(Serialize)]
pub struct Run {
    text: String,
    bold: bool,
    code: bool,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SpanView {
    Text { runs: Vec<Run> },
    /// A `[Run]` or `[Terminal]` button. `action` is its position among the
    /// guide's buttons; [`super::actions::run_guide_action`] re-reads the
    /// guide to find the command, so the page never holds one to send back.
    Action { label: String, action: usize },
    Keys { caps: Vec<String> },
}

#[derive(Serialize)]
pub struct BlockView {
    kind: &'static str,
    spans: Vec<SpanView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideView {
    title: String,
    category_title: String,
    simplified: bool,
    blocks: Vec<BlockView>,
}

/// The body actually shown: `content.beginner.md` when asked for and present.
pub fn guide_body(category: &str, id: &str, simplified: bool) -> Option<String> {
    let store = ContentStore::load();
    let guide = store.guide(category, id)?;
    Some(store.body(guide, simplified))
}

#[tauri::command]
pub fn get_guide(category: String, id: String, simplified: bool) -> Option<GuideView> {
    let store = ContentStore::load();
    let guide = store.guide(&category, &id)?;
    let has_simple = guide.dir.join("content.beginner.md").is_file();
    let simplified = simplified && has_simple;
    let body = store.body(guide, simplified);
    let binds = keybinds::load();
    let mut action = 0;
    let blocks = markdown::parse(&body)
        .into_iter()
        .filter_map(|block| {
            let (kind, spans) = match block {
                // The page shows the guide's title itself; a leading `# Title`
                // in the body would repeat it.
                Block::Heading(1, _) => return None,
                Block::Heading(_, text) => ("heading", vec![Span::Text(text)]),
                Block::Paragraph(spans) => ("paragraph", spans),
                Block::Bullet(spans) => ("bullet", spans),
            };
            let spans = spans
                .into_iter()
                .map(|span| match span {
                    Span::Text(t) => SpanView::Text {
                        runs: markdown::inline(&t)
                            .into_iter()
                            .map(|r| Run { text: r.text, bold: r.bold, code: r.code })
                            .collect(),
                    },
                    Span::Run { label, .. } | Span::Terminal { label, .. } => {
                        action += 1;
                        SpanView::Action { label, action: action - 1 }
                    }
                    Span::Keybind(combo) => SpanView::Keys {
                        caps: keybinds::find(&binds, &combo)
                            .map(|b| b.caps.clone())
                            .unwrap_or_else(|| keys::display_text_combo(&combo)),
                    },
                })
                .collect();
            Some(BlockView { kind, spans })
        })
        .collect();
    Some(GuideView {
        title: guide.meta.title.clone(),
        category_title: category_title(&guide.category),
        simplified,
        blocks,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyRow {
    caps: Vec<String>,
    label: String,
    category: String,
    /// The bind's identity for Try it / temporary rebind; only for a row
    /// that stands for exactly one bind.
    combo_id: Option<String>,
    /// What an `exec` bind runs, shown in the row's details.
    command: Option<String>,
    can_try: bool,
    can_rebind: bool,
}

#[derive(Serialize)]
pub struct Category {
    id: String,
    title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindsView {
    rows: Vec<KeyRow>,
    categories: Vec<Category>,
    source: String,
    bind_count: usize,
}

const CATEGORY_ORDER: &[(&str, &str)] = &[
    ("apps", "Apps"),
    ("windows", "Windows"),
    ("workspaces", "Workspaces"),
    ("focus", "Focus and layout"),
    ("screenshots", "Screenshots"),
    ("media", "Media keys"),
    ("mouse", "Mouse"),
];

fn keybind_category_title(id: &str) -> String {
    match CATEGORY_ORDER.iter().find(|(c, _)| *c == id) {
        Some((_, t)) => t.to_string(),
        None if id.is_empty() => "Other".to_string(),
        None => category_title(id),
    }
}

fn category_rank(id: &str) -> usize {
    CATEGORY_ORDER.iter().position(|(c, _)| *c == id).unwrap_or(CATEGORY_ORDER.len())
}

#[tauri::command]
pub fn get_keybinds() -> KeybindsView {
    let binds = keybinds::load();
    keybinds_view(&binds)
}

pub fn keybinds_view(binds: &[Keybind]) -> KeybindsView {
    let rows: Vec<KeyRow> = keybinds::rows(binds)
        .into_iter()
        .map(|row| {
            let single = (row.members.len() == 1).then(|| &binds[row.members[0]]);
            KeyRow {
                caps: row.caps,
                label: row.label,
                category: if row.category.is_empty() { "other".into() } else { row.category },
                combo_id: single.map(|b| b.combo_id.clone()),
                command: single.and_then(|b| b.raw.as_ref()).and_then(|r| r.strip_prefix("exec,")).map(str::to_string),
                can_try: single.is_some_and(|b| b.demo_cmd.is_some()),
                can_rebind: single.is_some_and(|b| b.raw.is_some()),
            }
        })
        .collect();
    let mut ids: Vec<String> = Vec::new();
    for r in &rows {
        if !ids.contains(&r.category) {
            ids.push(r.category.clone());
        }
    }
    ids.sort_by_key(|c| category_rank(c));
    KeybindsView {
        categories: ids.iter().map(|id| Category { id: id.clone(), title: keybind_category_title(id) }).collect(),
        rows,
        source: "~/.config/hypr/binds.json".into(),
        bind_count: binds.len(),
    }
}

#[derive(Serialize)]
pub struct FixView {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    tags: &'static str,
}

#[tauri::command]
pub fn list_fixes() -> Vec<FixView> {
    fixes::QUICK_FIXES
        .iter()
        .map(|f| FixView { id: f.id, label: f.label, description: f.description, tags: f.tags })
        .collect()
}

#[derive(Serialize)]
pub struct CommandView {
    name: &'static str,
    description: &'static str,
}

#[tauri::command]
pub fn list_commands() -> Vec<CommandView> {
    fixes::COMMANDS.iter().map(|c| CommandView { name: c.name, description: c.description }).collect()
}

#[derive(Serialize)]
pub struct OptionView {
    label: String,
    goto: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    id: String,
    prompt: String,
    options: Vec<OptionView>,
    fix: Option<String>,
    requires_confirm: bool,
}

#[derive(Serialize)]
pub struct TreeView {
    id: String,
    title: String,
    summary: String,
    nodes: Vec<NodeView>,
}

#[tauri::command]
pub fn list_symptoms() -> Vec<TreeView> {
    troubleshoot::load_all()
        .into_iter()
        .map(|(id, tree)| TreeView {
            id,
            title: tree.title,
            summary: tree.summary,
            nodes: tree
                .nodes
                .into_iter()
                .map(|n| NodeView {
                    id: n.id,
                    prompt: n.prompt,
                    options: n.options.into_iter().map(|o| OptionView { label: o.label, goto: o.goto }).collect(),
                    requires_confirm: n.fix.as_ref().is_some_and(|f| f.requires_confirm),
                    fix: n.fix.map(|f| f.description),
                })
                .collect(),
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Prefs {
    mode: &'static str,
    has_settings: bool,
}

#[tauri::command]
pub fn get_prefs() -> Prefs {
    Prefs {
        mode: State::load().mode().as_str(),
        has_settings: breadhelp_core::services::exec::command_exists("bos-settings"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keybind_view_orders_categories_and_exposes_single_bind_actions() {
        let binds = keybinds::parse(
            r#"{"bindings":[
              {"key":"1","action":"focus","label":"Switch to workspace 1","category":"workspaces"},
              {"key":"2","action":"focus","label":"Switch to workspace 2","category":"workspaces"},
              {"key":"SPACE","action":"exec","command":"breadbox","label":"Open the launcher","category":"apps","demo_cmd":"breadbox"}
            ]}"#,
        );
        let view = keybinds_view(&binds);
        let ids: Vec<&str> = view.categories.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["apps", "workspaces"]);
        let launcher = view.rows.iter().find(|r| r.label == "Open the launcher").unwrap();
        assert!(launcher.can_try && launcher.can_rebind);
        assert_eq!(launcher.command.as_deref(), Some("breadbox"));
        let ws = view.rows.iter().find(|r| r.category == "workspaces").unwrap();
        assert!(ws.combo_id.is_none() && !ws.can_try);
    }

    #[test]
    fn category_titles_read_naturally() {
        assert_eq!(category_title("getting-started"), "Getting started");
        assert_eq!(keybind_category_title("focus"), "Focus and layout");
        assert_eq!(keybind_category_title(""), "Other");
    }
}
