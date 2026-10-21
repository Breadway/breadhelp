//! The fixed tables the help window offers: one-click fixes and the small
//! command gallery search covers. The webview only ever names an entry by
//! `id`; the command itself is looked up here, so nothing the page sends is
//! ever run as a command.

pub struct QuickFix {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub command: &'static str,
    /// Words search matches on besides the label.
    pub tags: &'static str,
}

pub const QUICK_FIXES: &[QuickFix] = &[
    QuickFix {
        id: "sound",
        label: "Fix sound",
        description: "Restarts PipeWire and WirePlumber.",
        command: "systemctl --user restart wireplumber pipewire pipewire.socket pipewire-pulse pipewire-pulse.socket",
        tags: "audio speakers no sound volume pipewire",
    },
    QuickFix {
        id: "wifi",
        label: "Reconnect Wi-Fi",
        description: "Turns the Wi-Fi radio off and back on.",
        command: "nmcli radio wifi off && sleep 1 && nmcli radio wifi on",
        tags: "wifi wi-fi network internet wireless",
    },
    QuickFix {
        id: "update",
        label: "Update everything",
        description: "System packages and bread apps, in one go.",
        command: "bos-update",
        tags: "update upgrade bakery pacman",
    },
    QuickFix {
        id: "backup",
        label: "Create backup",
        description: "Takes a system snapshot you can roll back to.",
        command: "snapper -c root create --description 'breadhelp manual backup'",
        tags: "backup snapshot snapper restore",
    },
];

pub fn quick_fix(id: &str) -> Option<&'static QuickFix> {
    QUICK_FIXES.iter().find(|f| f.id == id)
}

pub struct CommandEntry {
    pub name: &'static str,
    pub description: &'static str,
}

/// Not every `bread*` binary, just the ones worth surfacing to someone
/// searching for "what can I run".
pub const COMMANDS: &[CommandEntry] = &[
    CommandEntry { name: "bos-update", description: "Update both the system (pacman) and the bread apps (bakery) in one go." },
    CommandEntry { name: "bos-settings", description: "Open BOS Settings." },
    CommandEntry { name: "breadbox", description: "Open the app launcher." },
    CommandEntry { name: "breadpad", description: "Open notes and reminders." },
    CommandEntry { name: "breadman", description: "Open the notes and task manager." },
    CommandEntry { name: "breadpaper", description: "Open the wallpaper picker." },
    CommandEntry { name: "breadclip", description: "Open clipboard history." },
    CommandEntry { name: "breadcrumbs", description: "Manage Wi-Fi networks and location profiles." },
];

pub fn command(name: &str) -> Option<&'static CommandEntry> {
    COMMANDS.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = QUICK_FIXES.iter().map(|f| f.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), QUICK_FIXES.len());
        let mut names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), COMMANDS.len());
    }

    #[test]
    fn lookups_only_return_table_entries() {
        assert_eq!(quick_fix("wifi").map(|f| f.label), Some("Reconnect Wi-Fi"));
        assert!(quick_fix("rm -rf /").is_none());
        assert!(command("breadbox").is_some());
        assert!(command("sh").is_none());
    }
}
