//! argv for the help window. The same parser reads the first launch's argv
//! and every later launch's argv, which the single-instance plugin hands to
//! the running process instead of starting a second window.

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    /// `--page <id>`: open on a page (`home`, `keybinds`, `guides`, `fix`,
    /// `setup`). A bare launch (Super + /) opens Keybinds.
    pub page: Option<String>,
    /// Set by the every-login autostart entry. Shows first-run setup when it
    /// is still needed; otherwise starts hidden only when the resident
    /// setting is on, and exits when it isn't.
    pub autostart: bool,
    /// `--unload`: a hidden resident window quits (see `commands::window`).
    pub unload: bool,
    /// `--suggest <id>`: a breadd module noticed something (see
    /// `services::breadd::resolve`). Shows a banner without stealing focus.
    pub suggest: Option<String>,
}

/// Flags handled before any window exists, see `lib.rs`.
#[derive(Debug, PartialEq, Eq)]
pub enum Early {
    /// `-h` / `--help`: print [`USAGE`] and exit 0.
    Help,
    /// `--version`: print the crate version and exit 0.
    Version,
    /// `breadhelp listen`: the bread command-bus subscriber.
    Listen,
    /// `--tour-event <id>`: forwarded to a running tour, else dropped.
    TourEvent(String),
    /// `--onboard`: start the tour from the first step.
    Onboard,
}

/// Kept in step with [`parse`] and the page ids the frontend understands
/// (`Page` in `frontend/src/lib/api.ts`).
pub const USAGE: &str = concat!(
    "breadhelp ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "Bread OS help window\n\n",
    "Usage:\n",
    "  breadhelp [options]\n",
    "  breadhelp listen\n\n",
    "Options:\n",
    "  -h, --help             Print this help and exit\n",
    "      --version          Print the version and exit\n",
    "      --page <id>        Open on a page: home, keybinds, guides, fix, setup\n",
    "      --autostart        First-run setup, or the resident hidden launch\n",
    "      --unload           Ask a hidden resident window to quit\n",
    "      --suggest <id>     Show a suggestion banner without taking focus\n",
    "      --onboard          Start the guided tour from the first step\n",
    "      --tour-event <id>  Forward an event to a running tour\n\n",
    "Commands:\n",
    "  listen                 Subscribe to the bread command bus\n",
);

pub fn early(args: &[String]) -> Option<Early> {
    // Answer help and version before anything that could start GTK or Tauri,
    // take the single-instance hand-off, or talk to breadd. Every other flag
    // is still parsed below.
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => return Some(Early::Help),
            "--version" => return Some(Early::Version),
            _ => {}
        }
    }

    if args.get(1).map(String::as_str) == Some("listen") {
        return Some(Early::Listen);
    }
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--tour-event" => return it.next().map(|id| Early::TourEvent(id.clone())),
            "--onboard" => return Some(Early::Onboard),
            _ => {}
        }
    }
    None
}

pub fn parse(args: &[String]) -> Args {
    let mut out = Args::default();
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--page" => out.page = it.next().cloned(),
            "--autostart" => out.autostart = true,
            "--unload" => out.unload = true,
            "--suggest" => out.suggest = it.next().cloned(),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn bare_launch_has_no_page() {
        assert_eq!(parse(&v(&["breadhelp"])), Args::default());
        assert_eq!(early(&v(&["breadhelp"])), None);
    }

    #[test]
    fn reads_page_autostart_and_suggest() {
        let a = parse(&v(&[
            "breadhelp",
            "--autostart",
            "--page",
            "fix",
            "--suggest",
            "monitor-setup",
        ]));
        assert!(a.autostart);
        assert_eq!(a.page.as_deref(), Some("fix"));
        assert_eq!(a.suggest.as_deref(), Some("monitor-setup"));
        assert!(parse(&v(&["breadhelp", "--unload"])).unload);
    }

    #[test]
    fn help_and_version_are_handled_early() {
        assert_eq!(early(&v(&["breadhelp", "-h"])), Some(Early::Help));
        assert_eq!(early(&v(&["breadhelp", "--help"])), Some(Early::Help));
        assert_eq!(early(&v(&["breadhelp", "--version"])), Some(Early::Version));
        // Help wins even when another launch flag comes first, and even for
        // the `listen` subcommand: neither should start a window or a
        // subscriber just to answer it.
        assert_eq!(
            early(&v(&["breadhelp", "--page", "fix", "--help"])),
            Some(Early::Help)
        );
        assert_eq!(
            early(&v(&["breadhelp", "listen", "--help"])),
            Some(Early::Help)
        );
    }

    #[test]
    fn usage_lists_the_supported_surface() {
        for needle in [
            "--help",
            "--version",
            "--page",
            "--autostart",
            "--unload",
            "--suggest",
            "--onboard",
            "--tour-event",
            "listen",
            "home",
            "keybinds",
            "guides",
            "fix",
            "setup",
        ] {
            assert!(USAGE.contains(needle), "usage is missing {needle}");
        }
        assert!(USAGE.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn tour_flags_are_handled_early() {
        assert_eq!(
            early(&v(&["breadhelp", "--tour-event", "layer:breadbox"])),
            Some(Early::TourEvent("layer:breadbox".into()))
        );
        assert_eq!(early(&v(&["breadhelp", "--onboard"])), Some(Early::Onboard));
        assert_eq!(early(&v(&["breadhelp", "listen"])), Some(Early::Listen));
        assert_eq!(early(&v(&["breadhelp", "--tour-event"])), None);
    }
}
