use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GBox, Orientation, Stack, StackTransitionType};

use crate::cli::Action;
use crate::config::State;
use crate::content::keybinds::Keybind;
use crate::content::{keybinds, ContentStore};

use super::home::Home;
use super::{ask, day_zero, learn, modes, tabs, tour};

const DEFAULT_TAB: &str = "home";

struct Handle {
    window: ApplicationWindow,
    home: Home,
    stack: Stack,
    store: Rc<ContentStore>,
    binds: Rc<Vec<Keybind>>,
}

thread_local! {
    static HANDLE: RefCell<Option<Handle>> = const { RefCell::new(None) };
}

/// Called from every `command_line` invocation, including ones forwarded
/// over D-Bus to this primary instance by a second `breadhelp` launch, so
/// it must build the window at most once and reuse it thereafter.
pub fn present(app: &Application, action: Action) {
    // Crash-recover keybinds even if this invocation never builds a window
    // (silent every-login autostart).
    tour::self_heal();

    // Once onboarding is done, autostart only needs a warm process so
    // Super+/ is instant. GTK/theme/font load is most of that cost and is
    // paid by staying resident (~14 MiB PSS, no window). Building tabs
    // adds ~2 MiB of dead weight for a session that may never open help.
    let is_bare_silent_autostart = action.autostart
        && action.screenshot.is_none()
        && !action.force_onboard
        && action.tour_event.is_none()
        && action.suggest.is_none()
        && State::load().onboarding_completed();
    if is_bare_silent_autostart {
        // No window means nothing holds GApplication open. Forget the hold
        // guard so the primary instance lives for the session; logout is
        // what ends it. Without this, the next Super+/ would cold-start.
        std::mem::forget(app.hold());
        return;
    }

    HANDLE.with(|cell| {
        {
            let mut cell_ref = cell.borrow_mut();
            if cell_ref.is_none() {
                *cell_ref = Some(build(app));
            }
        }
        let cell_ref = cell.borrow();
        let handle = cell_ref.as_ref().unwrap();
        let display = WidgetExt::display(&handle.window);

        if let Some(req) = action.screenshot_request() {
            crate::screenshot::dispatch(&handle.window, &handle.stack, req);
            return;
        }

        if action.force_onboard {
            tour::restart(&display);
            return;
        }

        if let Some(id) = &action.tour_event {
            tour::on_tour_event(id);
            return;
        }

        if let Some(id) = &action.suggest {
            if let Some(s) = crate::services::breadd::resolve(id) {
                handle.home.set_suggestion(Some(&s.text));
            }
            // Background daemon event: never steal focus. A `--suggest`
            // can now be the first invocation of the session because
            // silent autostart no longer builds a window.
            return;
        }

        // Bare silent autostart (onboarding done) already returned above.
        // Autostart that reaches here is a genuine first run.
        let onboarded = State::load().onboarding_completed();
        if action.autostart && onboarded {
            return;
        }

        if day_zero::needs_wizard() {
            let stack = handle.stack.clone();
            let display = display.clone();
            day_zero::attach(&handle.stack, &handle.store, &handle.binds, move || {
                // Drop the Welcome tab after the click handler returns;
                // removing it here would destroy the Skip/Finish button
                // mid-signal.
                let stack = stack.clone();
                let display = display.clone();
                glib::idle_add_local_once(move || {
                    day_zero::detach(&stack);
                    if !State::load().onboarding_completed() {
                        tour::start(&display);
                    }
                });
            });
            show_window(&handle.window, action.autostart);
            return;
        }

        if action.autostart && !onboarded {
            tour::start(&display);
            return;
        }
        show_window(&handle.window, action.autostart);
    });
}

/// Present the help window and publish `bread.help.opened`. Screenshot
/// mode calls `window.present()` itself and must not go through here;
/// that is capture, not a user-visible open.
fn show_window(window: &ApplicationWindow, autostart: bool) {
    window.present();
    crate::services::breadd::emit_opened(autostart);
}

fn build(app: &Application) -> Handle {
    // First thing on every cold start: revert any keybind a previous,
    // crashed run may have left temporarily rebound mid tour-step (see
    // `ui::tour`'s crash-safety notes) before the user could be surprised
    // by it firing again.
    tour::self_heal();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("BOS Help")
        .default_width(880)
        .default_height(600)
        .build();

    crate::theme::load(&WidgetExt::display(&window));
    crate::theme::bind_window(&window);

    let store = Rc::new(ContentStore::load());
    let binds = Rc::new(keybinds::load());
    let state = State::load();

    modes::apply(&window, state.mode());

    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(StackTransitionType::SlideLeftRight);

    let home = super::home::build(&binds, &window, state.mode(), {
        let window = window.clone();
        move |mode| {
            modes::apply(&window, mode);
            let mut state = State::load();
            state.set_mode(mode);
        }
    });
    stack.add_titled(&home.root, Some("home"), "Home");
    stack.add_titled(
        &learn::build(&store, &binds, state.mode()),
        Some("learn"),
        "Learn",
    );
    stack.add_titled(
        &ask::build(store.clone(), binds.clone(), state.mode()),
        Some("ask"),
        "Ask",
    );
    stack.set_visible_child_name(DEFAULT_TAB);

    let switcher = tabs::build(&stack);

    let content_vbox = GBox::new(Orientation::Vertical, 0);
    content_vbox.append(&switcher);
    content_vbox.append(&stack);

    window.set_child(Some(&content_vbox));
    // Deliberately not presented here: `present()` (the caller) decides
    // whether this initial build should actually be shown, so a from-cold
    // every-login autostart with onboarding already complete builds a
    // ready-but-hidden window instead of flashing it open. First run
    // presents the window with the day-zero wizard; the tour overlay
    // starts after that finishes.

    Handle {
        window,
        home,
        stack,
        store,
        binds,
    }
}
