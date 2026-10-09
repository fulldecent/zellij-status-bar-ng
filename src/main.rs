use std::collections::BTreeMap;
use zellij_tile::prelude::actions::{Action, SearchDirection, SearchOption};
use zellij_tile::prelude::*;

use plugin::{
    chrome_from, render_status_hits, render_status_line, Chrome, Click, Hit, StatusState,
};

struct State {
    mode: InputMode,
    multi_pane: bool,
    chrome: Chrome,
    hover_col: Option<usize>,
    hits: Vec<Hit>,
}

register_plugin!(State);

impl Default for State {
    fn default() -> Self {
        Self {
            mode: InputMode::Normal,
            multi_pane: false,
            chrome: Chrome::default(),
            hover_col: None,
            hits: Vec::new(),
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunActionsAsUser,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(status) => {
                if status == PermissionStatus::Granted {
                    set_selectable(false);
                }
                true
            }
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
            Event::Mouse(Mouse::Hover(_, col)) => {
                if self.hover_col == Some(col) {
                    false
                } else {
                    self.hover_col = Some(col);
                    true
                }
            }
            Event::Mouse(Mouse::LeftClick(_, col)) => {
                if let Some(hit) = self.hits.iter().find(|h| col >= h.start && col < h.end) {
                    run_click(hit.click);
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let state = StatusState {
            mode: mode_name(self.mode),
            multi_pane: self.multi_pane,
            chrome: self.chrome,
            hover_col: self.hover_col,
        };
        self.hits = render_status_hits(cols, &state);
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

fn input_mode(name: &str) -> Option<InputMode> {
    Some(match name {
        "normal" => InputMode::Normal,
        "locked" => InputMode::Locked,
        "pane" => InputMode::Pane,
        "tab" => InputMode::Tab,
        "resize" => InputMode::Resize,
        "move" => InputMode::Move,
        "scroll" => InputMode::Scroll,
        "enter_search" => InputMode::EnterSearch,
        "search" => InputMode::Search,
        "rename_tab" => InputMode::RenameTab,
        "rename_pane" => InputMode::RenamePane,
        "session" => InputMode::Session,
        "tmux" => InputMode::Tmux,
        _ => return None,
    })
}

fn launch(url: &str) {
    start_or_reload_plugin(url);
    switch_to_input_mode(&InputMode::Normal);
}

fn run_click(click: Click) {
    match click {
        Click::Mode(name) => {
            if let Some(mode) = input_mode(name) {
                switch_to_input_mode(&mode);
            }
        }
        Click::Quit => quit_zellij(),
        Click::NewPane => {
            new_pane();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::NewPaneDown => {
            run_action(
                Action::NewTiledPane {
                    direction: Some(Direction::Down),
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    borderless: None,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::NewPaneRight => {
            run_action(
                Action::NewTiledPane {
                    direction: Some(Direction::Right),
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    borderless: None,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::NewStacked => {
            run_action(
                Action::NewStackedPane {
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::FocusLeft => move_focus(Direction::Left),
        Click::FocusDown => move_focus(Direction::Down),
        Click::FocusUp => move_focus(Direction::Up),
        Click::FocusRight => move_focus(Direction::Right),
        Click::ResizeIncrease => resize_focused_pane(Resize::Increase),
        Click::ResizeDecrease => resize_focused_pane(Resize::Decrease),
        Click::ResizeIncreaseLeft => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Left)
        }
        Click::ResizeIncreaseDown => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Down)
        }
        Click::ResizeIncreaseUp => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Up)
        }
        Click::ResizeIncreaseRight => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Right)
        }
        Click::ResizeDecreaseLeft => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Left)
        }
        Click::ResizeDecreaseDown => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Down)
        }
        Click::ResizeDecreaseUp => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Up)
        }
        Click::ResizeDecreaseRight => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Right)
        }
        Click::MovePaneLeft => move_pane_with_direction(Direction::Left),
        Click::MovePaneDown => move_pane_with_direction(Direction::Down),
        Click::MovePaneUp => move_pane_with_direction(Direction::Up),
        Click::MovePaneRight => move_pane_with_direction(Direction::Right),
        Click::ToggleFloating => {
            toggle_floating_panes(None);
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::ToggleEmbed => {
            toggle_pane_embed_or_eject();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::ToggleFullscreen => {
            toggle_focus_fullscreen();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::ClosePane => {
            close_focus();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::CloseTab => {
            close_focused_tab();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::NewTab => {
            new_tab::<&str>(None, None);
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::RenamePane => switch_to_input_mode(&InputMode::RenamePane),
        Click::RenameTab => switch_to_input_mode(&InputMode::RenameTab),
        Click::SyncTab => {
            toggle_active_tab_sync();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::BreakPane => {
            run_action(Action::BreakPane, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::BreakPaneLeft => {
            run_action(Action::BreakPaneLeft, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::BreakPaneRight => {
            run_action(Action::BreakPaneRight, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::ToggleTab => toggle_tab(),
        Click::SelectPane => switch_to_input_mode(&InputMode::Normal),
        Click::Detach => detach(),
        Click::SessionManager => launch("zellij:session-manager"),
        Click::Configuration => launch("zellij:configuration"),
        Click::PluginManager => launch("zellij:plugin-manager"),
        Click::About => launch("zellij:about"),
        Click::Share => launch("zellij:share"),
        Click::LayoutManager => launch("zellij:layout-manager"),
        Click::EditScrollback => {
            edit_scrollback();
            switch_to_input_mode(&InputMode::Normal);
        }
        Click::ScrollUp => scroll_up(),
        Click::ScrollDown => scroll_down(),
        Click::PageScrollUp => page_scroll_up(),
        Click::PageScrollDown => page_scroll_down(),
        Click::EnterSearch => switch_to_input_mode(&InputMode::EnterSearch),
        Click::CancelSearch => switch_to_input_mode(&InputMode::Scroll),
        Click::ConfirmSearch => switch_to_input_mode(&InputMode::Search),
        Click::SearchDown => run_action(
            Action::Search {
                direction: SearchDirection::Down,
            },
            BTreeMap::new(),
        ),
        Click::SearchUp => run_action(
            Action::Search {
                direction: SearchDirection::Up,
            },
            BTreeMap::new(),
        ),
        Click::SearchCase => run_action(
            Action::SearchToggleOption {
                option: SearchOption::CaseSensitivity,
            },
            BTreeMap::new(),
        ),
        Click::SearchWrap => run_action(
            Action::SearchToggleOption {
                option: SearchOption::Wrap,
            },
            BTreeMap::new(),
        ),
        Click::SearchWhole => run_action(
            Action::SearchToggleOption {
                option: SearchOption::WholeWord,
            },
            BTreeMap::new(),
        ),
        Click::TabPrev => go_to_previous_tab(),
        Click::TabNext => go_to_next_tab(),
        Click::DoneRenamePane => {
            undo_rename_pane();
            switch_to_input_mode(&InputMode::Pane);
        }
        Click::DoneRenameTab => {
            undo_rename_tab();
            switch_to_input_mode(&InputMode::Tab);
        }
    }
}
