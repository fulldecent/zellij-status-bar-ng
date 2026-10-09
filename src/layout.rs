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

/// What a click on a painted range should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    Mode(&'static str),
    Quit,
    NewPane,
    NewPaneDown,
    NewPaneRight,
    NewStacked,
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    ResizeIncrease,
    ResizeDecrease,
    ResizeIncreaseLeft,
    ResizeIncreaseDown,
    ResizeIncreaseUp,
    ResizeIncreaseRight,
    ResizeDecreaseLeft,
    ResizeDecreaseDown,
    ResizeDecreaseUp,
    ResizeDecreaseRight,
    MovePaneLeft,
    MovePaneDown,
    MovePaneUp,
    MovePaneRight,
    ToggleFloating,
    ToggleEmbed,
    ToggleFullscreen,
    ClosePane,
    CloseTab,
    NewTab,
    RenamePane,
    RenameTab,
    SyncTab,
    BreakPane,
    BreakPaneLeft,
    BreakPaneRight,
    ToggleTab,
    SelectPane,
    Detach,
    SessionManager,
    Configuration,
    PluginManager,
    About,
    Share,
    LayoutManager,
    EditScrollback,
    ScrollUp,
    ScrollDown,
    EnterSearch,
    CancelSearch,
    ConfirmSearch,
    SearchDown,
    SearchUp,
    SearchCase,
    SearchWrap,
    SearchWhole,
    TabPrev,
    TabNext,
    PageScrollUp,
    PageScrollDown,
    DoneRenamePane,
    DoneRenameTab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub start: usize,
    pub end: usize,
    pub click: Click,
}

#[derive(Clone, Debug)]
struct Chunk {
    style: Style,
    text: String,
    hit: Option<Click>,
}

impl Chunk {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            style: Style::Plain,
            text: text.into(),
            hit: None,
        }
    }
    fn bold(text: impl Into<String>) -> Self {
        Self {
            style: Style::Bold,
            text: text.into(),
            hit: None,
        }
    }
    fn orange(text: impl Into<String>) -> Self {
        Self {
            style: Style::BoldOrange,
            text: text.into(),
            hit: None,
        }
    }
    fn rev(text: impl Into<String>) -> Self {
        Self {
            style: Style::Reverse,
            text: text.into(),
            hit: None,
        }
    }
    fn rev_hit(text: impl Into<String>, hit: Click) -> Self {
        Self {
            style: Style::Reverse,
            text: text.into(),
            hit: Some(hit),
        }
    }
    fn plain_hit(text: impl Into<String>, hit: Click) -> Self {
        Self {
            style: Style::Plain,
            text: text.into(),
            hit: Some(hit),
        }
    }
}

struct Hint {
    token: &'static str,
    full: &'static str,
    abbr: &'static str,
    kind: Kind,
}

