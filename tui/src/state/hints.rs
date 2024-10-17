use crate::hint::Shortcut;

use super::{AppState, Focus};

impl AppState {
    pub(super) fn get_keybinds(&self) -> (&str, Box<[Shortcut]>) {
        match self.focus {
            Focus::Search => (
                "Search bar",
                Box::new([Shortcut::new("Finish search", ["Enter"])]),
            ),

            Focus::List => {
                let mut hints = Vec::new();
                hints.push(Shortcut::new("Exit linutil", ["q", "CTRL-c"]));

                if self.at_root() {
                    hints.push(Shortcut::new("Focus tab list", ["h", "Left"]));
                    hints.extend(self.get_list_item_shortcut());
                } else if self.selected_item_is_up_dir() {
                    hints.push(Shortcut::new(
                        "Go to parent directory",
                        ["l", "Right", "Enter", "h", "Left"],
                    ));
                } else {
                    hints.push(Shortcut::new("Go to parent directory", ["h", "Left"]));
                    hints.extend(self.get_list_item_shortcut());
                }

                hints.push(Shortcut::new("Select item above", ["k", "Up"]));
                hints.push(Shortcut::new("Select item below", ["j", "Down"]));
                hints.push(Shortcut::new("Next theme", ["t"]));
                hints.push(Shortcut::new("Previous theme", ["T"]));

                if self.is_current_tab_multi_selectable() {
                    hints.push(Shortcut::new("Toggle multi-selection mode", ["v"]));
                    hints.push(Shortcut::new("Select multiple commands", ["Space"]));
                }

                hints.push(Shortcut::new("Next tab", ["Tab"]));
                hints.push(Shortcut::new("Previous tab", ["Shift-Tab"]));
                hints.push(Shortcut::new("Important actions guide", ["g"]));

                ("Command list", hints.into_boxed_slice())
            }

            Focus::TabList => (
                "Tab list",
                Box::new([
                    Shortcut::new("Exit linutil", ["q", "CTRL-c"]),
                    Shortcut::new("Focus action list", ["l", "Right", "Enter"]),
                    Shortcut::new("Select item above", ["k", "Up"]),
                    Shortcut::new("Select item below", ["j", "Down"]),
                    Shortcut::new("Next theme", ["t"]),
                    Shortcut::new("Previous theme", ["T"]),
                    Shortcut::new("Next tab", ["Tab"]),
                    Shortcut::new("Previous tab", ["Shift-Tab"]),
                ]),
            ),

            Focus::FloatingWindow(ref float) => float.get_shortcut_list(),
            Focus::ConfirmationPrompt(ref prompt) => prompt.get_shortcut_list(),
        }
    }

    fn get_list_item_shortcut(&self) -> Box<[Shortcut]> {
        if self.selected_item_is_dir() {
            Box::new([Shortcut::new("Go to selected dir", ["l", "Right", "Enter"])])
        } else {
            Box::new([
                Shortcut::new("Run selected command", ["l", "Right", "Enter"]),
                Shortcut::new("Enable preview", ["p"]),
                Shortcut::new("Command Description", ["d"]),
            ])
        }
    }
}
