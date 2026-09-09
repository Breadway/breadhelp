//! argv handling for a `HANDLES_COMMAND_LINE` GApplication: every
//! invocation (including ones launched while breadhelp is already running,
//! forwarded over D-Bus to the primary instance) reaches
//! `connect_command_line`, so `--onboard` on a second launch re-triggers the
//! tour in the existing window instead of spawning a duplicate.

use bread_utils::screenshot_cli::{validate_pair, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use std::path::Path;

#[derive(Clone)]
pub struct Action {
    /// Force-restart the onboarding tour from step 0, regardless of whether
    /// it was already completed or in progress.
    pub force_onboard: bool,
    /// Set by hyprland.lua's every-login autostart entry, as opposed to a
    /// user explicitly running `breadhelp` or pressing SUPER+/. Lets
    /// `present()` build the window (so the app is ready to respond the
    /// instant it's needed) without popping it open when onboarding is
    /// already done; autostart should only be visible on a genuine first
    /// run (day-zero wizard, then the desktop tour), never on every
    /// subsequent login.
    pub autostart: bool,
    /// From a breadd Lua module (e.g. `breadhelp-suggest.lua`) reacting to a
    /// system event. Resolved to banner text by `services::breadd::resolve`.
    pub suggest: Option<String>,
    /// From `breadhelp-tour.lua` forwarding a bread/Hyprland event (window
    /// opened, layer opened, workspace changed, or a rebind-chained ping) as
    /// a candidate "the user just did the thing" signal for the live tour.
    /// Only acted on if a tour is currently waiting for this exact id;
    /// see the crash-safety note on `ui::tour`.
    pub tour_event: Option<String>,
    /// Render the named tab, capture it, then exit instead of running
    /// normally. Known views: "home", "learn", "ask". See `crate::screenshot`.
    pub screenshot: Option<String>,
    /// PNG path to write the capture to. Required together with `screenshot`.
    pub output: Option<String>,
    /// Capture canvas width: matches the isolated compositor's output
    /// width (`bread-capture --isolate-width`).
    pub width: u32,
    /// Capture canvas height, see `width`.
    pub height: u32,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            force_onboard: false,
            autostart: false,
            suggest: None,
            tour_event: None,
            screenshot: None,
            output: None,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }
}

impl Action {
    /// `None` for a normal run. Exits the process with an error if the
    /// `--screenshot` / `--output` pair is incomplete, before any GTK setup
    /// happens.
    pub fn screenshot_request(&self) -> Option<crate::screenshot::ScreenshotRequest> {
        if let Err(e) = validate_pair(
            self.screenshot.as_deref(),
            self.output.as_deref().map(Path::new),
        ) {
            eprintln!("breadhelp: {e}");
            std::process::exit(1);
        }
        Some(crate::screenshot::ScreenshotRequest {
            view: self.screenshot.clone()?,
            output: self.output.clone()?.into(),
            width: self.width,
            height: self.height,
        })
    }
}

pub fn parse(args: &[std::ffi::OsString]) -> Action {
    let mut action = Action::default();
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        if arg == "--onboard" {
            action.force_onboard = true;
        } else if arg == "--autostart" {
            action.autostart = true;
        } else if arg == "--suggest" {
            action.suggest = it.next().and_then(|s| s.to_str()).map(str::to_string);
        } else if arg == "--tour-event" {
            action.tour_event = it.next().and_then(|s| s.to_str()).map(str::to_string);
        } else if arg == "--screenshot" {
            action.screenshot = it.next().and_then(|s| s.to_str()).map(str::to_string);
        } else if arg == "--output" {
            action.output = it.next().and_then(|s| s.to_str()).map(str::to_string);
        } else if arg == "--width" {
            if let Some(v) = it.next().and_then(|s| s.to_str()).and_then(|s| s.parse().ok()) {
                action.width = v;
            }
        } else if arg == "--height" {
            if let Some(v) = it.next().and_then(|s| s.to_str()).and_then(|s| s.parse().ok()) {
                action.height = v;
            }
        }
    }
    action
}
