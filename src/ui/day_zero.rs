//! In-window day-zero wizard for first boot / `--autostart`.
//!
//! Not a second app: this is one extra tab on the existing help window so
//! Learn/Ask stay reachable when bakery `content.tar.gz` is installed.
//! Existing users who already finished the desktop tour never see it
//! (`needs_wizard` is false when `onboarding.completed` is set).
//!
//! Finish (or skip) hands off to the live tour — the previous first-run
//! destination — with the Home cheatsheet already showing underneath.

use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GBox, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
    SearchEntry, SelectionMode, Stack,
};

use crate::config::State;
use crate::content::keybinds::Keybind;
use crate::content::{markdown, ContentStore};
use crate::services::exec;

const PAGE_NAME: &str = "day-zero";
const STEP_COUNT: usize = 6;
const WELCOME_GUIDE_CATEGORY: &str = "getting-started";
const WELCOME_GUIDE_ID: &str = "01-what-is-bos";
const FALLBACK_WELCOME: &str = "\
# Welcome to BOS

BOS (the Bread Operating System) is a complete Hyprland desktop with the \
bread ecosystem preinstalled: a bar, a launcher, notes, Wi-Fi profiles, and a \
settings app that needs no config files.

This short setup gets the clock, network, and updates in place. Use the tabs \
above any time to browse the full guide.
";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Welcome,
    Timezone,
    Network,
    Updates,
    Snapshot,
    Finish,
}

const STEPS: [Step; STEP_COUNT] = [
    Step::Welcome,
    Step::Timezone,
    Step::Network,
    Step::Updates,
    Step::Snapshot,
    Step::Finish,
];

struct Wizard {
    content: GBox,
    welcome_markdown: String,
    binds: Vec<Keybind>,
    step: RefCell<usize>,
    finished: Cell<bool>,
    on_finished: Box<dyn Fn()>,
}

/// First boot only: skip when the wizard already ran *or* the user already
/// finished the older tour-only onboarding (so upgrades don't re-prompt).
pub fn needs_wizard() -> bool {
    needs_wizard_from(
        State::load().day_zero_completed(),
        State::load().onboarding_completed(),
    )
}

pub(crate) fn needs_wizard_from(day_zero_done: bool, onboarding_done: bool) -> bool {
    !day_zero_done && !onboarding_done
}

/// Add (or focus) the Welcome tab and show it. Idempotent across D-Bus
/// re-activation while the wizard is already up.
pub fn attach(
    stack: &Stack,
    store: &ContentStore,
    binds: &[Keybind],
    on_finished: impl Fn() + 'static,
) {
    if stack.child_by_name(PAGE_NAME).is_some() {
        stack.set_visible_child_name(PAGE_NAME);
        return;
    }

    let start = (State::load().day_zero_step() as usize).min(STEP_COUNT - 1);
    let welcome_markdown = store
        .guide(WELCOME_GUIDE_CATEGORY, WELCOME_GUIDE_ID)
        .map(|g| store.body(g, State::load().mode().is_simplified()))
        .filter(|body| !body.trim().is_empty())
        .unwrap_or_else(|| FALLBACK_WELCOME.to_string());

    let content = GBox::new(Orientation::Vertical, 16);
    content.add_css_class("view-content");
    content.add_css_class("day-zero");

    let wizard = Rc::new(Wizard {
        content: content.clone(),
        welcome_markdown,
        binds: binds.to_vec(),
        step: RefCell::new(start),
        finished: Cell::new(false),
        on_finished: Box::new(on_finished),
    });
    wizard.render();

    let page = ScrolledWindow::builder()
        .child(&content)
        .vexpand(true)
        .hexpand(true)
        .build();
    stack.add_titled(&page, Some(PAGE_NAME), "Welcome");
    stack.set_visible_child_name(PAGE_NAME);
}

pub(crate) fn pick_network_tool(
    has_settings: bool,
    has_nm: bool,
) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    if has_settings {
        Some((
            "Open network settings",
            "bos-settings",
            &["--page", "network"],
        ))
    } else if has_nm {
        Some(("Open network editor", "nm-connection-editor", &[]))
    } else {
        None
    }
}

pub(crate) fn pick_updates_tool(
    has_settings: bool,
) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    if has_settings {
        Some(("Open BOS Settings", "bos-settings", &["--page", "packages"]))
    } else {
        None
    }
}

pub(crate) fn is_safe_timezone_name(tz: &str) -> bool {
    !tz.is_empty()
        && tz.len() < 128
        && tz
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '+' | '-' | '.'))
}

