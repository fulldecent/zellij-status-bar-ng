//! Normal-mode line for Zellij's stock status bar.
//!
//! Port of `default-plugins/status-bar` at tag `v0.45.1`
//! (`render_common_modifiers`, `long_mode_shortcut`, and the shortened
//! key ribbon in `one_line_ui.rs`). The upstream crate does not build
//! outside the Zellij workspace. The sources are in `third_party/`.

use unicode_width::UnicodeWidthStr;

use crate::layout::Chrome;
use crate::render::{strip_ansi, StatusState};

struct Hint {
    key: &'static str,
    label: &'static str,
}

pub fn render_stock_status_line(cols: usize, state: &StatusState) -> String {
    if cols == 0 || state.mode != "normal" {
        return pad("", cols, &state.chrome);
    }
    let modes = [
        Hint {
            key: "G",
            label: "Lock",
        },
        Hint {
            key: "P",
            label: "Pane",
        },
        Hint {
            key: "T",
            label: "Tab",
        },
        Hint {
            key: "N",
            label: "Resize",
        },
        Hint {
            key: "H",
            label: "Move",
        },
        Hint {
            key: "S",
            label: "Search",
        },
        Hint {
            key: "O",
            label: "Session",
        },
        Hint {
            key: "Q",
            label: "Quit",
        },
    ];
    let mut alt = vec![
        Hint {
            key: "N",
            label: "New Pane",
        },
        Hint {
            key: "F",
            label: "Floating",
        },
    ];
    if state.multi_pane {
        alt.insert(
            1,
            Hint {
                key: "←↓↑→",
                label: "Change Focus",
            },
        );
        alt.insert(
            2,
            Hint {
                key: "+-",
                label: "Resize",
            },
        );
    }
    let prefix = paint(" Ctrl + ", true, state.chrome.ctrl, state.chrome.bar);
    let full = format!(
        "{prefix}{}{}",
        tiles(&modes, true, &state.chrome),
        tiles(&alt, true, &state.chrome)
    );
    let short = format!(
        "{prefix}{}{}",
        tiles(&modes, false, &state.chrome),
        tiles(&alt, false, &state.chrome)
    );
    let chosen = if visible_width_of(&full) <= cols {
        full
    } else if visible_width_of(&short) <= cols {
        short
    } else {
        format!(
            "{prefix}{}",
            best_effort(&modes, &state.chrome, cols.saturating_sub(8))
        )
    };
    pad(&chosen, cols, &state.chrome)
}

fn tiles(items: &[Hint], long: bool, chrome: &Chrome) -> String {
    let mut out = String::new();
    for item in items {
        out.push_str(&chevron(chrome.bar, chrome.light));
        if long {
            out.push_str(&paint(" <", false, chrome.dark, chrome.light));
            out.push_str(&paint(item.key, false, chrome.key_accent, chrome.light));
            out.push_str(&paint(
                &format!("> {} ", item.label),
                false,
                chrome.dark,
                chrome.light,
            ));
        } else {
            out.push_str(&paint(
                &format!(" {} ", item.key),
                false,
                chrome.key_accent,
                chrome.light,
            ));
        }
        out.push_str(&chevron(chrome.light, chrome.bar));
    }
    out
}

fn best_effort(items: &[Hint], chrome: &Chrome, budget: usize) -> String {
    let mut out = String::new();
    for item in items {
        let piece = format!(
            "{}{}{}",
            chevron(chrome.bar, chrome.light),
            paint(
                &format!(" {} ", item.key),
                false,
                chrome.key_accent,
                chrome.light
            ),
            chevron(chrome.light, chrome.bar)
        );
        if visible_width_of(&out) + visible_width_of(&piece) > budget {
            break;
        }
        out.push_str(&piece);
    }
    out
}

fn chevron(fg: crate::layout::Ink, bg: crate::layout::Ink) -> String {
    paint("\u{e0b0}", false, fg, bg)
}

fn paint(text: &str, bold: bool, fg: crate::layout::Ink, bg: crate::layout::Ink) -> String {
    let weight = if bold { "1;" } else { "22;" };
    format!("\u{1b}[0m\u{1b}[{weight}{};{}m{text}", fg.fg(), bg.bg())
}

fn pad(body: &str, cols: usize, chrome: &Chrome) -> String {
    let width = visible_width_of(body);
    if width >= cols {
        return clip_visible(body, cols);
    }
    format!(
        "{body}\u{1b}[0m\u{1b}[{}m{}",
        chrome.bar.bg(),
        " ".repeat(cols - width)
    )
}

fn visible_width_of(rendered: &str) -> usize {
    strip_ansi(rendered).width()
}

fn clip_visible(rendered: &str, cols: usize) -> String {
    let plain = strip_ansi(rendered);
    let mut taken = String::new();
    let mut w = 0;
    for ch in plain.chars() {
        let cw = ch.to_string().width();
        if w + cw > cols {
            break;
        }
        taken.push(ch);
        w += cw;
    }
    paint(
        &taken,
        false,
        crate::layout::Ink::Bit(255),
        crate::layout::Ink::Bit(16),
    )
}
