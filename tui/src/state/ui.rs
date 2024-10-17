use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, Paragraph},
    Frame,
};

use crate::hint::create_shortcut_list;

use super::{AppState, Focus, ListEntry};

const MIN_HEIGHT: u16 = 19;
const MIN_WIDTH: u16 = 77;
const TITLE: &str = concat!("Linux Toolbox - ", env!("BUILD_DATE"));

impl AppState {
    pub fn draw(&mut self, frame: &mut Frame) {
        let terminal_size = frame.area();

        if !self.tui_drawable(frame, terminal_size) {
            return;
        }

        let label_block =
            Block::default()
                .borders(Borders::all())
                .border_set(ratatui::symbols::border::Set {
                    top_left: " ",
                    top_right: " ",
                    bottom_left: " ",
                    bottom_right: " ",
                    vertical_left: " ",
                    vertical_right: " ",
                    horizontal_top: "*",
                    horizontal_bottom: "*",
                });
        let str1 = "Linutil ";
        let str2 = "by Chris Titus";
        let label = Paragraph::new(Line::from(vec![
            Span::styled(str1, Style::default().bold()),
            Span::styled(str2, Style::default().italic()),
        ]))
        .block(label_block)
        .alignment(Alignment::Center);

        let longest_tab_display_len = self
            .tabs
            .iter()
            .map(|tab| tab.name.len() + self.theme.tab_icon().len())
            .max()
            .unwrap_or(0)
            .max(str1.len() + str2.len());

        let (keybind_scope, shortcuts) = self.get_keybinds();

        let keybind_render_width = terminal_size.width - 2;

        let keybinds_block = Block::default()
            .title(format!(" {} ", keybind_scope))
            .borders(Borders::all());

        let keybinds = create_shortcut_list(shortcuts, keybind_render_width);
        let n_lines = keybinds.len() as u16;

        let keybind_para = Paragraph::new(Text::from_iter(keybinds)).block(keybinds_block);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(0),
                Constraint::Max(n_lines as u16 + 2),
            ])
            .flex(Flex::Legacy)
            .margin(0)
            .split(frame.area());

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(longest_tab_display_len as u16 + 5),
                Constraint::Percentage(100),
            ])
            .split(vertical[0]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(horizontal[0]);
        frame.render_widget(label, left_chunks[0]);

        let tabs = self
            .tabs
            .iter()
            .map(|tab| tab.name.as_str())
            .collect::<Vec<_>>();

        let tab_hl_style = if let Focus::TabList = self.focus {
            Style::default().reversed().fg(self.theme.tab_color())
        } else {
            Style::new().fg(self.theme.tab_color())
        };

        let list = List::new(tabs)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(tab_hl_style)
            .highlight_symbol(self.theme.tab_icon());
        frame.render_stateful_widget(list, left_chunks[1], &mut self.current_tab);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(horizontal[1]);

        let list_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(chunks[1]);

        self.filter.draw_searchbar(frame, chunks[0], &self.theme);

        let mut items: Vec<Line> = Vec::new();
        let mut task_items: Vec<Line> = Vec::new();

        if !self.at_root() {
            items.push(
                Line::from(format!("{}  ..", self.theme.dir_icon())).style(self.theme.dir_color()),
            );
            task_items.push(Line::from(" ").style(self.theme.dir_color()));
        }

        items.extend(self.filter.item_list().iter().map(
            |ListEntry {
                 node, has_children, ..
             }| {
                let is_selected = self.selected_commands.contains(node);
                let (indicator, style) = if is_selected {
                    (self.theme.multi_select_icon(), Style::default().bold())
                } else {
                    ("", Style::new())
                };
                if *has_children {
                    Line::from(format!(
                        "{}  {} {}",
                        self.theme.dir_icon(),
                        node.name,
                        indicator
                    ))
                    .style(self.theme.dir_color())
                } else {
                    Line::from(format!(
                        "{}  {} {}",
                        self.theme.cmd_icon(),
                        node.name,
                        indicator
                    ))
                    .style(self.theme.cmd_color())
                    .patch_style(style)
                }
            },
        ));

        task_items.extend(self.filter.item_list().iter().map(
            |ListEntry {
                 node, has_children, ..
             }| {
                if *has_children {
                    Line::from(" ").style(self.theme.dir_color())
                } else {
                    Line::from(format!("{} ", node.task_list))
                        .alignment(Alignment::Right)
                        .style(self.theme.cmd_color())
                        .bold()
                }
            },
        ));

        let style = if let Focus::List = self.focus {
            Style::default().reversed()
        } else {
            Style::new()
        };

        let title = if self.multi_select {
            &format!("{} [Multi-Select]", TITLE)
        } else {
            TITLE
        };

        #[cfg(feature = "tips")]
        let bottom_title = Line::from(self.tip.bold().blue()).right_aligned();
        #[cfg(not(feature = "tips"))]
        let bottom_title = "";

        let task_list_title = Line::from("Important Actions ").right_aligned();

        // Create the list widget with items
        let list = List::new(items)
            .highlight_style(style)
            .block(
                Block::default()
                    .borders(Borders::ALL & !Borders::RIGHT)
                    .title(title)
                    .title_bottom(bottom_title),
            )
            .scroll_padding(1);
        frame.render_stateful_widget(list, list_chunks[0], &mut self.selection);

        let disclaimer_list = List::new(task_items).highlight_style(style).block(
            Block::default()
                .borders(Borders::ALL & !Borders::LEFT)
                .title(task_list_title),
        );

        frame.render_stateful_widget(disclaimer_list, list_chunks[1], &mut self.selection);

        match &mut self.focus {
            Focus::FloatingWindow(float) => float.draw(frame, chunks[1]),
            Focus::ConfirmationPrompt(prompt) => prompt.draw(frame, chunks[1]),
            _ => {}
        }

        frame.render_widget(keybind_para, vertical[1]);
    }

    fn tui_drawable(&mut self, frame: &mut Frame, terminal_size: Rect) -> bool {
        if terminal_size.width < MIN_WIDTH || terminal_size.height < MIN_HEIGHT {
            let warning = Paragraph::new(format!(
                "Terminal size too small:\nWidth = {} Height = {}\n\nMinimum size:\nWidth = {}  Height = {}",
                terminal_size.width,
                terminal_size.height,
                MIN_WIDTH,
                MIN_HEIGHT,
            ))
                .alignment(Alignment::Center)
                .style(Style::default().fg(ratatui::style::Color::Red).bold())
                .wrap(ratatui::widgets::Wrap { trim: true });

            let centered_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                ])
                .split(terminal_size);

            frame.render_widget(warning, centered_layout[1]);
            self.drawable = false;
            false
        } else {
            self.drawable = true;
            true
        }
    }
}