fn parse_timezone_property(stdout: &str) -> Option<String> {
    stdout
        .trim()
        .strip_prefix("Timezone=")
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn parse_timezone_list(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

impl Wizard {
    fn current_step(&self) -> Step {
        STEPS[*self.step.borrow()]
    }

    fn go(self: &Rc<Self>, step: usize) {
        let step = step.min(STEP_COUNT - 1);
        *self.step.borrow_mut() = step;
        let mut state = State::load();
        state.set_day_zero_step(step as i64);
        self.render();
    }

    fn finish(self: &Rc<Self>) {
        if self.finished.replace(true) {
            return;
        }
        let mut state = State::load();
        state.set_day_zero_completed(true);
        (self.on_finished)();
    }

    fn clear(&self) {
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }
    }

    fn render(self: &Rc<Self>) {
        self.clear();
        let index = *self.step.borrow();
        let step = self.current_step();

        let title = Label::new(Some(step_title(step)));
        title.add_css_class("page-title");
        title.set_xalign(0.0);
        self.content.append(&title);

        match step {
            Step::Welcome => self.render_welcome(),
            Step::Timezone => self.render_timezone(),
            Step::Network => self.render_network(),
            Step::Updates => self.render_updates(),
            Step::Snapshot => self.render_snapshot(),
            Step::Finish => self.render_finish(),
        }

        let counter = Label::new(Some(&format!("Step {} of {STEP_COUNT}", index + 1)));
        counter.add_css_class("dim-label");
        counter.set_xalign(0.0);
        self.content.append(&counter);

        let nav = GBox::new(Orientation::Horizontal, 8);
        let skip = Button::with_label("Skip setup");
        skip.add_css_class("flat");
        let this = self.clone();
        skip.connect_clicked(move |_| this.finish());

        let back = Button::with_label("Back");
        back.set_sensitive(index > 0);
        let this = self.clone();
        back.connect_clicked(move |_| {
            let i = *this.step.borrow();
            if i > 0 {
                this.go(i - 1);
            }
        });

        let spacer = GBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let is_last = index + 1 >= STEP_COUNT;
        let next = Button::with_label(if is_last { "Start tour" } else { "Next" });
        next.add_css_class("suggested-action");
        let this = self.clone();
        next.connect_clicked(move |_| {
            let i = *this.step.borrow();
            if i + 1 >= STEP_COUNT {
                this.finish();
            } else {
                this.go(i + 1);
            }
        });

        nav.append(&skip);
        nav.append(&spacer);
        nav.append(&back);
        nav.append(&next);
        self.content.append(&nav);
    }

    fn render_welcome(self: &Rc<Self>) {
        let blurb = Label::new(Some(
            "A few things before the desktop tour. The other tabs stay available if you want to look around.",
        ));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);
        self.content.append(&markdown::render(
            &markdown::parse(&self.welcome_markdown),
            &self.binds,
        ));
    }

    fn render_timezone(self: &Rc<Self>) {
        let (current, zones) = load_timezones();
        let intro = if zones.is_empty() {
            "Couldn't list timezones (is timedatectl available?). You can set this later from BOS Settings."
        } else {
            "The clock should match where you are. Applying a change may ask for your password."
        };
        let blurb = Label::new(Some(intro));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);

        let current_lbl = Label::new(Some(&if current.is_empty() {
            "Current timezone: unknown".into()
        } else {
            format!("Current timezone: {current}")
        }));
        current_lbl.set_xalign(0.0);
        current_lbl.add_css_class("dim-label");
        self.content.append(&current_lbl);

        if zones.is_empty() {
            return;
        }

        let search = SearchEntry::new();
        search.set_placeholder_text(Some("Search timezones"));
        search.set_hexpand(true);
        self.content.append(&search);

        let list = ListBox::new();
        list.set_selection_mode(SelectionMode::Single);
        list.add_css_class("boxed-list");
        for tz in &zones {
            let row = ListBoxRow::new();
            row.set_widget_name(tz);
            let lbl = Label::new(Some(tz));
            lbl.set_xalign(0.0);
            lbl.set_margin_top(4);
            lbl.set_margin_bottom(4);
            lbl.set_margin_start(8);
            row.set_child(Some(&lbl));
            list.append(&row);
            if tz == &current {
                list.select_row(Some(&row));
            }
        }

        {
            let list = list.clone();
            search.connect_search_changed(move |entry| {
                let q = entry.text().to_lowercase();
                let mut i = 0;
                let mut first_visible: Option<ListBoxRow> = None;
                loop {
                    let Some(row) = list.row_at_index(i) else {
                        break;
                    };
                    let name = row.widget_name();
                    let visible = q.is_empty() || name.to_lowercase().contains(&q);
                    row.set_visible(visible);
                    if visible && first_visible.is_none() {
                        first_visible = Some(row);
                    }
                    i += 1;
                }
                if let Some(row) = first_visible {
                    list.select_row(Some(&row));
                }
            });
        }

        let scroller = ScrolledWindow::builder()
            .child(&list)
            .min_content_height(180)
            .hexpand(true)
            .vexpand(true)
            .build();
        self.content.append(&scroller);

        let status = Label::new(None);
        status.set_xalign(0.0);
        status.add_css_class("dim-label");

        let apply = Button::with_label("Apply timezone");
        apply.set_halign(Align::Start);
        apply.add_css_class("suggested-action");
        let current_lbl = current_lbl.clone();
        let status_for_click = status.clone();
        apply.connect_clicked(move |_| {
            let Some(row) = list.selected_row() else {
                status_for_click.set_label("Pick a timezone first.");
                return;
            };
            let tz = row.widget_name().to_string();
            if !is_safe_timezone_name(&tz) {
                status_for_click.set_label("That timezone name doesn't look valid.");
                return;
            }
            status_for_click.set_label("Applying\u{2026}");
            let status = status_for_click.clone();
            let current_lbl = current_lbl.clone();
            apply_timezone(tz, move |msg, ok, tz| {
                status.set_label(&msg);
                if ok {
                    current_lbl.set_label(&format!("Current timezone: {tz}"));
                }
            });
        });
        self.content.append(&apply);
        self.content.append(&status);
    }

    fn render_network(self: &Rc<Self>) {
        let blurb = Label::new(Some(
            "Connect to Wi-Fi in BOS Settings (or the NetworkManager editor). This window does not manage networks itself.",
        ));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);

        if let Some((label, prog, args)) = pick_network_tool(
            exec::command_exists("bos-settings"),
            exec::command_exists("nm-connection-editor"),
        ) {
            let btn = Button::with_label(label);
            btn.set_halign(Align::Start);
            let prog = prog.to_string();
            let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
            btn.connect_clicked(move |_| {
                let argv: Vec<&str> = args.iter().map(String::as_str).collect();
                exec::run_argv(&prog, &argv);
            });
            self.content.append(&btn);
        } else {
            let hint = Label::new(Some(
                "No network UI found. Run `breadcrumbs` or `nmcli` in a terminal to connect.",
            ));
            hint.set_wrap(true);
            hint.set_xalign(0.0);
            hint.add_css_class("dim-label");
            self.content.append(&hint);
        }

        if exec::command_exists("breadcrumbs") {
            let crumbs = Label::new(Some(
                "Location profiles (home / work / away) are handled by breadcrumbs once a network is saved.",
            ));
            crumbs.set_wrap(true);
            crumbs.set_xalign(0.0);
            crumbs.add_css_class("dim-label");
            self.content.append(&crumbs);
        }
    }

    fn render_updates(self: &Rc<Self>) {
        let blurb = Label::new(Some(
            "When you're online, check for system and bread-ecosystem updates. Nothing is installed from this step.",
        ));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);

        if let Some((label, prog, args)) = pick_updates_tool(exec::command_exists("bos-settings")) {
            let btn = Button::with_label(label);
            btn.set_halign(Align::Start);
            let prog = prog.to_string();
            let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
            btn.connect_clicked(move |_| {
                let argv: Vec<&str> = args.iter().map(String::as_str).collect();
                exec::run_argv(&prog, &argv);
            });
            self.content.append(&btn);
        }

        let hint = Label::new(Some(if exec::command_exists("bos-settings") {
            "Or run `bos-update` in a terminal."
        } else {
            "BOS Settings isn't installed. Run `bos-update` in a terminal when you're ready."
        }));
        hint.set_wrap(true);
        hint.set_xalign(0.0);
        hint.add_css_class("dim-label");
        self.content.append(&hint);
    }

    fn render_snapshot(self: &Rc<Self>) {
        let available = exec::command_exists("snapper");
        let blurb = Label::new(Some(if available {
            "A snapshot is a restore point. We'll try to create one named day-zero — if this machine doesn't allow it, that's fine."
        } else {
            "snapper isn't available on this system, so no first-boot snapshot will be created. You can skip this."
        }));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);

        let status = Label::new(None);
        status.set_xalign(0.0);
        status.add_css_class("dim-label");

        if available {
            let retry = Button::with_label("Create snapshot");
            retry.set_halign(Align::Start);
            let status_click = status.clone();
            retry.connect_clicked(move |_| {
                status_click.set_label("Creating snapshot\u{2026}");
                let status = status_click.clone();
                create_day_zero_snapshot(move |ok| {
                    status.set_label(snapshot_status(ok));
                });
            });
            self.content.append(&retry);

            if !State::load().day_zero_snapshot_attempted() {
                let mut state = State::load();
                state.set_day_zero_snapshot_attempted(true);
                status.set_label("Creating snapshot\u{2026}");
                let status = status.clone();
                create_day_zero_snapshot(move |ok| {
                    status.set_label(snapshot_status(ok));
                });
            }
        }

        self.content.append(&status);
    }

    fn render_finish(self: &Rc<Self>) {
        let blurb = Label::new(Some(
            "You're set. Next is a short tour of the real desktop. Super+/ opens this help center any time — the Home tab is the keybind cheatsheet.",
        ));
        blurb.set_wrap(true);
        blurb.set_xalign(0.0);
        self.content.append(&blurb);
    }
}