enum Kind {
    Chip(Click),
    Keys(&'static [Click]),
}

fn width(chunks: &[Chunk]) -> usize {
    chunks.iter().map(|c| c.text.chars().count()).sum()
}

fn plain_text(chunks: &[Chunk]) -> String {
    chunks.iter().map(|c| c.text.as_str()).collect()
}

fn token_chunks(h: &Hint) -> Vec<Chunk> {
    match h.kind {
        Kind::Chip(click) => vec![Chunk::rev_hit(h.token, click)],
        Kind::Keys(keys) => {
            let chars: Vec<char> = h.token.chars().collect();
            let prefix = chars.len().saturating_sub(keys.len());
            let mut out = Vec::new();
            if prefix > 0 {
                let head: String = chars[..prefix].iter().collect();
                out.push(Chunk::rev(head));
            }
            for (i, ch) in chars[prefix..].iter().enumerate() {
                out.push(Chunk::rev_hit(ch.to_string(), keys[i]));
            }
            out
        }
    }
}

fn named(h: &Hint, label: &str) -> Vec<Chunk> {
    let mut out = token_chunks(h);
    match h.kind {
        Kind::Chip(click) => {
            out.push(Chunk::plain_hit(format!(" {label}"), click));
            out.push(Chunk::plain("  "));
        }
        Kind::Keys(_) => {
            out.push(Chunk::plain(format!(" {label}  ")));
        }
    }
    out
}

fn key_only(h: &Hint) -> Vec<Chunk> {
    let mut out = token_chunks(h);
    out.push(Chunk::plain("  "));
    out
}

fn chip(token: &'static str, full: &'static str, abbr: &'static str, click: Click) -> Hint {
    Hint {
        token,
        full,
        abbr,
        kind: Kind::Chip(click),
    }
}

fn keys(
    token: &'static str,
    full: &'static str,
    abbr: &'static str,
    clicks: &'static [Click],
) -> Hint {
    Hint {
        token,
        full,
        abbr,
        kind: Kind::Keys(clicks),
    }
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

fn hovered_click(chunks: &[Chunk], col: usize) -> Option<Click> {
    let mut x = 0;
    for chunk in chunks {
        let w = chunk.text.chars().count();
        if col >= x && col < x + w {
            return chunk.hit;
        }
        x += w;
    }
    None
}

fn paint(chunks: &[Chunk], chrome: &Chrome, hover: Option<usize>) -> String {
    let active = hover.and_then(|col| hovered_click(chunks, col));
    let mut out = String::new();
    for chunk in chunks {
        out.push_str("\u{1b}[0m");
        let invert = chunk.hit.is_some() && chunk.hit == active;
        let params = match (chunk.style, invert) {
            (Style::Plain, false) => format!("22;{};{}", chrome.light.fg(), chrome.bar.bg()),
            (Style::Plain, true) => format!("22;{};{}", chrome.bar.fg(), chrome.light.bg()),
            (Style::Bold, false) => format!("1;{};{}", chrome.ctrl.fg(), chrome.bar.bg()),
            (Style::Bold, true) => format!("1;{};{}", chrome.bar.fg(), chrome.ctrl.bg()),
            (Style::BoldOrange, false) => format!("22;{};{}", chrome.alt.fg(), chrome.bar.bg()),
            (Style::BoldOrange, true) => format!("22;{};{}", chrome.bar.fg(), chrome.alt.bg()),
            (Style::Reverse, false) => format!("22;{};{}", chrome.dark.fg(), chrome.light.bg()),
            (Style::Reverse, true) => format!("22;{};{}", chrome.light.fg(), chrome.dark.bg()),
        };
        out.push_str(&format!("\u{1b}[{params}m"));
        out.push_str(&chunk.text);
    }
    out.push_str("\u{1b}[0m");
    out
}

fn hits_of(chunks: &[Chunk]) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();
    let mut x = 0;
    for chunk in chunks {
        let w = chunk.text.chars().count();
        if let Some(click) = chunk.hit {
            if let Some(last) = hits.last_mut() {
                if last.click == click && last.end == x {
                    last.end = x + w;
                    x += w;
                    continue;
                }
            }
            hits.push(Hit {
                start: x,
                end: x + w,
                click,
            });
        }
        x += w;
    }
    hits
}

fn finish(
    mut chunks: Vec<Chunk>,
    cols: usize,
    chrome: &Chrome,
    hover: Option<usize>,
) -> (String, Vec<Hit>) {
    let gap = cols.saturating_sub(width(&chunks));
    if gap > 0 {
        chunks.push(Chunk::plain(" ".repeat(gap)));
    }
    let hits = hits_of(&chunks);
    (paint(&chunks, chrome, hover), hits)
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

/// Context word for an active input mode. Shown at the far left, before `Ctrl +`.
fn mode_heading(mode: &str) -> Option<&'static str> {
    match mode {
        "pane" => Some("Pane"),
        "tab" => Some("Tab"),
        "resize" => Some("Resize"),
        "move" => Some("Move"),
        "scroll" => Some("Search"),
        "session" => Some("Session"),
        "tmux" => Some("Tmux"),
        _ => None,
    }
}

fn alt_prefix() -> Vec<Chunk> {
    vec![Chunk::orange("Alt + ")]
}

const FOCUS_HJKL: [Click; 4] = [
    Click::FocusLeft,
    Click::FocusDown,
    Click::FocusUp,
    Click::FocusRight,
];
const FOCUS_ARROWS: [Click; 4] = [
    Click::FocusLeft,
    Click::FocusDown,
    Click::FocusUp,
    Click::FocusRight,
];
const RESIZE_PM: [Click; 2] = [Click::ResizeIncrease, Click::ResizeDecrease];
const RESIZE_INC: [Click; 4] = [
    Click::ResizeIncreaseLeft,
    Click::ResizeIncreaseDown,
    Click::ResizeIncreaseUp,
    Click::ResizeIncreaseRight,
];
const RESIZE_DEC: [Click; 4] = [
    Click::ResizeDecreaseLeft,
    Click::ResizeDecreaseDown,
    Click::ResizeDecreaseUp,
    Click::ResizeDecreaseRight,
];
const MOVE_HJKL: [Click; 4] = [
    Click::MovePaneLeft,
    Click::MovePaneDown,
    Click::MovePaneUp,
    Click::MovePaneRight,
];
const SCROLL_HJKL: [Click; 4] = [
    Click::PageScrollUp,
    Click::ScrollDown,
    Click::ScrollUp,
    Click::PageScrollDown,
];
const TAB_HL: [Click; 2] = [Click::TabPrev, Click::TabNext];
const BREAK_BRACKETS: [Click; 2] = [Click::BreakPaneLeft, Click::BreakPaneRight];
const ALT_RESIZE: [Click; 2] = [Click::ResizeIncrease, Click::ResizeDecrease];

fn normal_modes() -> Vec<Hint> {
    vec![
        chip("^G", "Lock", "Lock", Click::Mode("locked")),
        chip("^P", "Pane", "Pane", Click::Mode("pane")),
        chip("^T", "Tab", "Tab", Click::Mode("tab")),
        chip("^N", "Resize", "Resize", Click::Mode("resize")),
        chip("^H", "Move", "Move", Click::Mode("move")),
        chip("^S", "Search", "Search", Click::Mode("scroll")),
        chip("^O", "Session", "Session", Click::Mode("session")),
        chip("^Q", "Quit", "Quit", Click::Quit),
    ]
}

fn alt_items(multi_pane: bool) -> Vec<Hint> {
    let mut items = vec![chip("⌥N", "New Pane", "New", Click::NewPane)];
    if multi_pane {
        items.push(keys("⌥←↓↑→", "Change Focus", "Focus", &FOCUS_ARROWS));
        items.push(keys("⌥+-", "Resize", "Resize", &ALT_RESIZE));
    }
    items.push(chip("⌥F", "Floating", "Floating", Click::ToggleFloating));
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
        "locked" => vec![chip("^G", "Lock", "Lock", Click::Mode("normal"))],
        _ => return None,
    };
    Some(items)
}

