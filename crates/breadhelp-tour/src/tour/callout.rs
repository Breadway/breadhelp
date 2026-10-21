//! The floating explanation card: progress dots, title, body (rendered
//! through the same constrained markdown subset guides use, so
//! `[Show Keybind]`/`[Run]` tags work here too), then whichever of the
//! "waiting for you", "Show me" and "done" rows the step's state calls for,
//! and the Skip/Back/Next row. Positioned just below (or, if that would run
//! off the bottom of the screen, above) the spotlighted target; a
//! `Target::None` step gets no anchors at all, which wlr-layer-shell centers
//! on the monitor by convention.

use gtk4::prelude::*;
use gtk4::{Align, Box as GBox, Button, Label, Orientation, Window};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use breadhelp_core::content::keybinds::Keybind;
use breadhelp_core::content::markdown;
use breadhelp_core::content::tour::{Step, Success, DEFAULT_CONFIRMED_TEXT};

use super::target::Rect;
use crate::markdown_render;

pub struct Callout {
    pub window: Window,
    pub back_btn: Option<Button>,
    pub next_btn: Button,
    pub skip_btn: Option<Button>,
    pub demo_btn: Option<Button>,
}

const CALLOUT_WIDTH: i32 = 400;
/// Rough height estimate used only to decide "does this fit below the
/// target" before the widget has actually been laid out; doesn't need to
/// be exact, just enough to avoid running off the bottom of the screen.
const ESTIMATED_HEIGHT: i32 = 260;

/// Everything about the step's progress the card needs to draw itself.
pub struct View<'a> {
    pub step: &'a Step,
    pub binds: &'a [Keybind],
    pub index: usize,
    pub total: usize,
    pub confirmed: bool,
    pub show_demo: bool,
}

impl View<'_> {
    fn is_last(&self) -> bool {
        self.index + 1 >= self.total
    }

    /// A must-do step the user hasn't done yet: Next is demoted to "Skip
    /// this step" until the compositor confirms it, so the one hands-on step
    /// isn't skipped past by reflex.
    fn awaiting(&self) -> bool {
        matches!(self.step.success(), Success::Event(_)) && !self.confirmed
    }

    fn next_label(&self) -> &'static str {
        if self.is_last() {
            "Finish"
        } else if self.awaiting() {
            "Skip this step"
        } else if self.index == 0 {
            "Let's go"
        } else {
            "Next"
        }
    }
}

