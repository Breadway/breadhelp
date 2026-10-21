//! `--screenshot <view> --output <png>`: show the named page, capture it with
//! grim, exit. Driven by bread-ecosystem's `bread-capture`, or run by hand.
//! Same shape as bos-settings' screenshot mode: a webview has no map signal
//! to hook, so this waits for first paint, tells the page which view to
//! show, waits again for that view's data, then captures the known-size
//! window region.
//!
//! The tour is not a view here: it overlays the real desktop and has
//! nothing to point at in an isolated capture compositor.

use bread_utils::screenshot_cli::{validate_pair, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{Emitter, Manager};

const INITIAL_SETTLE_DELAY: Duration = Duration::from_millis(2000);
const VIEW_SETTLE_DELAY: Duration = Duration::from_millis(1500);

const KNOWN_VIEWS: &[&str] = &["home", "keybinds", "guides", "fix", "search", "setup"];

/// The GTK window's view names, which bread-capture still requests: the
/// Learn tab is Guides now, Ask is the search palette, and the
/// troubleshooter dialog is part of Fix a problem.
fn canonical_view(view: &str) -> &str {
    match view {
        "learn" => "guides",
        "ask" => "search",
        "troubleshoot-wizard" => "fix",
        other => other,
    }
}

pub struct ScreenshotRequest {
    view: String,
    output: PathBuf,
    width: u32,
    height: u32,
}

/// `None` for a normal run. Exits with an error for an unknown view or an
/// incomplete `--screenshot` / `--output` pair, before any window exists.
pub fn parse(args: &[String]) -> Option<ScreenshotRequest> {
    let mut view = None;
    let mut output = None;
    let mut width = DEFAULT_WIDTH;
    let mut height = DEFAULT_HEIGHT;
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--screenshot" => view = it.next().cloned(),
            "--output" => output = it.next().cloned(),
            "--width" => {
                if let Some(v) = it.next().and_then(|s| s.parse().ok()) {
                    width = v;
                }
            }
            "--height" => {
                if let Some(v) = it.next().and_then(|s| s.parse().ok()) {
                    height = v;
                }
            }
            _ => {}
        }
    }
    if let Err(e) = validate_pair(view.as_deref(), output.as_deref().map(Path::new)) {
        eprintln!("breadhelp: {e}");
        std::process::exit(1);
    }
    let view = canonical_view(&view?).to_string();
    if !KNOWN_VIEWS.contains(&view.as_str()) {
        eprintln!("breadhelp: unknown screenshot view '{view}' (known: {})", KNOWN_VIEWS.join(", "));
        std::process::exit(1);
    }
    Some(ScreenshotRequest { view, output: output?.into(), width, height })
}

pub fn dispatch(req: ScreenshotRequest, app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_size(tauri::LogicalSize::new(req.width, req.height));
        let _ = w.show();
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(INITIAL_SETTLE_DELAY).await;
        if let Err(e) = app.emit("screenshot-set-view", &req.view) {
            eprintln!("breadhelp: failed to emit screenshot-set-view: {e}");
            std::process::exit(1);
        }
        tokio::time::sleep(VIEW_SETTLE_DELAY).await;
        finish(capture(req.width as i32, req.height as i32, &req.output));
    });
}

fn capture(w: i32, h: i32, out: &Path) -> anyhow::Result<()> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let out_str = out.to_str().ok_or_else(|| anyhow::anyhow!("output path is not valid UTF-8"))?;
    let geometry = format!("0,0 {w}x{h}");
    let result = bread_utils::proc::run("grim", &["-g", &geometry, out_str], Duration::from_secs(5));
    if !result.success {
        anyhow::bail!("grim failed for geometry {geometry}: {}", result.stderr.trim());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_gtk_view_names_still_resolve() {
        for old in ["learn", "ask", "troubleshoot-wizard"] {
            assert!(KNOWN_VIEWS.contains(&canonical_view(old)), "{old}");
        }
        assert_eq!(canonical_view("keybinds"), "keybinds");
    }
}
