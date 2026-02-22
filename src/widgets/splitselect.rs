use crate::widgets::pianoroll::PianoRollState;
use crate::window::{Window, WindowManager};
use crate::input::LocalCommand;
use crate::widgets::theme::UIStyle;
use crate::widgets::buttonlist::{ButtonList, ButtonListState, Button};

use ratatui::style::{Modifier, Styled};
use ratatui::widgets::Paragraph;
use ratatui::{
    Frame,
    layout::{Rect, Direction, Layout, Constraint},
    style::{Style, Color},
    text::Line,
    widgets::Clear,
};

pub struct SplitSelect<'a> {
    list_state: ButtonListState<'a>,
}

impl<'a> SplitSelect<'a> {
    pub fn new() -> Self {
        let mut buttons = Vec::new();
        for i in 0..5 {
            buttons.push(Button {
                label: Line::from(format!("Button {}", i)).centered(),
                height: 1,
                style: Style::default(),
            });
        }

        Self { list_state: ButtonListState::new(buttons).hovered(Some(0)) }
    }
}

impl Window for SplitSelect<'_> {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let current = self.list_state.get_hovered().unwrap_or(0) + 1;
        let block = UIStyle::window_border("New Window", focused)
            .title_bottom(
            Line::from(format!(
                "{} of {}",
                current, self.list_state.get_num_items()
            )).right_aligned());

        let list_area = UIStyle::centered_rect(50, 50, area);
        frame.render_widget(Clear, block.inner(list_area));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .split(block.inner(list_area));

        let list = ButtonList::new()
            .padding(1)
            .style(Style::default().fg(Color::White))
            .hovered_style(Style::default().add_modifier(Modifier::REVERSED))
            .hovered_character("🡒");

        frame.render_widget(&block, list_area);
        frame.render_widget(Paragraph::new("Lorum Ipsum")
            .style(Style::default().fg(Color::White))
            .centered(), layout[0]);
        frame.render_stateful_widget(list, layout[1], &mut self.list_state);
    }

    fn handle_input(&mut self, cmd: LocalCommand) {
        match cmd {
            LocalCommand::MoveLocalCursor { dx: _, dy } => {
                self.list_state.jump_buttons(-dy);
            },

            LocalCommand::Confirm => {
                let i = self.list_state.get_hovered();
                self.list_state.set_selected(i);
            }
            
            _ => (),
        }
    }
}