fn mode_actions(mode: &str) -> Vec<Hint> {
    match mode {
        "pane" => vec![
            chip("^N", "New", "New", Click::NewPane),
            keys("^HJKL", "Change Focus", "Move", &FOCUS_HJKL),
            chip("^X", "Close", "Close", Click::ClosePane),
            chip("^C", "Rename", "Rename", Click::RenamePane),
            chip(
                "^F",
                "Toggle Fullscreen",
                "Fullscreen",
                Click::ToggleFullscreen,
            ),
            chip("^W", "Toggle Floating", "Floating", Click::ToggleFloating),
            chip("^E", "Toggle Embed", "Embed", Click::ToggleEmbed),
            chip("^R", "Split Right", "Right", Click::NewPaneRight),
            chip("^D", "Split Down", "Down", Click::NewPaneDown),
            chip("^S", "Stack", "Stack", Click::NewStacked),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tab" => vec![
            chip("^N", "New", "New", Click::NewTab),
            keys("^HL", "Change Focus", "Move", &TAB_HL),
            chip("^X", "Close", "Close", Click::CloseTab),
            chip("^R", "Rename", "Rename", Click::RenameTab),
            chip("^S", "Sync", "Sync", Click::SyncTab),
            chip("^B", "Break Pane To New Tab", "Break Out", Click::BreakPane),
            keys("^[]", "Break Pane Left/Right", "Break", &BREAK_BRACKETS),
            chip("^TAB", "Toggle", "Toggle", Click::ToggleTab),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "resize" => vec![
            keys(
                "^+-",
                "Increase/Decrease Size",
                "Increase/Decrease",
                &RESIZE_PM,
            ),
            keys("^HJKL", "Increase To", "Increase", &RESIZE_INC),
            keys("^HJKL", "Decrease From", "Decrease", &RESIZE_DEC),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "move" => vec![
            keys("^HJKL", "Switch Location", "Move", &MOVE_HJKL),
            chip("^ENTER", "When Done", "Back", Click::SelectPane),
        ],
        "scroll" => vec![
            chip("^S", "Enter Search Term", "Search", Click::EnterSearch),
            keys("^HJKL", "Scroll", "Scroll", &SCROLL_HJKL),
            chip(
                "^E",
                "Edit Scrollback In Default Editor",
                "Edit",
                Click::EditScrollback,
            ),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "enter_search" => vec![
            chip("^ENTER", "When Done", "Done", Click::ConfirmSearch),
            chip("^ESC", "Cancel", "Cancel", Click::CancelSearch),
        ],
        "search" => vec![
            chip("^N", "Search Down", "Down", Click::SearchDown),
            chip("^P", "Search Up", "Up", Click::SearchUp),
            chip("^C", "Case Sensitive", "Case", Click::SearchCase),
            chip("^W", "Wrap", "Wrap", Click::SearchWrap),
            chip("^O", "Whole Words", "Whole", Click::SearchWhole),
        ],
        "session" => vec![
            chip("^D", "Detach", "Detach", Click::Detach),
            chip("^W", "Session Manager", "Manager", Click::SessionManager),
            chip("^S", "Share", "Share", Click::Share),
            chip("^C", "Configure", "Config", Click::Configuration),
            chip("^L", "Layout Manager", "Layouts", Click::LayoutManager),
            chip("^P", "Plugin Manager", "Plugins", Click::PluginManager),
            chip("^A", "About", "About", Click::About),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tmux" => vec![
            keys("^HJKL", "Move Focus", "Move", &FOCUS_HJKL),
            chip("^\"", "Split Down", "Down", Click::NewPaneDown),
            chip("^%", "Split Right", "Right", Click::NewPaneRight),
            chip("^Z", "Fullscreen", "Fullscreen", Click::ToggleFullscreen),
            chip("^C", "New Tab", "New", Click::NewTab),
            chip("^,", "Rename Tab", "Rename", Click::RenameTab),
            chip("^P", "Previous Tab", "Previous", Click::TabPrev),
            chip("^N", "Next Tab", "Next", Click::TabNext),
            chip("^ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "rename_pane" => vec![chip("^ESC", "When Done", "Done", Click::DoneRenamePane)],
        "rename_tab" => vec![chip("^ESC", "When Done", "Done", Click::DoneRenameTab)],
        _ => vec![],
    }
}

#[cfg(test)]
fn render_bar(mode: &str, cols: usize, multi_pane: bool) -> String {
    render_bar_with(mode, cols, multi_pane, &Chrome::default(), None).0
}

pub fn render_bar_with(
    mode: &str,
    cols: usize,
    multi_pane: bool,
    chrome: &Chrome,
    hover: Option<usize>,
) -> (String, Vec<Hit>) {
    if cols == 0 {
        return (String::new(), Vec::new());
    }

    if mode == "normal" || mode == "locked" {
        let ctrl_options = indicator(mode)
            .map(|items| variants(ctrl_prefix(), &items, false))
            .unwrap_or_else(|| vec![Vec::new()]);
        let alt_options = if mode == "normal" {
            variants(alt_prefix(), &alt_items(multi_pane), true)
        } else {
            vec![Vec::new()]
        };
        // Keep the Ctrl group as large as possible. Only the leftover width is
        // offered to Alt, which then uses the largest form that fits there.
        for ctrl in &ctrl_options {
            if width(ctrl) > cols {
                continue;
            }
            if plain_text(ctrl) == "..." {
                return finish(ctrl.clone(), cols, chrome, hover);
            }
            let remain = cols - width(ctrl);
            let alt = alt_options
                .iter()
                .find(|option| width(option) <= remain)
                .cloned()
                .unwrap_or_default();
            return finish(place(ctrl.clone(), alt, cols), cols, chrome, hover);
        }
        return finish(Vec::new(), cols, chrome, hover);
    }

    if let Some(heading) = mode_heading(mode) {
        let prefix = vec![Chunk::orange(heading), Chunk::bold(" Ctrl + ")];
        let chosen = first_fit(&variants(prefix, &mode_actions(mode), true), cols);
        return finish(chosen, cols, chrome, hover);
    }

    let mut remain = cols;
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
        return finish(Vec::new(), cols, chrome, hover);
    };
    remain = remain.saturating_sub(width(&sep));
    let actions = first_fit(&variants(Vec::new(), &mode_actions(mode), true), remain);
    let mut out = sep;
    if !actions.is_empty() {
        out.extend(actions);
    } else {
        out = Vec::new();
    }
    finish(out, cols, chrome, hover)
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

    #[test]
    fn resize_chip_is_nine_cells() {
        let hits = render_bar_with("normal", 220, false, &Chrome::default(), None).1;
        let hit = hits
            .iter()
            .find(|h| h.click == Click::Mode("resize"))
            .expect("resize chip");
        assert_eq!(hit.end - hit.start, 9);
        let text = shown("normal", 220, false);
        assert_eq!(
            &text.chars().skip(hit.start).take(9).collect::<String>(),
            "^N Resize"
        );
    }

    #[test]
    fn arrow_hits_are_one_cell() {
        let hits = render_bar_with("normal", 220, true, &Chrome::default(), None).1;
        let arrows: Vec<_> = hits
            .iter()
            .filter(|h| {
                matches!(
                    h.click,
                    Click::FocusLeft | Click::FocusDown | Click::FocusUp | Click::FocusRight
                )
            })
            .collect();
        assert_eq!(arrows.len(), 4);
        for hit in arrows {
            assert_eq!(hit.end - hit.start, 1);
        }
    }

    #[test]
    fn hover_inverts_chip_without_new_colors() {
        let chrome = Chrome::default();
        let idle = render_bar_with("normal", 220, false, &chrome, None).0;
        let hits = render_bar_with("normal", 220, false, &chrome, None).1;
        let resize = hits
            .iter()
            .find(|h| h.click == Click::Mode("resize"))
            .unwrap();
        let hovered = render_bar_with("normal", 220, false, &chrome, Some(resize.start)).0;
        assert_ne!(idle, hovered);
        assert!(hovered.contains(&format!("22;{};{}", chrome.light.fg(), chrome.dark.bg())));
        assert!(!hovered.contains("38;5;255;48;5;255"));
    }

    #[test]
    fn active_mode_name_sits_left_of_ctrl() {
        for (mode, name) in [
            ("pane", "Pane"),
            ("tab", "Tab"),
            ("resize", "Resize"),
            ("move", "Move"),
            ("scroll", "Search"),
            ("session", "Session"),
            ("tmux", "Tmux"),
        ] {
            let raw = render_bar(mode, 220, false);
            let text = visible(&raw);
            assert!(
                text.starts_with(&format!("{name} Ctrl + ")),
                "{mode} should start with {name} Ctrl +: {text}"
            );
            assert!(
                raw.contains(&format!("\u{1b}[22;38;5;166;48;5;16m{name}")),
                "{mode} heading should be orange: {raw}"
            );
            assert!(
                raw.contains("\u{1b}[1;38;5;255;48;5;16m Ctrl + "),
                "{mode} should keep a bold Ctrl + after the heading"
            );
        }

        let session = shown("session", 220, false);
        assert!(session.contains("^D Detach"), "{session}");
        assert!(
            !session.contains("^O Session"),
            "session mode should not repeat the Session chip after Ctrl +: {session}"
        );

        let tab = shown("tab", 220, false);
        assert!(tab.contains("^N New"), "{tab}");
        assert!(
            !tab.contains("^T Tab"),
            "tab mode should not repeat the Tab chip after Ctrl +: {tab}"
        );

        let normal = shown("normal", 220, false);
        assert!(normal.starts_with(" Ctrl + "), "{normal}");
        assert!(!normal.starts_with("Normal"), "{normal}");

        let locked = shown("locked", 220, false);
        assert!(locked.starts_with(" Ctrl + "), "{locked}");
        assert!(locked.contains("^G Lock"), "{locked}");
    }
}