pub fn build(monitor: &gdk4::Monitor, rect: Option<Rect>, monitor_size: (i32, i32), view: &View) -> Callout {
    let window = Window::new();
    window.set_decorated(false);
    window.add_css_class("tour-window");
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_monitor(Some(monitor));
    crate::theme::bind_window(&window);

    if let Some(rect) = rect {
        let (mon_w, mon_h) = monitor_size;
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Top, true);
        // Centered under the target when there's room, so a wide target
        // (the bar) gets the card under its middle rather than its far edge.
        let centered = rect.x + (rect.w - CALLOUT_WIDTH) / 2;
        let margin_left = centered.clamp(20, (mon_w - CALLOUT_WIDTH - 20).max(20));
        let below = rect.y + rect.h + 14;
        let margin_top = if below + ESTIMATED_HEIGHT <= mon_h {
            below
        } else {
            (rect.y - 14 - ESTIMATED_HEIGHT).max(20)
        };
        window.set_margin(Edge::Left, margin_left);
        window.set_margin(Edge::Top, margin_top);
    }
    // No anchors for a `Target::None` step: left to the compositor, which
    // centers unanchored layer surfaces.

    let root = GBox::new(Orientation::Vertical, 12);
    root.add_css_class("tour-callout");
    root.set_size_request(CALLOUT_WIDTH, -1);

    root.append(&progress_row(view.index, view.total));

    let title = Label::new(Some(&view.step.title));
    title.add_css_class("tour-title");
    title.set_xalign(0.0);
    title.set_wrap(true);
    root.append(&title);

    root.append(&markdown_render::render(&markdown::parse(&view.step.body), view.binds));

    if !view.step.shortcuts.is_empty() {
        let list = GBox::new(Orientation::Vertical, 8);
        list.add_css_class("tour-panel");
        for shortcut in &view.step.shortcuts {
            let row = GBox::new(Orientation::Horizontal, 10);
            let lbl = Label::new(Some(&shortcut.label));
            lbl.set_xalign(0.0);
            lbl.set_hexpand(true);
            row.append(&lbl);
            row.append(&markdown_render::keycaps(view.binds, &shortcut.keybind));
            list.append(&row);
        }
        root.append(&list);
    }

    if view.awaiting() {
        let row = GBox::new(Orientation::Horizontal, 10);
        row.add_css_class("tour-panel");
        let dot = GBox::new(Orientation::Horizontal, 0);
        dot.add_css_class("tour-live");
        dot.set_valign(Align::Center);
        row.append(&dot);
        let text = if view.step.keybind.is_some() {
            "Your turn. Waiting for you to press"
        } else {
            "Your turn. Waiting for you to do it"
        };
        let lbl = Label::new(Some(text));
        lbl.add_css_class("tour-dim");
        lbl.set_xalign(0.0);
        lbl.set_hexpand(true);
        lbl.set_wrap(true);
        row.append(&lbl);
        if let Some(combo) = &view.step.keybind {
            row.append(&markdown_render::keycaps(view.binds, combo));
        }
        root.append(&row);
    }

    let demo_btn = if view.show_demo {
        let hint_row = GBox::new(Orientation::Horizontal, 10);
        let hint_label = Label::new(Some("Still there? We can open it for you."));
        hint_label.add_css_class("tour-dim");
        hint_label.set_hexpand(true);
        hint_label.set_xalign(0.0);
        hint_label.set_wrap(true);
        let btn = Button::with_label("Show me");
        hint_row.append(&hint_label);
        hint_row.append(&btn);
        root.append(&hint_row);
        Some(btn)
    } else {
        None
    };

    if view.confirmed {
        let row = GBox::new(Orientation::Horizontal, 10);
        row.add_css_class("tour-confirmed");
        let check = Label::new(Some("\u{2713}"));
        check.add_css_class("tour-check");
        check.set_valign(Align::Center);
        row.append(&check);
        let text = view.step.confirmed_text.as_deref().unwrap_or(DEFAULT_CONFIRMED_TEXT);
        let lbl = Label::new(Some(text));
        lbl.set_xalign(0.0);
        lbl.set_hexpand(true);
        lbl.set_wrap(true);
        lbl.set_max_width_chars(40);
        row.append(&lbl);
        root.append(&row);
    }

    let nav = GBox::new(Orientation::Horizontal, 8);
    nav.set_margin_top(4);
    let skip_btn = (!view.is_last()).then(|| {
        let btn = Button::with_label("Skip tour");
        btn.add_css_class("flat");
        nav.append(&btn);
        btn
    });
    let spacer = GBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    nav.append(&spacer);
    let back_btn = (view.index > 0).then(|| {
        let btn = Button::with_label("Back");
        nav.append(&btn);
        btn
    });
    let next_btn = Button::with_label(view.next_label());
    if !view.awaiting() {
        next_btn.add_css_class("suggested-action");
    }
    nav.append(&next_btn);
    root.append(&nav);

    window.set_child(Some(&root));
    window.present();
    if view.confirmed {
        next_btn.grab_focus();
    }

    Callout { window, back_btn, next_btn, skip_btn, demo_btn }
}

/// Dots (done, current, to come) with the plain "Step N of M" beside them.
fn progress_row(index: usize, total: usize) -> GBox {
    let row = GBox::new(Orientation::Horizontal, 6);
    for i in 0..total {
        let dot = GBox::new(Orientation::Horizontal, 0);
        dot.add_css_class("tour-dot");
        dot.set_valign(Align::Center);
        if i == index {
            dot.add_css_class("current");
        } else if i < index {
            dot.add_css_class("done");
        }
        row.append(&dot);
    }
    let spacer = GBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    row.append(&spacer);
    let counter = Label::new(Some(&format!("Step {} of {total}", index + 1)));
    counter.add_css_class("tour-dim");
    row.append(&counter);
    row
}
