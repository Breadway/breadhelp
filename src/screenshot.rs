//! `--screenshot` CLI mode: switch to the named tab, capture it via
//! `bread-screenshots`, then exit — driven by `bread-ecosystem`'s
//! `bread-capture` orchestrator, or run standalone for one-off captures.
//!
//! breadhelp has three tabs worth capturing (Home/Learn/Ask), switched via
//! the same `Stack::set_visible_child_name` the tab switcher itself uses —
//! see `ui::tabs`. The window is a plain top-level (not layer-shell), so a
//! full known-size canvas capture is enough, same reasoning as breadpad's
//! popup view.
//!
//! One more view isn't a tab: "troubleshoot-wizard" opens the modal wizard
//! window (`ui::troubleshoot_wizard`, normally reachable via Home's
//! "Something's wrong?" button) directly.
//!
//! Deliberately NOT covered here: the onboarding tour (`ui::tour`). It's a
//! live, multi-step walkthrough that overlays callouts on the *real*
//! desktop as the user does real things (opens real apps, switches real
//! workspaces) — there's no meaningful "screenshot" of a single step in an
//! isolated environment with no real desktop for it to point at.
//! `ui::guide_view` and `ui::keybind_viewer` also aren't separate views:
//! both render embedded *inside* the Learn/Home tabs respectively, so
//! they're already covered by those tabs' own captures.

use bread_utils::screenshot_cli::SETTLE_DELAY;
use gtk4::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// Delay before opening the wizard modal — same reasoning as every other
/// app's PRE_POPUP_DELAY: the parent window's own layout needs a beat to
/// settle first.
const PRE_POPUP_DELAY: Duration = SETTLE_DELAY;

const KNOWN_VIEWS: &[&str] = &["home", "learn", "ask", "troubleshoot-wizard"];

#[derive(Clone)]
pub struct ScreenshotRequest {
    pub view: String,
    pub output: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Wire up the given view's screenshot sequence against an already-built
/// window and its tab `Stack`. Every path here ends by exiting the process
/// — it never returns control to the normal help-center UI.
pub fn dispatch(window: &gtk4::ApplicationWindow, stack: &gtk4::Stack, req: ScreenshotRequest) {
    if !KNOWN_VIEWS.contains(&req.view.as_str()) {
        eprintln!(
            "breadhelp: unknown screenshot view '{}' (known: {})",
            req.view,
            KNOWN_VIEWS.join(", ")
        );
        std::process::exit(1);
    }
    let output = req.output;
    let (width, height) = (req.width as i32, req.height as i32);

    if req.view == "troubleshoot-wizard" {
        window.connect_map(move |win| {
            let output = output.clone();
            let win = win.clone();
            gtk4::glib::timeout_add_local_once(PRE_POPUP_DELAY, move || {
                crate::ui::troubleshoot_wizard::open(&win, move |dialog| {
                    let output = output.clone();
                    dialog.connect_map(move |_| {
                        let output = output.clone();
                        gtk4::glib::timeout_add_local_once(SETTLE_DELAY, move || {
                            finish(bread_screenshots::capture_region(0, 0, width, height, &output));
                        });
                    });
                });
            });
        });
        // The caller (`ui::window::present`) returns immediately after this
        // for the screenshot path, skipping its own normal `window.present()`
        // call — trigger it here instead, so `connect_map` above actually has
        // something to fire for.
        window.present();
        return;
    }

    stack.set_visible_child_name(&req.view);

    window.connect_map(move |_| {
        let output = output.clone();
        gtk4::glib::timeout_add_local_once(SETTLE_DELAY, move || {
            finish(bread_screenshots::capture_region(0, 0, width, height, &output));
        });
    });
    window.present();
}

fn finish(result: anyhow::Result<()>) {
    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("breadhelp: screenshot capture failed: {e}");
            std::process::exit(1);
        }
    }
}
