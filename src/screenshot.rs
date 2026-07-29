//! `--screenshot` CLI mode: switch to the named tab, capture it via
//! `bread-screenshots`, then exit — driven by `bread-ecosystem`'s
//! `bread-capture` orchestrator, or run standalone for one-off captures.
//!
//! breadhelp has three tabs worth capturing (Home/Learn/Ask), switched via
//! the same `Stack::set_visible_child_name` the tab switcher itself uses —
//! see `ui::tabs`. The window is a plain top-level (not layer-shell), so a
//! full known-size canvas capture is enough, same reasoning as breadpad's
//! popup view.

use gtk4::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// Extra settle time after `map` for the first frame to actually paint
/// before grim runs — `map` fires once the surface exists, not once
/// anything has been drawn into it.
const SETTLE_DELAY: Duration = Duration::from_millis(300);

const KNOWN_VIEWS: &[&str] = &["home", "learn", "ask"];

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
    stack.set_visible_child_name(&req.view);

    let output = req.output;
    let (width, height) = (req.width as i32, req.height as i32);
    window.connect_map(move |_| {
        let output = output.clone();
        gtk4::glib::timeout_add_local_once(SETTLE_DELAY, move || {
            finish(bread_screenshots::capture_region(0, 0, width, height, &output));
        });
    });
    // The caller (`ui::window::present`) returns immediately after this for
    // the screenshot path, skipping its own normal `window.present()` call
    // — trigger it here instead, so `connect_map` above actually has
    // something to fire for.
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
