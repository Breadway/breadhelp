//! Constrained-subset markdown parser for guide `content.md`.
//!
//! Deliberately not a full CommonMark parser (`pulldown-cmark`/`comrak`): a
//! full parser still wouldn't understand the `[Run]`/`[Show Keybind]` tags,
//! and this content is authored in-repo by BOS maintainers. Parsing to this
//! block/span tree (never to HTML) is what lets the help window render guides
//! as plain text nodes and buttons: guide text never reaches the webview as
//! markup. The tour overlay renders the same tree with GTK widgets.
//!
//! Supported: `#`/`##` headings, one-block-per-line paragraphs, `- ` bullets,
//! inline `**bold**`/`` `code` `` (see [`inline`]), and three custom tags:
//! `[Run]<command>[/Run]` (optional `|Label` suffix, run through a shell),
//! `[Terminal]<command>[/Terminal]` (optional `|Label` suffix, split on
//! whitespace and run in the BOS terminal with no shell, for terminal-only
//! commands such as `breadbug`), and `[Show Keybind]<combo>[/Show Keybind]`.

#[derive(Debug, Clone, PartialEq)]
pub enum Span {
    Text(String),
    Run { command: String, label: String },
    Terminal { argv: Vec<String>, label: String },
    Keybind(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading(u8, String),
    Paragraph(Vec<Span>),
    Bullet(Vec<Span>),
}

/// Guides hard-wrap their source, so a paragraph or bullet runs on until a
/// blank line, a heading or the next bullet, and its lines join with a
/// space (a tag split across a line break still parses).
pub fn parse(src: &str) -> Vec<Block> {
    enum Open {
        Paragraph(String),
        Bullet(String),
    }
    fn close(open: Option<Open>, blocks: &mut Vec<Block>) {
        match open {
            Some(Open::Paragraph(t)) => blocks.push(Block::Paragraph(parse_spans(&t))),
            Some(Open::Bullet(t)) => blocks.push(Block::Bullet(parse_spans(&t))),
            None => {}
        }
    }

    let mut blocks = Vec::new();
    let mut open: Option<Open> = None;
    for raw_line in src.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            close(open.take(), &mut blocks);
        } else if let Some(rest) = line.strip_prefix("## ") {
            close(open.take(), &mut blocks);
            blocks.push(Block::Heading(2, rest.trim().to_string()));
        } else if let Some(rest) = line.strip_prefix("# ") {
            close(open.take(), &mut blocks);
            blocks.push(Block::Heading(1, rest.trim().to_string()));
        } else if let Some(rest) = line.strip_prefix("- ") {
            close(open.take(), &mut blocks);
            open = Some(Open::Bullet(rest.to_string()));
        } else {
            match &mut open {
                Some(Open::Paragraph(t) | Open::Bullet(t)) => {
                    t.push(' ');
                    t.push_str(line);
                }
                None => open = Some(Open::Paragraph(line.to_string())),
            }
        }
    }
    close(open, &mut blocks);
    blocks
}

