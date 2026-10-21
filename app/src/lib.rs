//! `breadhelp`: the BOS help window.
//!
//! A Tauri app (Rust here, Svelte in `frontend/`) that stays resident and
//! hidden for the session so Super + / opens instantly. The guided tour is a
//! separate GTK4 binary, `breadhelp-tour`; `breadhelp_core::tourctl` is how
//! the two meet.

mod cli;
mod commands;
mod listen;
mod screenshot;

use breadhelp_core::config::State;
use breadhelp_core::services::breadd;
use breadhelp_core::{setup, tourctl};
use tauri::AppHandle;

use commands::window::{self, UiState};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    // These never need a window. `--tour-event` in particular fires on every
    // window, workspace and layer change (breadhelp-tour.lua), so it must
    // stay a cheap no-op when no tour is running.
    match cli::early(&args) {
        Some(cli::Early::Listen) => return listen::run(),
        Some(cli::Early::TourEvent(id)) => {
            if tourctl::tour_running() {
                tourctl::spawn_tour(&["--tour-event", &id]);
            }
            return;
        }
        Some(cli::Early::Onboard) => return tourctl::spawn_tour(&["--onboard"]),
        None => {}
    }

    // A previous tour killed mid-step may have left a keybind rebound.
    tourctl::revert_pending_rebind();

    // GTK3 takes the Wayland app-id from the program name, which would be
    // the bare binary name. BOS's Hyprland window rule (float, size) and the
    // .desktop file's StartupWMClass both match `com.breadway.breadhelp`, the
    // app-id the GTK4 help window always had.
    glib::set_prgname(Some("com.breadway.breadhelp"));

    let screenshot_req = screenshot::parse(&args);
    let mut builder = tauri::Builder::default();
    // Registered first, per the plugin's docs, so it owns the second-launch
    // hand-off. A screenshot run must not hand off to the user's real window.
    if screenshot_req.is_none() {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            handle(app, &cli::parse(&argv));
        }));
    }

    builder
        .manage(UiState::default())
        .setup(move |app| {
            let app = app.handle().clone();
            commands::theme::watch_and_emit(&app);
            window::hide_on_close(&app);
            match screenshot_req {
                Some(req) => screenshot::dispatch(req, app),
                None => handle(&app, &cli::parse(&args)),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::theme::get_theme_css,
            commands::window::get_open_state,
            commands::window::dismiss_suggestion,
            commands::window::hide_window,
            commands::content::list_guides,
            commands::content::get_guide,
            commands::content::get_keybinds,
            commands::content::list_fixes,
            commands::content::list_commands,
            commands::content::list_symptoms,
            commands::content::get_prefs,
            commands::actions::run_fix,
            commands::actions::run_symptom_fix,
            commands::actions::run_guide_action,
            commands::actions::run_gallery_command,
            commands::actions::try_keybind,
            commands::actions::rebind_keybind,
            commands::actions::set_mode,
            commands::actions::open_settings,
            commands::actions::report_problem,
            commands::actions::start_tour,
            commands::setup::get_setup,
            commands::setup::set_setup_step,
            commands::setup::apply_timezone,
            commands::setup::open_network_tool,
            commands::setup::open_updates_tool,
            commands::setup::create_snapshot,
            commands::setup::finish_setup,
        ])
        .run(tauri::generate_context!())
        .expect("breadhelp: failed to start the help window");
}

/// One launch's worth of argv, first launch or forwarded.
fn handle(app: &AppHandle, args: &cli::Args) {
    if let Some(id) = &args.suggest {
        if let Some(s) = breadd::resolve(id) {
            window::suggest(app, s.text);
        }
        // A background daemon event: never steal focus.
        return;
    }
    if args.autostart {
        // Autostart is only ever visible on a genuine first run: setup, then
        // the tour. Every later login just keeps the window warm.
        if setup::needs_setup() {
            window::show(app, "setup", true);
        } else if !State::load().onboarding_completed() {
            tourctl::spawn_tour(&["--start"]);
        }
        return;
    }
    // Super + / is a bare launch: the keybind reference, always.
    let page = args.page.as_deref().unwrap_or("keybinds");
    window::show(app, page, false);
}
