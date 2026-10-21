//! `breadhelp`: the BOS help window.
//!
//! A Tauri app (Rust here, Svelte in `frontend/`). Closing it quits, unless
//! the opt-in resident setting keeps it loaded and hidden for a faster
//! Super + / (see `commands::window`). The guided tour is a separate GTK4
//! binary, `breadhelp-tour`; `breadhelp_core::tourctl` is how the two meet.

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

    // Background-only calls decide here whether a window process is needed
    // at all. A running one gets the call through the single-instance
    // hand-off below; with none running, nothing should start just to hold
    // a banner, unload nothing, or idle after a login.
    let args_parsed = cli::parse(&args);
    let mut tour_started = false;
    if !window::window_pid().alive() {
        if args_parsed.unload {
            return;
        }
        if let Some(id) = &args_parsed.suggest {
            window::save_pending_suggestion(id);
            return;
        }
        if args_parsed.autostart && !setup::needs_setup() {
            if !State::load().onboarding_completed() {
                tourctl::spawn_tour(&["--start"]);
                tour_started = true;
            }
            if !State::load().resident() {
                return;
            }
        }
    }

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
            match screenshot_req {
                // No close routing or presence for a capture run: it is not
                // the user's window, and quitting it must not touch theirs.
                Some(req) => screenshot::dispatch(req, app),
                None => {
                    // Only the main instance gets here (a second launch hands
                    // off in the plugin above), so only it runs this. Never
                    // while a tour is up: a step may be holding a chained
                    // temporary bind on purpose.
                    if !tourctl::tour_running() {
                        tourctl::revert_pending_rebind();
                    }
                    window::route_close(&app);
                    window::presence_started();
                    window::take_pending_suggestion(&app);
                    let shown = handle(&app, &args_parsed);
                    settle_first_launch(&app, &args_parsed, shown, tour_started);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::theme::get_theme_css,
            commands::window::get_open_state,
            commands::window::dismiss_suggestion,
            commands::window::hide_window,
            commands::window::get_resident,
            commands::window::set_resident,
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

/// The checks in `main` decide most background-only calls before a window
/// exists, but they read a pid file that can be wrong (a window that just
/// quit, a stale file whose pid was reused). If this first launch ended up
/// with nothing to show and the resident setting is off, it must not linger
/// as an invisible 200 MB process: keep the suggestion for the next open,
/// start the tour autostart would have started, and quit.
///
/// `shown` is whether `handle` asked for the window: not its visibility,
/// which lags behind `show()` until the compositor maps the window.
fn settle_first_launch(app: &AppHandle, args: &cli::Args, shown: bool, tour_started: bool) {
    if shown || State::load().resident() {
        return;
    }
    if let Some(id) = &args.suggest {
        window::save_pending_suggestion(id);
    }
    if args.autostart && !tour_started && !setup::needs_setup() && !State::load().onboarding_completed() {
        tourctl::spawn_tour(&["--start"]);
    }
    window::quit(app);
}

/// One launch's worth of argv, first launch or forwarded. Returns whether
/// it showed the window.
fn handle(app: &AppHandle, args: &cli::Args) -> bool {
    if args.unload {
        window::unload(app);
        return false;
    }
    if let Some(id) = &args.suggest {
        if let Some(s) = breadd::resolve(id) {
            window::suggest(app, s.text);
        }
        // A background daemon event: never steal focus.
        return false;
    }
    if args.autostart {
        // Autostart is only ever visible on a genuine first run (setup, then
        // the tour). Reaching here otherwise means the resident setting is
        // on: stay loaded and hidden. `main` already started the tour.
        if setup::needs_setup() {
            window::show(app, "setup", true);
            return true;
        }
        return false;
    }
    // Super + / is a bare launch: the keybind reference, always.
    let page = args.page.as_deref().unwrap_or("keybinds");
    window::show(app, page, false);
    true
}
