use crate::window::Window;
use crate::input::LocalCommand;
use crate::widgets::theme::UIStyle;
use crate::widgets::buttonlist::{ButtonList, ButtonListState, Button};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::Line,
    widgets::{Block, Borders, BorderType, StatefulWidget},
};

#[derive(Default)]
pub struct SplitSelect<'a> {
    list_state: ButtonListState<'a>,
}

impl SplitSelect<'_> {
    pub fn new() -> Self {
        let mut buttons = Vec::new();
        for i in 0..5 {
            buttons.push(Button {
                label: Line::from(format!("Button {}", i)).centered(),
                height: 1,
                style: Style::default(),
            })
        }

        Self {
            list_state: ButtonListState::new(buttons),
        }
    }
}

impl Window for SplitSelect<'_> {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = UIStyle::window_border("", focused);
        frame.render_widget(&block, area);

        let list_area = UIStyle::centered_rect(50, 50, area);

        let list = ButtonList::new()
            .block(Block::new().borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::White));

        list.render(block.inner(list_area), frame.buffer_mut(), &mut self.list_state);
    }

    fn handle_input(&mut self, cmd: LocalCommand) {
        match cmd {
            LocalCommand::MoveLocalCursor { dx: _, dy } => {
                self.list_state.jump_buttons(-dy);
            },
            
            // _ => (),
        }
    }
}
