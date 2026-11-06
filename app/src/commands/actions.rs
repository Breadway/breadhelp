//! Everything the page can make happen. Each command takes an id (a fix id,
//! a guide button's position, a bind's combo) and looks the command up here,
//! so no command text from the webview is ever executed.

use breadhelp_core::config::{Mode, State};
use breadhelp_core::content::keybinds;
use breadhelp_core::content::markdown::{self, Block, Span};
use breadhelp_core::content::troubleshoot;
use breadhelp_core::services::{exec, hyprland};
use breadhelp_core::{fixes, tourctl};
use tauri::AppHandle;

/// The safety gate every run command goes through. A fix authored with
/// `requires_confirm` only runs when the caller says the user already
/// answered a confirmation prompt; nothing about the fix's id or text can
/// bypass it. Fixes without the flag keep running directly.
pub fn confirm_guard(label: &str, requires_confirm: bool, confirmed: bool) -> Result<(), String> {
    if requires_confirm && !confirmed {
        return Err(format!("{label} needs confirmation before it can run"));
    }
    Ok(())
}

/// The command a run is about to execute, returned only after the gate has
/// passed. Both run commands call this, so neither can drop the check.
fn confirmed_command(label: &str, requires_confirm: bool, command: &str, confirmed: bool) -> Result<String, String> {
    confirm_guard(label, requires_confirm, confirmed)?;
    Ok(command.to_string())
}

/// One-click fix from the Home / Fix a problem lists. Waits for the fix to
/// finish and reports whether it succeeded. `confirmed` is the webview's
/// answer to the prompt this fix requested (see `confirm_guard`).
#[tauri::command]
pub async fn run_fix(id: String, confirmed: bool) -> Result<bool, String> {
    let fix = fixes::quick_fix(&id).ok_or_else(|| format!("unknown fix {id:?}"))?;
    let command = confirmed_command(fix.label, fix.requires_confirm, fix.command, confirmed)?;
    Ok(tauri::async_runtime::spawn_blocking(move || exec::run_status(&command)).await.unwrap_or(false))
}

/// The fix at the end of a symptom tree, by tree file stem and node id.
#[tauri::command]
pub async fn run_symptom_fix(tree: String, node: String, confirmed: bool) -> Result<bool, String> {
    let trees = troubleshoot::load_all();
    let fix = trees
        .get(&tree)
        .and_then(|t| t.node(&node))
        .and_then(|n| n.fix.as_ref())
        .ok_or_else(|| format!("no fix at {tree}:{node}"))?;
    let command = confirmed_command(&fix.description, fix.requires_confirm, &fix.command, confirmed)?;
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
        // Same numbering as `content::get_guide`: an empty [Terminal] is
        // text there, not a button.
        .filter(|s| match s {
            Span::Run { .. } => true,
            Span::Terminal { argv, .. } => !argv.is_empty(),
            _ => false,
        })
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
    keybinds::validate_combo(&new_combo)?;
    // Hyprland would silently replace whatever else is on that combo.
    if let Some(taken) = keybinds::find(&binds, &new_combo).filter(|b| b.combo_id != combo_id) {
        return Err(format!("{} is already \"{}\"", taken.caps.join(" + "), taken.action));
    }
    let mods_key = keybinds::to_hypr_mods_key(&new_combo);
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

/// "Take the tour": restart the tour from the first step, and get this
/// window out of the way (hidden or closed, per the resident setting).
#[tauri::command]
pub fn start_tour(app: AppHandle) {
    tourctl::spawn_tour(&["--onboard"]);
    super::window::dismiss(&app);
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

    #[test]
    fn a_requires_confirm_fix_refuses_without_confirmation() {
        assert!(confirm_guard("Reset the network", true, false).is_err());
    }

    #[test]
    fn a_requires_confirm_fix_runs_after_confirmation() {
        assert!(confirm_guard("Reset the network", true, true).is_ok());
    }

    #[test]
    fn a_fix_without_the_flag_runs_directly() {
        assert!(confirm_guard("Fix sound", false, false).is_ok());
        assert!(confirm_guard("Fix sound", false, true).is_ok());
    }

    #[test]
    fn both_run_commands_apply_the_gate_before_returning_a_command() {
        assert!(confirmed_command("Reset the network", true, "nmcli radio wifi off", false).is_err());
        assert_eq!(
            confirmed_command("Reset the network", true, "nmcli radio wifi off", true).unwrap(),
            "nmcli radio wifi off"
        );
        assert_eq!(confirmed_command("Fix sound", false, "true", false).unwrap(), "true");
    }
}