fn step_title(step: Step) -> &'static str {
    match step {
        Step::Welcome => "Welcome to BOS",
        Step::Timezone => "Timezone",
        Step::Network => "Network",
        Step::Updates => "Updates",
        Step::Snapshot => "Snapshot",
        Step::Finish => "You're ready",
    }
}

fn snapshot_status(ok: bool) -> &'static str {
    if ok {
        "Created snapshot “day-zero”."
    } else {
        "Couldn't create a snapshot (that's OK — you can do this later from Settings)."
    }
}

fn load_timezones() -> (String, Vec<String>) {
    let current = Command::new("timedatectl")
        .args(["show", "--property=Timezone"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| parse_timezone_property(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    let zones = Command::new("timedatectl")
        .arg("list-timezones")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_timezone_list(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    (current, zones)
}

fn apply_timezone(tz: String, on_done: impl Fn(String, bool, String) + 'static) {
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let direct = Command::new("timedatectl")
            .args(["set-timezone", &tz])
            .status();
        if direct.map(|s| s.success()).unwrap_or(false) {
            let _ = tx.send_blocking((format!("Timezone set to {tz}."), true, tz));
            return;
        }
        if !exec::command_exists("pkexec") {
            let _ = tx.send_blocking((
                "Need permission to change the timezone, and pkexec isn't available.".into(),
                false,
                tz,
            ));
            return;
        }
        let elevated = Command::new("pkexec")
            .args(["timedatectl", "set-timezone", &tz])
            .status();
        if elevated.map(|s| s.success()).unwrap_or(false) {
            let _ = tx.send_blocking((format!("Timezone set to {tz}."), true, tz));
        } else {
            let _ = tx.send_blocking((
                "Couldn't change the timezone. You can try again from BOS Settings.".into(),
                false,
                tz,
            ));
        }
    });
    glib::MainContext::default().spawn_local(async move {
        if let Ok((msg, ok, tz)) = rx.recv().await {
            on_done(msg, ok, tz);
        }
    });
}

fn create_day_zero_snapshot(on_done: impl Fn(bool) + 'static) {
    exec::run_argv_reporting(
        "snapper",
        &["-c", "root", "create", "-d", "day-zero"],
        on_done,
    );
}

/// Used by `window` after the wizard finishes: drop the Welcome tab so the
/// help center is just Home/Learn/Ask again.
pub fn detach(stack: &Stack) {
    if let Some(child) = stack.child_by_name(PAGE_NAME) {
        stack.remove(&child);
    }
    if stack.child_by_name("home").is_some() {
        stack.set_visible_child_name("home");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_onboarding_skips_wizard() {
        assert!(!needs_wizard_from(false, true));
        assert!(!needs_wizard_from(true, true));
        assert!(!needs_wizard_from(true, false));
        assert!(needs_wizard_from(false, false));
    }

    #[test]
    fn network_prefers_bos_settings_then_nm() {
        let (label, prog, args) = pick_network_tool(true, true).unwrap();
        assert_eq!(prog, "bos-settings");
        assert_eq!(args, &["--page", "network"]);
        assert!(label.contains("network"));
        let (_, prog, _) = pick_network_tool(false, true).unwrap();
        assert_eq!(prog, "nm-connection-editor");
        assert!(pick_network_tool(false, false).is_none());
    }

    #[test]
    fn updates_tool_only_when_settings_exists() {
        assert!(pick_updates_tool(true).is_some());
        assert!(pick_updates_tool(false).is_none());
    }

    #[test]
    fn timezone_names_reject_shell_metacharacters() {
        assert!(is_safe_timezone_name("Australia/Perth"));
        assert!(is_safe_timezone_name("Etc/GMT+8"));
        assert!(is_safe_timezone_name("America/Argentina/ComodRivadavia"));
        assert!(!is_safe_timezone_name(""));
        assert!(!is_safe_timezone_name("America/New York"));
        assert!(!is_safe_timezone_name("x; rm -rf /"));
        assert!(!is_safe_timezone_name("$(reboot)"));
    }

    #[test]
    fn parses_timedatectl_show_and_list() {
        assert_eq!(
            parse_timezone_property("Timezone=Australia/Perth\n").as_deref(),
            Some("Australia/Perth")
        );
        assert_eq!(parse_timezone_property("Timezone=\n"), None);
        assert_eq!(
            parse_timezone_list("Africa/Abidjan\nAustralia/Perth\n\nUTC\n"),
            vec!["Africa/Abidjan", "Australia/Perth", "UTC"]
        );
    }
}
