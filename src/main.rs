use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use plugin::{chrome_from, render_status_line, Chrome, StatusState};

struct State {
    mode: InputMode,
    multi_pane: bool,
    chrome: Chrome,
}

register_plugin!(State);

impl Default for State {
    fn default() -> Self {
        Self {
            mode: InputMode::Normal,
            multi_pane: false,
            chrome: Chrome::default(),
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // Same as the built-in status bar: this is chrome, not a pane. If it
        // stays selectable, Ctrl-D leaves the tab open and this bar slides up
        // into the space the shell vacated.
        set_selectable(false);
        request_permission(&[PermissionType::ReadApplicationState]);
        subscribe(&[EventType::ModeUpdate, EventType::TabUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(info) => {
                self.mode = info.mode;
                self.chrome = chrome_from(&info.style);
                true
            }
            Event::TabUpdate(tabs) => {
                self.multi_pane = tabs
                    .iter()
                    .find(|tab| tab.active)
                    .map(|tab| {
                        if tab.are_floating_panes_visible {
                            tab.selectable_floating_panes_count > 1
                        } else {
                            tab.selectable_tiled_panes_count > 1
                        }
                    })
                    .unwrap_or(false);
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let state = StatusState {
            mode: mode_name(self.mode),
            multi_pane: self.multi_pane,
            chrome: self.chrome,
        };
        print!("{}", render_status_line(cols, &state));
    }
}

fn mode_name(mode: InputMode) -> &'static str {
    match mode {
        InputMode::Normal => "normal",
        InputMode::Locked => "locked",
        InputMode::Pane => "pane",
        InputMode::Tab => "tab",
        InputMode::Resize => "resize",
        InputMode::Move => "move",
        InputMode::Scroll => "scroll",
        InputMode::EnterSearch => "enter_search",
        InputMode::Search => "search",
        InputMode::RenameTab => "rename_tab",
        InputMode::RenamePane => "rename_pane",
        InputMode::Session => "session",
        InputMode::Tmux => "tmux",
        _ => "prompt",
    }
}
