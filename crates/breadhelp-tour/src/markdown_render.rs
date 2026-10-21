//! GTK rendering of `breadhelp_core::content::markdown`'s block tree, for
//! the tour callout's body text. The help window renders the same tree in
//! the webview.

use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, Label, Orientation};

use breadhelp_core::content::keybinds::{self, Keybind};
use breadhelp_core::content::keys;
use breadhelp_core::content::markdown::{inline, Block, Span};
use breadhelp_core::services::exec;

pub fn render(blocks: &[Block], binds: &[Keybind]) -> gtk4::Widget {
    let vbox = GBox::new(Orientation::Vertical, 10);
    for block in blocks {
        match block {
            Block::Heading(_, text) => {
                let lbl = Label::new(Some(text));
                lbl.add_css_class("tour-heading");
                lbl.set_xalign(0.0);
                lbl.set_wrap(true);
                vbox.append(&lbl);
            }
            Block::Paragraph(spans) => vbox.append(&render_spans(spans, binds)),
            Block::Bullet(spans) => {
                let row = GBox::new(Orientation::Horizontal, 6);
                row.append(&Label::new(Some("\u{2022}")));
                row.append(&render_spans(spans, binds));
                vbox.append(&row);
            }
        }
    }
    vbox.upcast()
}

fn render_spans(spans: &[Span], binds: &[Keybind]) -> gtk4::Widget {
    let row = GBox::new(Orientation::Horizontal, 6);
    for span in spans {
        match span {
            Span::Text(t) => {
                if t.trim().is_empty() {
                    continue;
                }
                let lbl = Label::new(None);
                lbl.set_markup(&pango_markup(t));
                lbl.set_wrap(true);
                // `set_wrap` alone only wraps once something constrains the
                // label's width; a freestanding layer-shell window has
                // nothing to allocate against, so without this a long
                // paragraph reports its unwrapped width and the callout
                // balloons to fit it.
                lbl.set_max_width_chars(44);
                lbl.set_xalign(0.0);
                lbl.add_css_class("tour-body");
                row.append(&lbl);
            }
            Span::Run { command, label } => {
                let btn = Button::with_label(label);
                let cmd = command.clone();
                btn.connect_clicked(move |_| exec::run(&cmd));
                row.append(&btn);
            }
            Span::Terminal { argv, label } => {
                // No shell: the words go straight to the terminal. An empty
                // command renders as plain text, so a typo can never spawn a
                // bare terminal.
                if argv.is_empty() {
                    let lbl = Label::new(Some(label.as_str()));
                    lbl.set_xalign(0.0);
                    row.append(&lbl);
                } else {
                    let btn = Button::with_label(label);
                    let argv = argv.clone();
                    btn.connect_clicked(move |_| exec::run_terminal(&argv));
                    row.append(&btn);
                }
            }
            Span::Keybind(combo) => row.append(&keycaps(binds, combo)),
        }
    }
    row.upcast()
}

/// One keycap per key. Uses the bound combo when the guide's text matches a
/// real bind, else the guide's own text.
pub fn keycaps(binds: &[Keybind], combo: &str) -> gtk4::Widget {
    let caps = keybinds::find(binds, combo).map(|k| k.caps.clone()).unwrap_or_else(|| keys::display_text_combo(combo));
    let hbox = GBox::new(Orientation::Horizontal, 4);
    hbox.set_valign(gtk4::Align::Center);
    for cap in caps {
        let lbl = Label::new(Some(&cap));
        lbl.add_css_class("keycap");
        hbox.append(&lbl);
    }
    hbox.upcast()
}

fn pango_markup(text: &str) -> String {
    let mut out = String::new();
    for run in inline(text) {
        let escaped = glib::markup_escape_text(&run.text);
        match (run.bold, run.code) {
            (true, true) => out.push_str(&format!("<b><tt>{escaped}</tt></b>")),
            (true, false) => out.push_str(&format!("<b>{escaped}</b>")),
            (false, true) => out.push_str(&format!("<tt>{escaped}</tt>")),
            (false, false) => out.push_str(&escaped),
        }
    }
    out
}
