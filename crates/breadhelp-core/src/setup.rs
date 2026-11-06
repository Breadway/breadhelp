//! First-boot setup (timezone, network, updates, a first snapshot), shown
//! once on the first login before the desktop tour. Existing users who
//! already finished the tour never see it: [`needs_setup`] is false when
//! `onboarding.completed` is set, so upgrades don't re-prompt.

use std::process::Command;

use crate::config::State;
use crate::services::exec;

pub const STEP_COUNT: usize = 6;

pub fn needs_setup() -> bool {
    let state = State::load();
    needs_setup_from(state.day_zero_completed(), state.onboarding_completed())
}

pub fn needs_setup_from(day_zero_done: bool, onboarding_done: bool) -> bool {
    !day_zero_done && !onboarding_done
}

/// `(button label, program, args)` for the network step.
pub fn pick_network_tool(
    has_settings: bool,
    has_nm: bool,
) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    if has_settings {
        Some((
            "Open network settings",
            "bos-settings",
            &["--page", "network"],
        ))
    } else if has_nm {
        Some(("Open network editor", "nm-connection-editor", &[]))
    } else {
        None
    }
}

/// `(button label, program, args)` for the updates step.
pub fn pick_updates_tool(
    has_settings: bool,
) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    if has_settings {
        Some(("Open Updates", "bos-settings", &["--page", "updates"]))
    } else {
        None
    }
}

/// Timezone names come from the UI and end up as an argv entry; this also
/// keeps them to the shape `timedatectl list-timezones` actually produces.
pub fn is_safe_timezone_name(tz: &str) -> bool {
    !tz.is_empty()
        && tz.len() < 128
        && tz
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '+' | '-' | '.'))
}

fn parse_timezone_property(stdout: &str) -> Option<String> {
    stdout
        .trim()
        .strip_prefix("Timezone=")
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn parse_timezone_list(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// `(current timezone or "", every known timezone)`. Empty list when
/// `timedatectl` is missing.
pub fn load_timezones() -> (String, Vec<String>) {
    let current = Command::new("timedatectl")
        .args(["show", "--property=Timezone"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| parse_timezone_property(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    let zones = Command::new("timedatectl")
        .arg("list-timezones")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_timezone_list(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    (current, zones)
}

/// Blocks. Tries `timedatectl` directly (polkit may allow it), then via
/// `pkexec`. Returns a message for the user and whether it worked.
pub fn apply_timezone(tz: &str) -> (String, bool) {
    if !is_safe_timezone_name(tz) {
        return ("That timezone name doesn't look valid.".into(), false);
    }
    if exec::run_argv_status("timedatectl", &["set-timezone", tz]) {
        return (format!("Timezone set to {tz}."), true);
    }
    if !exec::command_exists("pkexec") {
        return (
            "Need permission to change the timezone, and pkexec isn't available.".into(),
            false,
        );
    }
    if exec::run_argv_status("pkexec", &["timedatectl", "set-timezone", tz]) {
        (format!("Timezone set to {tz}."), true)
    } else {
        (
            "Couldn't change the timezone. You can try again from BOS Settings.".into(),
            false,
        )
    }
}

pub fn snapper_available() -> bool {
    exec::command_exists("snapper")
}

/// Blocks. Creates the `day-zero` restore point.
pub fn create_snapshot() -> bool {
    exec::run_argv_status("snapper", &["-c", "root", "create", "-d", "day-zero"])
}

pub fn snapshot_message(ok: bool) -> &'static str {
    if ok {
        "Created a snapshot named day-zero."
    } else {
        "Couldn't create a snapshot. That's OK, you can make one later from Settings."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_onboarding_skips_setup() {
        assert!(!needs_setup_from(false, true));
        assert!(!needs_setup_from(true, true));
        assert!(!needs_setup_from(true, false));
        assert!(needs_setup_from(false, false));
    }

    #[test]
    fn network_prefers_bos_settings_then_nm() {
        let (label, prog, args) = pick_network_tool(true, true).unwrap();
        assert_eq!(prog, "bos-settings");
        assert_eq!(args, &["--page", "network"]);
        assert!(label.contains("network"));
        let (_, prog, _) = pick_network_tool(false, true).unwrap();
        assert_eq!(prog, "nm-connection-editor");
        assert!(pick_network_tool(false, false).is_none());
    }

    #[test]
    fn updates_tool_opens_the_updates_page() {
        let (label, prog, args) = pick_updates_tool(true).unwrap();
        assert_eq!(prog, "bos-settings");
        // The step must land on Updates, not whatever page bos-settings
        // happened to show last.
        assert_eq!(args, &["--page", "updates"]);
        assert_eq!(label, "Open Updates");
        assert!(pick_updates_tool(false).is_none());
    }

    #[test]
    fn timezone_names_reject_shell_metacharacters() {
        assert!(is_safe_timezone_name("Australia/Perth"));
        assert!(is_safe_timezone_name("Etc/GMT+8"));
        assert!(is_safe_timezone_name("America/Argentina/ComodRivadavia"));
        assert!(!is_safe_timezone_name(""));
        assert!(!is_safe_timezone_name("America/New York"));
        assert!(!is_safe_timezone_name("x; rm -rf /"));
        assert!(!is_safe_timezone_name("$(reboot)"));
    }

    #[test]
    fn parses_timedatectl_show_and_list() {
        assert_eq!(
            parse_timezone_property("Timezone=Australia/Perth\n").as_deref(),
            Some("Australia/Perth")
        );
        assert_eq!(parse_timezone_property("Timezone=\n"), None);
        assert_eq!(
            parse_timezone_list("Africa/Abidjan\nAustralia/Perth\n\nUTC\n"),
            vec!["Africa/Abidjan", "Australia/Perth", "UTC"]
        );
    }
}
