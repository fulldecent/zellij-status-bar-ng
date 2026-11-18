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

fn is_chip_label(chunk: &Chunk) -> bool {
    chunk.hit.is_some() && chunk.style == Style::Plain && chunk.text.chars().any(|ch| ch != ' ')
}

fn click_has_label(chunks: &[Chunk], click: Click) -> bool {
    chunks.iter().any(|chunk| {
        chunk.hit == Some(click)
            && chunk.style == Style::Plain
            && chunk.text.chars().any(|ch| ch != ' ')
    })
}

/// Click of a labeled chip whose key chunk is `chunks[idx]`.
fn labeled_chip_at(chunks: &[Chunk], idx: usize) -> Option<Click> {
    let click = chunks.get(idx)?.hit?;
    if !click_has_label(chunks, click) {
        return None;
    }
    // The label shares this click and appears before a different click.
    for chunk in chunks.iter().skip(idx + 1) {
        if chunk.hit == Some(click)
            && chunk.style == Style::Plain
            && chunk.text.chars().any(|ch| ch != ' ')
        {
            return Some(click);
        }
        if chunk.hit.is_some() && chunk.hit != Some(click) {
            return None;
        }
        if chunk.text.chars().any(|ch| ch != ' ')
            && chunk.hit != Some(click)
            && chunk.style != Style::Reverse
        {
            return None;
        }
    }
    None
}

