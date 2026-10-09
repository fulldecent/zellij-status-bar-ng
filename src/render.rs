use unicode_width::UnicodeWidthStr;

use crate::layout::{render_bar_with, Chrome, Hit};

/// Snapshot the status plugin would paint for one mode. No Zellij I/O.
#[derive(Clone, Debug)]
pub struct StatusState {
    pub mode: &'static str,
    pub multi_pane: bool,
    pub chrome: Chrome,
    pub hover_col: Option<usize>,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            mode: "normal",
            multi_pane: false,
            chrome: Chrome::default(),
            hover_col: None,
        }
    }
}

pub fn render_status_line(cols: usize, state: &StatusState) -> String {
    render_bar_with(
        state.mode,
        cols,
        state.multi_pane,
        &state.chrome,
        state.hover_col,
    )
    .0
}

pub fn render_status_hits(cols: usize, state: &StatusState) -> Vec<Hit> {
    render_bar_with(
        state.mode,
        cols,
        state.multi_pane,
        &state.chrome,
        state.hover_col,
    )
    .1
}

pub fn visible_width(rendered: &str) -> usize {
    strip_ansi(rendered).width()
}

pub fn strip_ansi(rendered: &str) -> String {
    let mut out = String::new();
    let mut chars = rendered.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                for c in chars.by_ref() {
                    if c == '\u{7}' || c == '\\' {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}
