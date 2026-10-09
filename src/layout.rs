//! Both groups shrink from the right before either one collapses to "...".
//! Full names, then shorter names, then keys only from the right. Once the
//! keys themselves no longer fit, keep as many as possible and end with
//! "...", and only then a colored ellipsis.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Style {
    Plain,
    Bold,
    BoldOrange,
    Reverse,
}

#[derive(Clone, Debug)]
struct Chunk {
    style: Style,
    text: String,
}

impl Chunk {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            style: Style::Plain,
            text: text.into(),
        }
    }
    fn bold(text: impl Into<String>) -> Self {
        Self {
            style: Style::Bold,
            text: text.into(),
        }
    }
    fn orange(text: impl Into<String>) -> Self {
        Self {
            style: Style::BoldOrange,
            text: text.into(),
        }
    }
    fn rev(text: impl Into<String>) -> Self {
        Self {
            style: Style::Reverse,
            text: text.into(),
        }
    }
}

struct Hint {
    token: &'static str,
    full: &'static str,
    abbr: &'static str,
}

fn width(chunks: &[Chunk]) -> usize {
    chunks.iter().map(|c| c.text.chars().count()).sum()
}

fn plain_text(chunks: &[Chunk]) -> String {
    chunks.iter().map(|c| c.text.as_str()).collect()
}

fn named(h: &Hint, label: &str) -> Vec<Chunk> {
    vec![Chunk::rev(h.token), Chunk::plain(format!(" {label}  "))]
}

fn key_only(h: &Hint) -> Vec<Chunk> {
    vec![Chunk::rev(h.token), Chunk::plain("  ")]
}

fn hint(token: &'static str, full: &'static str, abbr: &'static str) -> Hint {
    Hint { token, full, abbr }
}

/// Largest form first. The last entry is the colored ellipsis with no prefix.
fn variants(prefix: Vec<Chunk>, items: &[Hint], abbreviate: bool) -> Vec<Vec<Chunk>> {
    let mut out: Vec<Vec<Chunk>> = Vec::new();
    let mut push = |chunks: Vec<Chunk>| {
        let text = plain_text(&chunks);
        if out
            .last()
            .map(|prev| plain_text(prev) != text)
            .unwrap_or(true)
        {
            out.push(chunks);
        }
    };

    let mut full = prefix.clone();
    for item in items {
        full.extend(named(item, item.full));
    }
    push(full);

    if abbreviate {
        let mut abbr = prefix.clone();
        for item in items {
            abbr.extend(named(item, item.abbr));
        }
        push(abbr);
    }

    let n = items.len();
    for dropped in 1..=n {
        let mut row = prefix.clone();
        for (i, item) in items.iter().enumerate() {
            if i + dropped < n {
                let label = if abbreviate { item.abbr } else { item.full };
                row.extend(named(item, label));
            } else {
                row.extend(key_only(item));
            }
        }
        push(row);
    }

    // Drop keys from the right, but keep an ellipsis so the cut is visible.
    // One column under the all-keys row still shows every key that fits.
    for keep in (1..n).rev() {
        let mut row = prefix.clone();
        for item in items.iter().take(keep - 1) {
            row.extend(key_only(item));
        }
        row.push(Chunk::rev(items[keep - 1].token));
        row.push(Chunk::plain(" ..."));
        push(row);
    }

    push(vec![Chunk::orange("...")]);
    out
}

fn first_fit(options: &[Vec<Chunk>], cols: usize) -> Vec<Chunk> {
    options
        .iter()
        .find(|option| width(option) <= cols)
        .cloned()
        .unwrap_or_default()
}