/// The two spaces after a labeled chip join the click targets. The first is
/// the right side of this chip. The second is the left side of the next
/// labeled chip, or this chip's right side when nothing labeled follows.
/// A prefix space immediately before a labeled chip is that chip's left side.
/// Key-only chips and letter combos such as `HJKL` keep their own hits.
fn bind_chip_pads(chunks: &[Chunk]) -> Vec<Chunk> {
    let mut spaced = Vec::with_capacity(chunks.len());
    let mut i = 0;
    while i < chunks.len() {
        let separator = chunks
            .get(i + 1)
            .is_some_and(|chunk| chunk.hit.is_none() && chunk.text == "  ");
        if separator && is_chip_label(&chunks[i]) {
            let click = chunks[i].hit.expect("labeled chip");
            spaced.push(chunks[i].clone());
            spaced.push(Chunk::plain_hit(" ", click));
            let next = labeled_chip_at(chunks, i + 2).unwrap_or(click);
            spaced.push(Chunk::plain_hit(" ", next));
            i += 2;
            continue;
        }
        spaced.push(chunks[i].clone());
        i += 1;
    }

    let mut out = Vec::with_capacity(spaced.len());
    let mut i = 0;
    while i < spaced.len() {
        let peel = spaced[i].hit.is_none()
            && spaced[i].text.ends_with(' ')
            && spaced[i].text.chars().any(|ch| ch != ' ');
        if peel {
            if let Some(click) = labeled_chip_at(&spaced, i + 1) {
                let mut prefix = spaced[i].clone();
                prefix.text.pop();
                out.push(prefix);
                out.push(Chunk::plain_hit(" ", click));
                i += 1;
                continue;
            }
        }
        out.push(spaced[i].clone());
        i += 1;
    }
    out
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
    // A labeled chip reverses the key and the name together. A key with no
    // name still flips from reversed to normal.
    let labeled = active.is_some_and(|click| click_has_label(chunks, click));
    let mut out = String::new();
    for chunk in chunks {
        out.push_str("\u{1b}[0m");
        let same = chunk.hit.is_some() && chunk.hit == active;
        let params = if same && labeled {
            format!("22;{};{}", chrome.dark.fg(), chrome.light.bg())
        } else {
            let invert = same;
            match (chunk.style, invert) {
                (Style::Plain, false) => format!("22;{};{}", chrome.light.fg(), chrome.bar.bg()),
                (Style::Plain, true) => format!("22;{};{}", chrome.bar.fg(), chrome.light.bg()),
                (Style::Bold, false) => format!("1;{};{}", chrome.ctrl.fg(), chrome.bar.bg()),
                (Style::Bold, true) => format!("1;{};{}", chrome.bar.fg(), chrome.ctrl.bg()),
                (Style::BoldOrange, false) => format!("22;{};{}", chrome.alt.fg(), chrome.bar.bg()),
                (Style::BoldOrange, true) => format!("22;{};{}", chrome.bar.fg(), chrome.alt.bg()),
                (Style::Reverse, false) => format!("22;{};{}", chrome.dark.fg(), chrome.light.bg()),
                (Style::Reverse, true) => format!("22;{};{}", chrome.light.fg(), chrome.dark.bg()),
            }
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
    chunks: Vec<Chunk>,
    cols: usize,
    chrome: &Chrome,
    hover: Option<usize>,
) -> (String, Vec<Hit>) {
    let mut chunks = bind_chip_pads(&chunks);
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
            chip("N", "New", "New", Click::NewPane),
            keys("HJKL", "Change Focus", "Move", &FOCUS_HJKL),
            chip("X", "Close", "Close", Click::ClosePane),
            chip("C", "Rename", "Rename", Click::RenamePane),
            chip(
                "F",
                "Toggle Fullscreen",
                "Fullscreen",
                Click::ToggleFullscreen,
            ),
            chip("W", "Toggle Floating", "Floating", Click::ToggleFloating),
            chip("E", "Toggle Embed", "Embed", Click::ToggleEmbed),
            chip("R", "Split Right", "Right", Click::NewPaneRight),
            chip("D", "Split Down", "Down", Click::NewPaneDown),
            chip("S", "Stack", "Stack", Click::NewStacked),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tab" => vec![
            chip("N", "New", "New", Click::NewTab),
            keys("HL", "Change Focus", "Move", &TAB_HL),
            chip("X", "Close", "Close", Click::CloseTab),
            chip("R", "Rename", "Rename", Click::RenameTab),
            chip("S", "Sync", "Sync", Click::SyncTab),
            chip("B", "Break Pane To New Tab", "Break Out", Click::BreakPane),
            keys("[]", "Break Pane Left/Right", "Break", &BREAK_BRACKETS),
            chip("TAB", "Toggle", "Toggle", Click::ToggleTab),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "resize" => vec![
            keys(
                "+-",
                "Increase/Decrease Size",
                "Increase/Decrease",
                &RESIZE_PM,
            ),
            keys("HJKL", "Increase To", "Increase", &RESIZE_INC),
            keys("HJKL", "Decrease From", "Decrease", &RESIZE_DEC),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "move" => vec![
            keys("HJKL", "Switch Location", "Move", &MOVE_HJKL),
            chip("ENTER", "When Done", "Back", Click::SelectPane),
        ],
        "scroll" => vec![
            chip("S", "Enter Search Term", "Search", Click::EnterSearch),
            keys("HJKL", "Scroll", "Scroll", &SCROLL_HJKL),
            chip(
                "E",
                "Edit Scrollback In Default Editor",
                "Edit",
                Click::EditScrollback,
            ),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "enter_search" => vec![
            chip("ENTER", "When Done", "Done", Click::ConfirmSearch),
            chip("ESC", "Cancel", "Cancel", Click::CancelSearch),
        ],
        "search" => vec![
            chip("N", "Search Down", "Down", Click::SearchDown),
            chip("P", "Search Up", "Up", Click::SearchUp),
            chip("C", "Case Sensitive", "Case", Click::SearchCase),
            chip("W", "Wrap", "Wrap", Click::SearchWrap),
            chip("O", "Whole Words", "Whole", Click::SearchWhole),
        ],
        "session" => vec![
            chip("D", "Detach", "Detach", Click::Detach),
            chip("W", "Session Manager", "Manager", Click::SessionManager),
            chip("S", "Share", "Share", Click::Share),
            chip("C", "Configure", "Config", Click::Configuration),
            chip("L", "Layout Manager", "Layouts", Click::LayoutManager),
            chip("P", "Plugin Manager", "Plugins", Click::PluginManager),
            chip("A", "About", "About", Click::About),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tmux" => vec![
            keys("HJKL", "Move Focus", "Move", &FOCUS_HJKL),
            chip("\"", "Split Down", "Down", Click::NewPaneDown),
            chip("%", "Split Right", "Right", Click::NewPaneRight),
            chip("Z", "Fullscreen", "Fullscreen", Click::ToggleFullscreen),
            chip("C", "New Tab", "New", Click::NewTab),
            chip(",", "Rename Tab", "Rename", Click::RenameTab),
            chip("P", "Previous Tab", "Previous", Click::TabPrev),
            chip("N", "Next Tab", "Next", Click::TabNext),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "rename_pane" => vec![chip("ESC", "When Done", "Done", Click::DoneRenamePane)],
        "rename_tab" => vec![chip("ESC", "When Done", "Done", Click::DoneRenameTab)],
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
        let actions = mode_actions(mode);
        let mut options = variants(prefix.clone(), &actions, true);
        let mut compact = Vec::new();
        if let Some(first) = actions.first() {
            let mut row = prefix.clone();
            row.extend(token_chunks(first));
            compact.push(row);
        }
        compact.push(prefix);
        compact.push(vec![Chunk::orange(heading)]);
        match options.iter().position(|row| plain_text(row) == "...") {
            Some(i) => {
                for row in compact.into_iter().rev() {
                    options.insert(i, row);
                }
            }
            None => options.extend(compact),
        }
        let chosen = first_fit(&options, cols);
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

    fn cell_sgr(rendered: &str) -> Vec<String> {
        let mut cells = Vec::new();
        let mut params = String::new();
        let mut chars = rendered.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    let mut seq = String::new();
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                        seq.push(c);
                    }
                    if seq == "0" {
                        params.clear();
                    } else {
                        params = seq;
                    }
                }
                continue;
            }
            cells.push(params.clone());
        }
        cells
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
        assert!(raw.contains("\u{1b}[1;38;5;255;48;5;16m Ctrl +"));
        assert!(raw.contains("\u{1b}[22;38;5;166;48;5;16mAlt +"));
        assert!(raw.contains("\u{1b}[22;38;5;16;48;5;245m^G"));
        assert!(raw.contains("\u{1b}[22;38;5;245;48;5;16m Lock"));
        assert!(!raw.contains("\u{1b}[1;38;5;166m"));
        let text = visible(&raw);
        assert!(text.contains(" Ctrl + ^G Lock"), "{text}");
        assert!(text.contains("Alt + ⌥N"), "{text}");
        assert!(!text.contains("Ctrl +  ^"), "{text}");
        assert!(!text.contains("Alt +  ⌥"), "{text}");
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
    fn resize_chip_includes_the_side_spaces() {
        let hits = render_bar_with("normal", 220, false, &Chrome::default(), None).1;
        let hit = hits
            .iter()
            .find(|h| h.click == Click::Mode("resize"))
            .expect("resize chip");
        let text = shown("normal", 220, false);
        let slice: String = text
            .chars()
            .skip(hit.start)
            .take(hit.end - hit.start)
            .collect();
        assert_eq!(slice, " ^N Resize ");
        assert_eq!(hit.end - hit.start, 11);
        let tab = hits
            .iter()
            .find(|h| h.click == Click::Mode("tab"))
            .expect("tab chip");
        assert_eq!(tab.end, hit.start, "the gap between chips is not dead");
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
    fn hover_reverses_the_whole_labeled_chip() {
        let chrome = Chrome {
            bar: Ink::Bit(232),
            ..Chrome::default()
        };
        let reverse = format!("22;{};{}", chrome.dark.fg(), chrome.light.bg());
        let (idle, hits) = render_bar_with("normal", 220, false, &chrome, None);
        let resize = hits
            .iter()
            .find(|h| h.click == Click::Mode("resize"))
            .unwrap();
        let idle_cells = cell_sgr(&idle);
        for col in resize.start..resize.end {
            let hovered = render_bar_with("normal", 220, false, &chrome, Some(col)).0;
            let cells = cell_sgr(&hovered);
            assert!(
                cells[resize.start..resize.end]
                    .iter()
                    .all(|cell| cell == &reverse),
                "hover {col} left part of the chip unreversed"
            );
            if resize.start > 0 {
                assert_ne!(cells[resize.start - 1], reverse);
            }
            if resize.end < cells.len() {
                assert_ne!(cells[resize.end], reverse);
            }
        }
        let name_at = resize.start + 4;
        assert_ne!(idle_cells[name_at], reverse, "the name starts reversed");
        assert!(!idle.contains("38;5;255;48;5;255"));
    }

    #[test]
    fn key_only_hover_turns_the_key_normal() {
        let chrome = Chrome {
            bar: Ink::Bit(232),
            ..Chrome::default()
        };
        let mut found = false;
        for cols in 20..80 {
            let text = shown("normal", cols, false);
            if !text.contains("^G  ^P  ") || text.contains("Lock") {
                continue;
            }
            let (idle, hits) = render_bar_with("normal", cols, false, &chrome, None);
            let lock = hits
                .iter()
                .find(|h| h.click == Click::Mode("locked"))
                .expect("lock key");
            let slice: String = text
                .chars()
                .skip(lock.start)
                .take(lock.end - lock.start)
                .collect();
            assert_eq!(slice, "^G", "{text}");
            assert!(text[lock.end..].starts_with("  "), "{text}");
            let normal = format!("22;{};{}", chrome.light.fg(), chrome.dark.bg());
            let plain = format!("22;{};{}", chrome.light.fg(), chrome.bar.bg());
            let hovered = render_bar_with("normal", cols, false, &chrome, Some(lock.start)).0;
            let cells = cell_sgr(&hovered);
            assert_eq!(cells[lock.start], normal);
            assert_eq!(cells[lock.start + 1], normal);
            assert_eq!(
                cells[lock.end], plain,
                "the space after a key is not part of it"
            );
            assert_ne!(cell_sgr(&idle)[lock.start], normal);
            found = true;
            break;
        }
        assert!(found, "no key-only row");
    }

    #[test]
    fn ctrl_space_is_the_first_chips_left_side() {
        let text = shown("normal", 220, false);
        let hits = render_bar_with("normal", 220, false, &Chrome::default(), None).1;
        let lock = hits
            .iter()
            .find(|h| h.click == Click::Mode("locked"))
            .unwrap();
        let slice: String = text
            .chars()
            .skip(lock.start)
            .take(lock.end - lock.start)
            .collect();
        assert!(slice.starts_with(" ^G Lock"), "{slice}");
        assert!(text[..lock.start].ends_with("Ctrl +"), "{text}");
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
                raw.contains("\u{1b}[1;38;5;255;48;5;16m Ctrl +"),
                "{mode} should keep a bold Ctrl + after the heading"
            );
        }

        let session = shown("session", 220, false);
        assert!(session.contains("D Detach"), "{session}");
        assert!(session.contains("L Layout Manager"), "{session}");
        assert!(
            !session.contains('^'),
            "in-mode keys are unchorded: {session}"
        );

        let tab = shown("tab", 220, false);
        assert!(tab.contains("N New"), "{tab}");
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

    #[test]
    fn session_layout_manager_chip_is_clickable() {
        let hits = render_bar_with("session", 220, false, &Chrome::default(), None).1;
        let hit = hits
            .iter()
            .find(|h| h.click == Click::LayoutManager)
            .expect("layout manager chip");
        let text = shown("session", 220, false);
        let slice: String = text
            .chars()
            .skip(hit.start)
            .take(hit.end - hit.start)
            .collect();
        assert!(
            slice.contains('L') && slice.contains("Layout Manager"),
            "hit {hit:?} covers {slice:?} in {text}"
        );
    }

    #[test]
    fn narrow_active_mode_keeps_the_heading() {
        for (mode, name) in [
            ("pane", "Pane"),
            ("tab", "Tab"),
            ("resize", "Resize"),
            ("move", "Move"),
            ("scroll", "Search"),
            ("session", "Session"),
            ("tmux", "Tmux"),
        ] {
            let prefix_width = format!("{name} Ctrl + ").chars().count();
            let with_ctrl = shown(mode, prefix_width, false);
            assert!(
                with_ctrl.starts_with(&format!("{name} Ctrl +")),
                "{mode} at {prefix_width}: {with_ctrl}"
            );

            let name_width = name.chars().count();
            let headed = shown(mode, name_width, false);
            assert!(headed.starts_with(name), "{mode} at {name_width}: {headed}");
        }

        // Session prefix is 15 cells; the old shortest headed form was 21.
        let session_15 = shown("session", 15, false);
        assert!(session_15.starts_with("Session Ctrl +"), "{session_15}");
        assert!(!session_15.contains("..."), "{session_15}");

        let session_16 = shown("session", 16, false);
        assert!(session_16.starts_with("Session Ctrl + D"), "{session_16}");
        assert!(!session_16.contains("..."), "{session_16}");

        let normal_10 = shown("normal", 10, false);
        assert!(
            normal_10.starts_with("..."),
            "normal at 10 should stay on the ellipsis: {normal_10}"
        );
        let locked_10 = shown("locked", 10, false);
        assert!(
            locked_10.starts_with("..."),
            "locked at 10 should stay on the ellipsis: {locked_10}"
        );
    }
}
