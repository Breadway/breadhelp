//! argv for the help window. The same parser reads the first launch's argv
//! and every later launch's argv, which the single-instance plugin hands to
//! the running process instead of starting a second window.

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    /// `--page <id>`: open on a page (`home`, `keybinds`, `guides`, `fix`,
    /// `setup`). A bare launch (Super + /) opens Keybinds.
    pub page: Option<String>,
    /// Set by the every-login autostart entry. Shows first-run setup when it
    /// is still needed, otherwise stays hidden so a later Super + / is
    /// instant.
    pub autostart: bool,
    /// `--suggest <id>`: a breadd module noticed something (see
    /// `services::breadd::resolve`). Shows a banner without stealing focus.
    pub suggest: Option<String>,
}

/// Flags handled before any window exists, see `lib.rs`.
#[derive(Debug, PartialEq, Eq)]
pub enum Early {
    /// `breadhelp listen`: the bread command-bus subscriber.
    Listen,
    /// `--tour-event <id>`: forwarded to a running tour, else dropped.
    TourEvent(String),
    /// `--onboard`: start the tour from the first step.
    Onboard,
}

pub fn early(args: &[String]) -> Option<Early> {
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
        let a = parse(&v(&["breadhelp", "--autostart", "--page", "fix", "--suggest", "monitor-setup"]));
        assert!(a.autostart);
        assert_eq!(a.page.as_deref(), Some("fix"));
        assert_eq!(a.suggest.as_deref(), Some("monitor-setup"));
    }

    #[test]
    fn tour_flags_are_handled_early() {
        assert_eq!(early(&v(&["breadhelp", "--tour-event", "layer:breadbox"])), Some(Early::TourEvent("layer:breadbox".into())));
        assert_eq!(early(&v(&["breadhelp", "--onboard"])), Some(Early::Onboard));
        assert_eq!(early(&v(&["breadhelp", "listen"])), Some(Early::Listen));
        assert_eq!(early(&v(&["breadhelp", "--tour-event"])), None);
    }
}
