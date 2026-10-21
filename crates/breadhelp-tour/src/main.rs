//! `breadhelp-tour`: the live guided tour overlay.
//!
//! Started by the help window (first run, or "Take the tour"), never by
//! hand. Arguments, any one per invocation:
//!
//! - `--start`: resume the tour from the saved step (first run).
//! - `--onboard`: restart from step 0.
//! - `--tour-event <id>`: a compositor event forwarded from
//!   `breadhelp-tour.lua` through `breadhelp`; a hard no-op unless the
//!   current step waits for exactly that id.
//!
//! Every invocation reaches the primary instance's argv (GApplication
//! `HANDLES_COMMAND_LINE` over D-Bus), so events land in the running tour.
//! The process exits when no tour is on screen.

mod markdown_render;
mod theme;
mod tour;

use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;

enum Action {
    Start,
    Restart,
    Event(String),
    None,
}

fn parse(args: &[std::ffi::OsString]) -> Action {
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        if arg == "--start" {
            return Action::Start;
        } else if arg == "--onboard" {
            return Action::Restart;
        } else if arg == "--tour-event" {
            if let Some(id) = it.next().and_then(|s| s.to_str()) {
                return Action::Event(id.to_string());
            }
        }
    }
    Action::None
}

fn main() {
    // Distinct from the help window's `com.breadway.breadhelp`, so the two
    // never claim each other's D-Bus name.
    let app = gtk4::Application::builder()
        .application_id("com.breadway.breadhelp.tour")
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|_| {
        // A previous run killed mid-step may have left a keybind rebound.
        tour::self_heal();
        theme::load();
    });

    app.connect_command_line(|app, cmdline| {
        match parse(&cmdline.arguments()) {
            Action::Start => tour::start(app),
            Action::Restart => tour::restart(app),
            Action::Event(id) => tour::on_tour_event(&id),
            Action::None => eprintln!("breadhelp-tour: expected --start, --onboard or --tour-event <id>"),
        }
        glib::ExitCode::SUCCESS
    });

    // Best-effort revert of a step's temporarily-rebound keybind on a clean
    // exit; `rebind_temp` is a short-lived `hyprctl` call.
    app.connect_shutdown(|_| {
        tour::self_heal();
        breadhelp_core::tourctl::remove_pid();
    });

    app.run();
}
