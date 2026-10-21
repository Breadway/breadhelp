//! Everything the page can make happen. Each command takes an id (a fix id,
//! a guide button's position, a bind's combo) and looks the command up here,
//! so no command text from the webview is ever executed.

use breadhelp_core::config::{Mode, State};
use breadhelp_core::content::keybinds;
use breadhelp_core::content::markdown::{self, Block, Span};
use breadhelp_core::content::troubleshoot;
use breadhelp_core::services::{exec, hyprland};
use breadhelp_core::{fixes, tourctl};
use tauri::{AppHandle, Manager};

/// One-click fix from the Home / Fix a problem lists. Waits for the fix to
/// finish and reports whether it succeeded.
#[tauri::command]
pub async fn run_fix(id: String) -> Result<bool, String> {
    let fix = fixes::quick_fix(&id).ok_or_else(|| format!("unknown fix {id:?}"))?;
    Ok(tauri::async_runtime::spawn_blocking(move || exec::run_status(fix.command)).await.unwrap_or(false))
}

/// The fix at the end of a symptom tree, by tree file stem and node id.
#[tauri::command]
pub async fn run_symptom_fix(tree: String, node: String) -> Result<bool, String> {
    let trees = troubleshoot::load_all();
    let command = trees
        .get(&tree)
        .and_then(|t| t.node(&node))
        .and_then(|n| n.fix.as_ref())
        .map(|f| f.command.clone())
        .ok_or_else(|| format!("no fix at {tree}:{node}"))?;
    Ok(tauri::async_runtime::spawn_blocking(move || exec::run_status(&command)).await.unwrap_or(false))
}

/// The `action`-th `[Run]`/`[Terminal]` button of a guide, counted the same
/// way `content::get_guide` numbered them.
#[tauri::command]
pub fn run_guide_action(category: String, id: String, simplified: bool, action: usize) -> Result<(), String> {
    let body = super::content::guide_body(&category, &id, simplified).ok_or("unknown guide")?;
    let span = markdown::parse(&body)
        .into_iter()
        .filter_map(|b| match b {
            Block::Paragraph(s) | Block::Bullet(s) => Some(s),
            Block::Heading(..) => None,
        })
        .flatten()
        .filter(|s| matches!(s, Span::Run { .. } | Span::Terminal { .. }))
        .nth(action)
        .ok_or("no such button")?;
    match span {
        Span::Run { command, .. } => exec::run(&command),
        Span::Terminal { argv, .. } => exec::run_terminal(&argv),
        _ => unreachable!("filtered to buttons above"),
    }
    Ok(())
}

/// Search's command gallery: only names from `fixes::COMMANDS` run.
#[tauri::command]
pub fn run_gallery_command(name: String) -> Result<(), String> {
    let entry = fixes::command(&name).ok_or_else(|| format!("unknown command {name:?}"))?;
    exec::run_argv(entry.name, &[]);
    Ok(())
}

/// "Try it" on a keybind row: the bind's `demo_cmd`, never its real action
/// (a bind's real action can be "log out").
#[tauri::command]
pub fn try_keybind(combo_id: String) -> Result<(), String> {
    let binds = keybinds::load();
    let bind = binds.iter().find(|b| b.combo_id == combo_id).ok_or("unknown keybind")?;
    let demo = bind.demo_cmd.as_ref().ok_or("this keybind has no demo")?;
    exec::run(demo);
    Ok(())
}

/// Temporary rebind of an `exec` bind to `new_combo` (typed or captured on
/// the page). Resets on the next Hyprland reload or login.
#[tauri::command]
pub fn rebind_keybind(combo_id: String, new_combo: String) -> Result<(), String> {
    let binds = keybinds::load();
    let bind = binds.iter().find(|b| b.combo_id == combo_id).ok_or("unknown keybind")?;
    let raw = bind.raw.as_ref().ok_or("only app-launching keybinds can be changed here")?;
    let mods_key = keybinds::to_hypr_mods_key(&new_combo);
    if mods_key.is_empty() || mods_key.ends_with(',') {
        return Err("press a key combination first".into());
    }
    hyprland::rebind_temp(&format!("{mods_key},{raw}"))
}

#[tauri::command]
pub fn set_mode(mode: String) {
    let mode = match mode.as_str() {
        "beginner" => Mode::Beginner,
        "dad" => Mode::Dad,
        "compact" => Mode::Compact,
        _ => Mode::Normal,
    };
    State::load().set_mode(mode);
}

/// BOS Settings pages this window links to.
const SETTINGS_PAGES: &[&str] = &["keybinds", "network", "updates", "sound", "displays", "snapshots"];

#[tauri::command]
pub fn open_settings(page: String) -> Result<(), String> {
    if !SETTINGS_PAGES.contains(&page.as_str()) {
        return Err(format!("unknown settings page {page:?}"));
    }
    exec::run_argv("bos-settings", &["--page", &page]);
    Ok(())
}

/// breadbug is terminal-only (it needs a TTY), same as the guide's button.
#[tauri::command]
pub fn report_problem() {
    exec::run_terminal(&["breadbug".to_string()]);
}

/// "Take the tour": hide this window so the overlay has the screen, then
/// restart the tour from the first step.
#[tauri::command]
pub fn start_tour(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    tourctl::spawn_tour(&["--onboard"]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Only the rejecting path is exercised: an accepted page would really
    // launch bos-settings.
    fn settings_links_are_an_allowlist() {
        assert!(open_settings("../../bin/sh".into()).is_err());
        assert!(open_settings("keybinds; reboot".into()).is_err());
        assert!(SETTINGS_PAGES.contains(&"keybinds"));
    }
}
