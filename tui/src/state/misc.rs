use super::{AppState, Focus};
use crate::{
    confirmation::ConfirmPrompt,
    float::{Float, FloatContent},
    floating_text::FloatingText,
    running_command::RunningCommand,
};

use linutil_core::ListNode;
use std::rc::Rc;

const ACTIONS_GUIDE: &str = "List of important tasks performed by commands' names:

D  - disk modifications (ex. partitioning) (privileged)
FI - flatpak installation
FM - file modification
I  - installation (privileged)
MP - package manager actions
SI - full system installation
SS - systemd actions (privileged)
RP - package removal

P* - privileged *
";

impl AppState {
    pub(super) fn at_root(&self) -> bool {
        self.visit_stack.len() == 1
    }

    pub(super) fn enable_description(&mut self) {
        if let Some(command_description) = self.get_selected_description() {
            let description = FloatingText::new(command_description, "Command Description");
            self.spawn_float(description, 80, 80);
        }
    }

    pub(super) fn enable_preview(&mut self) {
        if let Some(list_node) = self.get_selected_node() {
            let mut preview_title = "[Preview] - ".to_string();
            preview_title.push_str(list_node.name.as_str());
            if let Some(preview) = FloatingText::from_command(&list_node.command, preview_title) {
                self.spawn_float(preview, 80, 80);
            }
        }
    }

    fn enter_parent_directory(&mut self) {
        self.visit_stack.pop();
        self.selection.select(Some(0));
        self.update_items();
    }

    pub(super) fn enter_search(&mut self) {
        self.focus = Focus::Search;
        self.filter.activate_search();
        self.selection.select(None);
    }

    pub(super) fn exit_search(&mut self) {
        self.selection.select(Some(0));
        self.focus = Focus::List;
        self.filter.deactivate_search();
        self.update_items();
    }

    fn get_selected_description(&self) -> Option<String> {
        self.get_selected_node()
            .map(|node| node.description.clone())
    }

    fn get_selected_node(&self) -> Option<Rc<ListNode>> {
        let mut selected_index = self.selection.selected().unwrap_or(0);

        if !self.at_root() && selected_index == 0 {
            return None;
        }
        if !self.at_root() {
            selected_index = selected_index.saturating_sub(1);
        }

        if let Some(item) = self.filter.item_list().get(selected_index) {
            if !item.has_children {
                return Some(item.node.clone());
            }
        }
        None
    }

    pub(super) fn go_back(&mut self) {
        if self.at_root() {
            self.focus = Focus::TabList;
        } else {
            self.enter_parent_directory();
        }
    }

    pub(super) fn go_to_selected_dir(&mut self) {
        let mut selected_index = self.selection.selected().unwrap_or(0);

        if !self.at_root() && selected_index == 0 {
            self.enter_parent_directory();
            return;
        }

        if !self.at_root() {
            selected_index = selected_index.saturating_sub(1);
        }

        if let Some(item) = self.filter.item_list().get(selected_index) {
            if item.has_children {
                self.visit_stack.push(item.id);
                self.selection.select(Some(0));
                self.update_items();
            }
        }
    }

    pub(super) fn handle_confirm_command(&mut self) {
        let commands = self
            .selected_commands
            .iter()
            .map(|node| node.command.clone())
            .collect();

        let command = RunningCommand::new(commands);
        self.spawn_float(command, 80, 80);
        self.selected_commands.clear();
    }

    pub(super) fn handle_enter(&mut self) {
        if self.selected_item_is_cmd() {
            if self.selected_commands.is_empty() {
                if let Some(node) = self.get_selected_node() {
                    self.selected_commands.push(node);
                }
            }

            let cmd_names = self
                .selected_commands
                .iter()
                .map(|node| node.name.as_str())
                .collect::<Vec<_>>();

            let prompt = ConfirmPrompt::new(&cmd_names[..]);
            self.focus = Focus::ConfirmationPrompt(Float::new(Box::new(prompt), 40, 40));
        } else {
            self.go_to_selected_dir();
        }
    }

    pub(super) fn is_current_tab_multi_selectable(&self) -> bool {
        let index = self.current_tab.selected().unwrap_or(0);
        self.tabs
            .get(index)
            .map_or(false, |tab| tab.multi_selectable)
    }

    pub(super) fn refresh_tab(&mut self) {
        self.visit_stack = vec![self.tabs[self.current_tab.selected().unwrap()]
            .tree
            .root()
            .id()];
        self.selection.select(Some(0));
        self.update_items();
    }

    pub(super) fn selected_item_is_dir(&self) -> bool {
        let mut selected_index = self.selection.selected().unwrap_or(0);

        if !self.at_root() && selected_index == 0 {
            return false;
        }

        if !self.at_root() {
            selected_index = selected_index.saturating_sub(1);
        }

        self.filter
            .item_list()
            .get(selected_index)
            .map_or(false, |item| item.has_children)
    }

    pub(super) fn selected_item_is_cmd(&self) -> bool {
        // Any item that is not a directory or up directory (..) must be a command
        self.selection.selected().is_some()
            && !(self.selected_item_is_up_dir() || self.selected_item_is_dir())
    }

    pub(super) fn selected_item_is_up_dir(&self) -> bool {
        let selected_index = self.selection.selected().unwrap_or(0);

        !self.at_root() && selected_index == 0
    }

    fn spawn_float<T: FloatContent + 'static>(&mut self, float: T, width: u16, height: u16) {
        self.focus = Focus::FloatingWindow(Float::new(Box::new(float), width, height));
    }

    pub(super) fn toggle_multi_select(&mut self) {
        if self.is_current_tab_multi_selectable() {
            self.multi_select = !self.multi_select;
            if !self.multi_select {
                self.selected_commands.clear();
            }
        }
    }

    pub(super) fn toggle_selection(&mut self) {
        if let Some(command) = self.get_selected_node() {
            if self.selected_commands.contains(&command) {
                self.selected_commands.retain(|c| c != &command);
            } else {
                self.selected_commands.push(command);
            }
        }
    }

    pub(super) fn toggle_task_list_guide(&mut self) {
        self.spawn_float(
            FloatingText::new(ACTIONS_GUIDE.to_string(), "Important Actions Guide"),
            80,
            80,
        );
    }

    pub(super) fn update_items(&mut self) {
        self.filter.update_items(
            &self.tabs,
            self.current_tab.selected().unwrap(),
            *self.visit_stack.last().unwrap(),
        );
        if !self.is_current_tab_multi_selectable() {
            self.multi_select = false;
            self.selected_commands.clear();
        }
    }
}
