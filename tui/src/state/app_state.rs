#[cfg(feature = "tips")]
use super::tips::get_random_tip;
use crate::{
    confirmation::ConfirmPrompt,
    filter::Filter,
    float::{Float, FloatContent},
    theme::Theme,
};

use ego_tree::NodeId;
use linutil_core::{ListNode, Tab};
use ratatui::widgets::ListState;
use std::rc::Rc;
use temp_dir::TempDir;

pub enum Focus {
    ConfirmationPrompt(Float<ConfirmPrompt>),
    FloatingWindow(Float<dyn FloatContent>),
    List,
    Search,
    TabList,
}

pub struct AppState {
    /// This must be passed to retain the temp dir until the end of the program
    _temp_dir: TempDir,
    /// Current tab
    pub(super) current_tab: ListState,
    /// Enough size to draw terminal
    pub(super) drawable: bool,
    pub(super) filter: Filter,
    /// Currently focused area
    pub(super) focus: Focus,
    /// Selected theme
    pub(super) multi_select: bool,
    pub(super) selected_commands: Vec<Rc<ListNode>>,
    /// This is the state associated with the list widget, used to display the selection in the
    /// widget
    pub(super) selection: ListState,
    /// List of tabs
    pub(super) tabs: Vec<Tab>,
    pub(super) theme: Theme,
    #[cfg(feature = "tips")]
    pub(super) tip: &'static str,
    /// This stack keeps track of our "current directory". You can think of it as `pwd`. but not
    /// just the current directory, all paths that took us here, so we can "cd .."
    pub(super) visit_stack: Vec<NodeId>,
}

pub struct ListEntry {
    pub has_children: bool,
    pub id: NodeId,
    pub node: Rc<ListNode>,
}

impl AppState {
    pub fn new(theme: Theme, override_validation: bool) -> Self {
        let (temp_dir, tabs) = linutil_core::get_tabs(!override_validation);
        let root_id = tabs[0].tree.root().id();

        let mut state = Self {
            _temp_dir: temp_dir,
            current_tab: ListState::default().with_selected(Some(0)),
            drawable: false,
            filter: Filter::new(),
            focus: Focus::List,
            multi_select: false,
            selected_commands: Vec::new(),
            selection: ListState::default().with_selected(Some(0)),
            tabs,
            theme,
            #[cfg(feature = "tips")]
            tip: get_random_tip(),
            visit_stack: vec![root_id],
        };

        state.update_items();
        state
    }
}
