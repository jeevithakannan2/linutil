use super::{AppState, Focus};
use crate::{confirmation::ConfirmStatus, filter::SearchAction, float::FloatContent};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl AppState {
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        // This should be defined first to allow closing
        // the application even when not drawable ( If terminal is small )
        // Exit on 'q' or 'Ctrl-c' input
        if matches!(
            self.focus,
            Focus::TabList | Focus::List | Focus::ConfirmationPrompt(_)
        ) && (key.code == KeyCode::Char('q')
            || key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            return false;
        }

        // If UI is not drawable returning true will mark as the key handled
        if !self.drawable {
            return true;
        }

        // Handle key only when Tablist or List is focused
        // Prevents exiting the application even when a command is running
        // Add keys here which should work on both TabList and List
        if matches!(self.focus, Focus::TabList | Focus::List) {
            match key.code {
                KeyCode::BackTab => {
                    if self.current_tab.selected().unwrap() == 0 {
                        self.current_tab.select(Some(self.tabs.len() - 1));
                    } else {
                        self.current_tab.select_previous();
                    }
                    self.refresh_tab();
                }

                KeyCode::Tab => {
                    if self.current_tab.selected().unwrap() == self.tabs.len() - 1 {
                        self.current_tab.select_first();
                    } else {
                        self.current_tab.select_next();
                    }
                    self.refresh_tab();
                }
                _ => {}
            }
        }

        match &mut self.focus {
            Focus::FloatingWindow(command) => {
                if command.handle_key_event(key) {
                    self.focus = Focus::List;
                }
            }

            Focus::ConfirmationPrompt(confirm) => {
                confirm.content.handle_key_event(key);
                match confirm.content.status {
                    ConfirmStatus::Abort => {
                        self.focus = Focus::List;
                        // selected command was pushed to selection list if multi-select was
                        // enabled, need to clear it to prevent state corruption
                        if !self.multi_select {
                            self.selected_commands.clear()
                        }
                    }
                    ConfirmStatus::Confirm => self.handle_confirm_command(),
                    ConfirmStatus::None => {}
                }
            }

            Focus::Search => match self.filter.handle_key(key) {
                SearchAction::Exit => self.exit_search(),
                SearchAction::Update => self.update_items(),
                SearchAction::None => {}
            },

            Focus::TabList => match key.code {
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.focus = Focus::List,

                KeyCode::Char('j') | KeyCode::Down
                    if self.current_tab.selected().unwrap() + 1 < self.tabs.len() =>
                {
                    self.current_tab.select_next();
                    self.refresh_tab();
                }

                KeyCode::Char('k') | KeyCode::Up => {
                    self.current_tab.select_previous();
                    self.refresh_tab();
                }

                KeyCode::Char('/') => self.enter_search(),
                KeyCode::Char('t') => self.theme.next(),
                KeyCode::Char('T') => self.theme.prev(),
                KeyCode::Char('g') => self.toggle_task_list_guide(),
                _ => {}
            },

            Focus::List if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.selection.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.selection.select_previous(),
                KeyCode::Char('p') | KeyCode::Char('P') => self.enable_preview(),
                KeyCode::Char('d') | KeyCode::Char('D') => self.enable_description(),
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => self.handle_enter(),
                KeyCode::Char('h') | KeyCode::Left => self.go_back(),
                KeyCode::Char('/') => self.enter_search(),
                KeyCode::Char('t') => self.theme.next(),
                KeyCode::Char('T') => self.theme.prev(),
                KeyCode::Char('g') => self.toggle_task_list_guide(),
                KeyCode::Char('v') | KeyCode::Char('V') => self.toggle_multi_select(),
                KeyCode::Char(' ') if self.multi_select => self.toggle_selection(),
                _ => {}
            },

            _ => (),
        };
        true
    }
}
