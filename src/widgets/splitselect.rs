use crate::window::Window;
use crate::input::LocalCommand;
use crate::widgets::theme::UIStyle;
use crate::widgets::buttonlist::{ButtonList, ButtonListState, Button};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::Line,
    layout::Direction,
    widgets::{StatefulWidget, Clear},
};

#[derive(Default)]
pub struct SplitSelect<'a> {
    list_state: ButtonListState<'a>,
    direction: Direction,
}

impl SplitSelect<'_> {
    pub fn new(direction: Direction) -> Self {
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
            direction,
        }
    }
}

impl Window for SplitSelect<'_> {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = UIStyle::window_border("New Window", focused);

        let list_area = UIStyle::centered_rect(50, 50, area);
        frame.render_widget(Clear, block.inner(list_area));

        let list = ButtonList::new()
            .block(block)
            .style(Style::default().fg(Color::White));

        list.render(list_area, frame.buffer_mut(), &mut self.list_state);
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