fn parse_spans(text: &str) -> Vec<Span> {
    const RUN_OPEN: &str = "[Run]";
    const RUN_CLOSE: &str = "[/Run]";
    const TERM_OPEN: &str = "[Terminal]";
    const TERM_CLOSE: &str = "[/Terminal]";
    const KB_OPEN: &str = "[Show Keybind]";
    const KB_CLOSE: &str = "[/Show Keybind]";

    // (offset, kind): 0 = shell run, 1 = terminal run, 2 = keybind.
    let mut spans = Vec::new();
    let mut rest = text;
    loop {
        let run_idx = rest.find(RUN_OPEN).map(|i| (i, 0));
        let term_idx = rest.find(TERM_OPEN).map(|i| (i, 1));
        let kb_idx = rest.find(KB_OPEN).map(|i| (i, 2));
        let next = [run_idx, term_idx, kb_idx].into_iter().flatten().min();
        let Some((idx, kind)) = next else {
            if !rest.is_empty() {
                spans.push(Span::Text(rest.to_string()));
            }
            break;
        };
        if idx > 0 {
            spans.push(Span::Text(rest[..idx].to_string()));
        }
        let (open_len, close_tag) = match kind {
            0 => (RUN_OPEN.len(), RUN_CLOSE),
            1 => (TERM_OPEN.len(), TERM_CLOSE),
            _ => (KB_OPEN.len(), KB_CLOSE),
        };
        let after = &rest[idx + open_len..];
        let Some(end) = after.find(close_tag) else {
            // Unterminated tag: keep the literal text rather than dropping it.
            spans.push(Span::Text(rest[idx..].to_string()));
            break;
        };
        let inner = &after[..end];
        if kind == 2 {
            spans.push(Span::Keybind(inner.trim().to_string()));
        } else {
            let (command, label) = match inner.split_once('|') {
                Some((c, l)) => (c.trim().to_string(), l.trim().to_string()),
                None => (inner.trim().to_string(), inner.trim().to_string()),
            };
            if kind == 0 {
                spans.push(Span::Run { command, label });
            } else {
                spans.push(Span::Terminal {
                    argv: crate::services::exec::split_argv(&command),
                    label: if label.is_empty() { command } else { label },
                });
            }
        }
        rest = &after[end + close_tag.len()..];
    }
    spans
}

/// A run of text with one inline style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    pub text: String,
    pub bold: bool,
    pub code: bool,
}