/// Colors taken from the same palette fields the stock status bar uses.
/// Defaults match `Styling::from(default_palette())`:
/// `ribbon_unselected.base` is black 16, `ribbon_unselected.background` is fg 245.
#[derive(Clone, Copy, Debug)]
pub struct Chrome {
    pub bar: Ink,
    pub dark: Ink,
    pub light: Ink,
    pub ctrl: Ink,
    pub alt: Ink,
    pub plus_hover: Ink,
    /// `ribbon_unselected.emphasis_0` — the red key letter inside a stock `<g>` chip.
    pub key_accent: Ink,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            bar: Ink::Bit(16),
            dark: Ink::Bit(16),
            light: Ink::Bit(245),
            ctrl: Ink::Bit(255),
            alt: Ink::Bit(166),
            plus_hover: Ink::Bit(255),
            key_accent: Ink::Bit(124),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ink {
    Bit(u8),
    Rgb(u8, u8, u8),
}

impl Ink {
    pub(crate) fn fg(self) -> String {
        match self {
            Ink::Bit(n) => format!("38;5;{n}"),
            Ink::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }
    pub(crate) fn bg(self) -> String {
        match self {
            Ink::Bit(n) => format!("48;5;{n}"),
            Ink::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
        }
    }
}

fn paint(chunks: &[Chunk], chrome: &Chrome) -> String {
    let mut out = String::new();
    for chunk in chunks {
        out.push_str("\u{1b}[0m");
        let params = match chunk.style {
            // Labels are the ribbon gray on the black bar. Only Ctrl + is bold.
            Style::Plain => format!("22;{};{}", chrome.light.fg(), chrome.bar.bg()),
            Style::Bold => format!("1;{};{}", chrome.ctrl.fg(), chrome.bar.bg()),
            Style::BoldOrange => format!("22;{};{}", chrome.alt.fg(), chrome.bar.bg()),
            // `^G` / `⌥N`: the dark `<` `>` color on the ribbon gray.
            Style::Reverse => format!("22;{};{}", chrome.dark.fg(), chrome.light.bg()),
        };
        out.push_str(&format!("\u{1b}[{params}m"));
        out.push_str(&chunk.text);
    }
    out.push_str("\u{1b}[0m");
    out
}

fn finish(mut chunks: Vec<Chunk>, cols: usize, chrome: &Chrome) -> String {
    let gap = cols.saturating_sub(width(&chunks));
    if gap > 0 {
        chunks.push(Chunk::plain(" ".repeat(gap)));
    }
    paint(&chunks, chrome)
}

#[cfg(test)]
fn visible(rendered: &str) -> String {
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
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn ctrl_prefix() -> Vec<Chunk> {
    vec![Chunk::bold(" Ctrl + ")]
}

fn alt_prefix() -> Vec<Chunk> {
    vec![Chunk::orange("Alt + ")]
}

fn normal_modes() -> Vec<Hint> {
    vec![
        hint("^G", "Lock", "Lock"),
        hint("^P", "Pane", "Pane"),
        hint("^T", "Tab", "Tab"),
        hint("^N", "Resize", "Resize"),
        hint("^H", "Move", "Move"),
        hint("^S", "Search", "Search"),
        hint("^O", "Session", "Session"),
        hint("^Q", "Quit", "Quit"),
    ]
}

fn alt_items(multi_pane: bool) -> Vec<Hint> {
    let mut items = vec![hint("⌥N", "New Pane", "New")];
    if multi_pane {
        items.push(hint("⌥←↓↑→", "Change Focus", "Focus"));
        items.push(hint("⌥+-", "Resize", "Resize"));
    }
    items.push(hint("⌥F", "Floating", "Floating"));
    items
}

fn place(left: Vec<Chunk>, right: Vec<Chunk>, cols: usize) -> Vec<Chunk> {
    if right.is_empty() {
        return left;
    }
    let used = width(&left) + width(&right);
    let pad = cols.saturating_sub(1).saturating_sub(used);
    let mut out = left;
    if pad > 0 {
        out.push(Chunk::plain(" ".repeat(pad)));
    }
    out.extend(right);
    out
}

fn help_title(mode: &str) -> Option<&'static str> {
    match mode {
        "rename_pane" => Some("Renaming Pane"),
        "rename_tab" => Some("Renaming Tab"),
        "enter_search" => Some("Entering Search Term"),
        "search" => Some("Searching"),
        _ => None,
    }
}

fn indicator(mode: &str) -> Option<Vec<Hint>> {
    let items = match mode {
        "normal" => normal_modes(),
        "locked" => vec![hint("^G", "Lock", "Lock")],
        "pane" => vec![hint("^P", "Pane", "Pane")],
        "tab" => vec![hint("^T", "Tab", "Tab")],
        "resize" => vec![hint("^N", "Resize", "Resize")],
        "move" => vec![hint("^H", "Move", "Move")],
        "scroll" => vec![hint("^S", "Search", "Search")],
        "session" => vec![hint("^O", "Session", "Session")],
        "tmux" => vec![hint("^B", "Tmux", "Tmux")],
        _ => return None,
    };
    Some(items)
}

fn mode_actions(mode: &str) -> Vec<Hint> {
    match mode {
        "pane" => vec![
            hint("^N", "New", "New"),
            hint("^HJKL", "Change Focus", "Move"),
            hint("^X", "Close", "Close"),
            hint("^C", "Rename", "Rename"),
            hint("^F", "Toggle Fullscreen", "Fullscreen"),
            hint("^W", "Toggle Floating", "Floating"),
            hint("^E", "Toggle Embed", "Embed"),
            hint("^R", "Split Right", "Right"),
            hint("^D", "Split Down", "Down"),
            hint("^S", "Stack", "Stack"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "tab" => vec![
            hint("^N", "New", "New"),
            hint("^HL", "Change Focus", "Move"),
            hint("^X", "Close", "Close"),
            hint("^R", "Rename", "Rename"),
            hint("^S", "Sync", "Sync"),
            hint("^B", "Break Pane To New Tab", "Break Out"),
            hint("^[]", "Break Pane Left/Right", "Break"),
            hint("^TAB", "Toggle", "Toggle"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "resize" => vec![
            hint("^+-", "Increase/Decrease Size", "Increase/Decrease"),
            hint("^HJKL", "Increase To", "Increase"),
            hint("^HJKL", "Decrease From", "Decrease"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "move" => vec![
            hint("^HJKL", "Switch Location", "Move"),
            hint("^ENTER", "When Done", "Back"),
        ],
        "scroll" => vec![
            hint("^S", "Enter Search Term", "Search"),
            hint("^HJKL", "Scroll", "Scroll"),
            hint("^E", "Edit Scrollback In Default Editor", "Edit"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "enter_search" => vec![
            hint("^ENTER", "When Done", "Done"),
            hint("^ESC", "Cancel", "Cancel"),
        ],
        "search" => vec![
            hint("^N", "Search Down", "Down"),
            hint("^P", "Search Up", "Up"),
            hint("^C", "Case Sensitive", "Case"),
            hint("^W", "Wrap", "Wrap"),
            hint("^O", "Whole Words", "Whole"),
        ],
        "session" => vec![
            hint("^D", "Detach", "Detach"),
            hint("^W", "Session Manager", "Manager"),
            hint("^S", "Share", "Share"),
            hint("^C", "Configure", "Config"),
            hint("^L", "Layout Manager", "Layouts"),
            hint("^P", "Plugin Manager", "Plugins"),
            hint("^A", "About", "About"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "tmux" => vec![
            hint("^HJKL", "Move Focus", "Move"),
            hint("^\"", "Split Down", "Down"),
            hint("^%", "Split Right", "Right"),
            hint("^Z", "Fullscreen", "Fullscreen"),
            hint("^C", "New Tab", "New"),
            hint("^,", "Rename Tab", "Rename"),
            hint("^P", "Previous Tab", "Previous"),
            hint("^N", "Next Tab", "Next"),
            hint("^ENTER", "Select Pane", "Select"),
        ],
        "rename_pane" | "rename_tab" => vec![hint("^ESC", "When Done", "Done")],
        _ => vec![],
    }
}

#[cfg(test)]
fn render_bar(mode: &str, cols: usize, multi_pane: bool) -> String {
    render_bar_with(mode, cols, multi_pane, &Chrome::default())
}

pub fn render_bar_with(mode: &str, cols: usize, multi_pane: bool, chrome: &Chrome) -> String {
    if cols == 0 {
        return String::new();
    }

    let ctrl_options = indicator(mode)
        .map(|items| variants(ctrl_prefix(), &items, false))
        .unwrap_or_else(|| vec![Vec::new()]);
    let alt_options = if mode == "normal" {
        variants(alt_prefix(), &alt_items(multi_pane), true)
    } else {
        vec![Vec::new()]
    };

    if mode == "normal" || mode == "locked" {
        // Keep the Ctrl group as large as possible. Only the leftover width is
        // offered to Alt, which then uses the largest form that fits there.
        for ctrl in &ctrl_options {
            if width(ctrl) > cols {
                continue;
            }
            if plain_text(ctrl) == "..." {
                return finish(ctrl.clone(), cols, chrome);
            }
            let remain = cols - width(ctrl);
            let alt = alt_options
                .iter()
                .find(|option| width(option) <= remain)
                .cloned()
                .unwrap_or_default();
            return finish(place(ctrl.clone(), alt, cols), cols, chrome);
        }
        return finish(Vec::new(), cols, chrome);
    }

    let ctrl = first_fit(&ctrl_options, cols);
    if plain_text(&ctrl) == "..." {
        return finish(ctrl, cols, chrome);
    }
    let mut remain = cols.saturating_sub(width(&ctrl));
    let sep = if let Some(title) = help_title(mode) {
        let text = format!(" {title}  ");
        if text.chars().count() <= remain {
            Some(vec![Chunk::orange(text)])
        } else if remain >= 2 {
            Some(vec![Chunk::plain("  ")])
        } else {
            None
        }
    } else if remain >= 2 {
        Some(vec![Chunk::plain("  ")])
    } else {
        None
    };
    let Some(sep) = sep else {
        return finish(ctrl, cols, chrome);
    };
    remain = remain.saturating_sub(width(&sep));
    let actions = first_fit(&variants(Vec::new(), &mode_actions(mode), true), remain);
    let mut out = ctrl;
    if !actions.is_empty() {
        out.extend(sep);
        out.extend(actions);
    }
    finish(out, cols, chrome)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(mode: &str, cols: usize, multi: bool) -> String {
        visible(&render_bar(mode, cols, multi))
    }

    #[test]
    fn never_wider_than_the_screen() {
        for cols in 0..240 {
            for mode in [
                "normal",
                "locked",
                "pane",
                "tab",
                "resize",
                "move",
                "scroll",
                "session",
                "tmux",
                "search",
                "enter_search",
                "rename_tab",
                "rename_pane",
            ] {
                let text = shown(mode, cols, true);
                assert!(
                    text.chars().count() <= cols,
                    "{mode} at {cols} is {} wide: {text}",
                    text.chars().count()
                );
            }
        }
    }

    #[test]
    fn prefixes_are_bold_and_names_are_title_case() {
        let raw = render_bar("normal", 220, false);
        assert!(raw.contains("\u{1b}[1;38;5;255;48;5;16m Ctrl + "));
        assert!(raw.contains("\u{1b}[22;38;5;166;48;5;16mAlt + "));
        assert!(raw.contains("\u{1b}[22;38;5;16;48;5;245m^G"));
        assert!(raw.contains("\u{1b}[22;38;5;245;48;5;16m Lock"));
        assert!(!raw.contains("\u{1b}[1;38;5;166m"));
        let text = visible(&raw);
        assert!(text.contains("^G Lock"));
        assert!(text.contains("^Q Quit"));
        assert!(!text.contains("LOCK"));
        assert!(text.contains("⌥N New Pane"));
        assert!(text.contains("⌥F Floating"));
    }

    #[test]
    fn alt_group_drops_names_before_collapsing_to_ellipsis() {
        let mut saw_key_without_floating_word = false;
        let mut saw_keys_only = false;
        let mut saw_first_plus_ellipsis = false;
        for cols in (1..160).rev() {
            let text = shown("normal", cols, false);
            if text.contains("⌥N New") && text.contains("⌥F  ") && !text.contains("Floating") {
                saw_key_without_floating_word = true;
            }
            if text.contains("Alt + ⌥N  ⌥F  ") && !text.contains("New") {
                saw_keys_only = true;
            }
            if text.contains("Alt + ⌥N ...") {
                saw_first_plus_ellipsis = true;
            }
        }
        assert!(saw_key_without_floating_word);
        assert!(saw_keys_only);
        assert!(saw_first_plus_ellipsis);
    }

    #[test]
    fn one_column_under_all_keys_keeps_the_keys_that_fit() {
        let options = variants(alt_prefix(), &alt_items(true), true);
        let all_keys = options
            .iter()
            .find(|option| {
                let text = plain_text(option);
                text.contains("⌥←↓↑→")
                    && text.contains("⌥+-")
                    && text.contains("⌥F")
                    && !text.contains("...")
                    && !text.contains("New")
                    && !text.contains("Focus")
                    && !text.contains("Floating")
            })
            .expect("all-keys form");
        let text = plain_text(&first_fit(&options, width(all_keys) - 1));
        assert!(text.contains("⌥←↓↑→"), "dropped too far: {text}");
        assert!(text.contains("..."), "{text}");
        assert!(!text.contains("⌥F"), "{text}");
    }

    #[test]
    fn colored_ellipsis_is_last_resort() {
        let text = shown("normal", 4, false);
        assert!(text.starts_with("..."));
        assert_eq!(text.chars().count(), 4);
        assert!(render_bar("normal", 4, false).contains("38;5;166"));
    }

    #[test]
    fn ctrl_peels_the_right_hand_name_first() {
        let mut found = false;
        for cols in 70..90 {
            let text = shown("normal", cols, false);
            if text.contains("^O Session") && text.contains("^Q  ") && !text.contains("Quit") {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}