/// Split a `Span::Text` into `**bold**` / `` `code` `` runs. Code wins over
/// bold (a `**` inside backticks stays literal), and an unclosed delimiter
/// keeps its literal text instead of styling the rest of the line.
pub fn inline(text: &str) -> Vec<Inline> {
    fn push(out: &mut Vec<Inline>, text: &str, bold: bool, code: bool) {
        if !text.is_empty() {
            out.push(Inline { text: text.to_string(), bold, code });
        }
    }
    let mut out: Vec<Inline> = Vec::new();
    let mut rest = text;
    let mut bold = false;
    loop {
        let tick = rest.find('`');
        let star = rest.find("**");
        match (tick, star) {
            (Some(t), s) if s.is_none_or(|s| t < s) => {
                let after = &rest[t + 1..];
                match after.find('`') {
                    Some(end) => {
                        push(&mut out, &rest[..t], bold, false);
                        push(&mut out, &after[..end], bold, true);
                        rest = &after[end + 1..];
                    }
                    None => {
                        push(&mut out, rest, bold, false);
                        break;
                    }
                }
            }
            (_, Some(s)) => {
                let closes_later = bold || rest[s + 2..].contains("**");
                if closes_later {
                    push(&mut out, &rest[..s], bold, false);
                    bold = !bold;
                    rest = &rest[s + 2..];
                } else {
                    push(&mut out, rest, bold, false);
                    break;
                }
            }
            _ => {
                push(&mut out, rest, bold, false);
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_lines_join_into_one_block() {
        let blocks = parse("One line\nand its wrap.\n\n- a bullet\n  that wraps\n- next\nSecond para");
        assert_eq!(
            blocks,
            vec![
                Block::Paragraph(vec![Span::Text("One line and its wrap.".into())]),
                Block::Bullet(vec![Span::Text("a bullet that wraps".into())]),
                Block::Bullet(vec![Span::Text("next Second para".into())]),
            ]
        );
    }

    #[test]
    fn a_tag_split_across_a_line_break_still_parses() {
        let blocks = parse("Press [Show Keybind]super +\nspace[/Show Keybind] now.");
        assert!(matches!(&blocks[0], Block::Paragraph(s) if s.contains(&Span::Keybind("super + space".into()))));
    }

    #[test]
    fn inline_styles_bold_and_code() {
        let runs = inline("Run `bos-update` to get **everything** new.");
        let shape: Vec<(&str, bool, bool)> = runs.iter().map(|r| (r.text.as_str(), r.bold, r.code)).collect();
        assert_eq!(
            shape,
            vec![
                ("Run ", false, false),
                ("bos-update", false, true),
                (" to get ", false, false),
                ("everything", true, false),
                (" new.", false, false),
            ]
        );
    }

    #[test]
    fn inline_leaves_unclosed_delimiters_literal() {
        assert_eq!(inline("a ** b"), vec![Inline { text: "a ** b".into(), bold: false, code: false }]);
        assert_eq!(inline("a ` b"), vec![Inline { text: "a ` b".into(), bold: false, code: false }]);
        let runs = inline("`a**b`");
        assert_eq!(runs, vec![Inline { text: "a**b".into(), bold: false, code: true }]);
    }

    #[test]
    fn parses_heading_bullet_paragraph() {
        let blocks = parse("# Title\n\nSome text.\n\n- a bullet\n");
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Heading(1, t) if t == "Title"));
        assert!(matches!(&blocks[1], Block::Paragraph(_)));
        assert!(matches!(&blocks[2], Block::Bullet(_)));
    }

    #[test]
    fn parses_run_tag_with_label() {
        let spans = parse_spans("Click [Run]bos-update|Update everything[/Run] to update.");
        assert_eq!(spans.len(), 3);
        assert!(matches!(&spans[0], Span::Text(t) if t == "Click "));
        assert_eq!(
            spans[1],
            Span::Run { command: "bos-update".into(), label: "Update everything".into() }
        );
        assert!(matches!(&spans[2], Span::Text(t) if t == " to update."));
    }

    #[test]
    fn parses_run_tag_without_label() {
        let spans = parse_spans("[Run]breadpad[/Run]");
        assert_eq!(spans, vec![Span::Run { command: "breadpad".into(), label: "breadpad".into() }]);
    }

    #[test]
    fn parses_show_keybind_tag() {
        let spans = parse_spans("Press [Show Keybind]super + space[/Show Keybind] to launch.");
        assert_eq!(spans[1], Span::Keybind("super + space".into()));
    }

    #[test]
    fn counts_multiple_tags_in_one_line() {
        let spans =
            parse_spans("[Run]a[/Run] then [Show Keybind]super+/[/Show Keybind] then [Run]b|B[/Run]");
        let run_count = spans.iter().filter(|s| matches!(s, Span::Run { .. })).count();
        let kb_count = spans.iter().filter(|s| matches!(s, Span::Keybind(_))).count();
        assert_eq!(run_count, 2);
        assert_eq!(kb_count, 1);
    }

    #[test]
    fn unterminated_tag_falls_back_to_text() {
        let spans = parse_spans("oops [Run]no closing tag");
        assert_eq!(spans, vec![Span::Text("oops ".into()), Span::Text("[Run]no closing tag".into())]);
    }

    #[test]
    fn parses_terminal_tag_into_argv_without_a_shell() {
        let spans = parse_spans("Click [Terminal]breadbug --app clip|Report a problem[/Terminal] now.");
        assert_eq!(spans.len(), 3);
        assert_eq!(
            spans[1],
            Span::Terminal {
                argv: vec!["breadbug".into(), "--app".into(), "clip".into()],
                label: "Report a problem".into(),
            }
        );
    }

    #[test]
    fn parses_terminal_tag_without_label() {
        let spans = parse_spans("[Terminal]breadbug[/Terminal]");
        assert_eq!(
            spans,
            vec![Span::Terminal { argv: vec!["breadbug".into()], label: "breadbug".into() }]
        );
    }

    #[test]
    fn unterminated_terminal_tag_falls_back_to_text() {
        let spans = parse_spans("oops [Terminal]no closing tag");
        assert_eq!(spans, vec![Span::Text("oops ".into()), Span::Text("[Terminal]no closing tag".into())]);
    }
}
