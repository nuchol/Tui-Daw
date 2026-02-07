use crate::window::Window;
use crate::input::LocalCommand;
use crate::widgets::theme::UIStyle;
use crate::widgets::buttonlist::{ButtonList, ButtonListState, Button};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::Line,
    widgets::{Block, Borders, StatefulWidget},
};

#[derive(Default)]
pub struct SplitSelect;
impl Window for SplitSelect {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = UIStyle::window_border("Piano Roll", focused);
        frame.render_widget(&block, area);

        let mut buttons = Vec::new();
        for i in 0..5 {
            buttons.push(Button {
                label: Line::from(format!("Button {}", i)),
                height: 1,
                style: Style::default(),
            })
        }

        let list = ButtonList::new(buttons)
            .block(Block::new().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        let mut list_state = ButtonListState::default();
        list_state.hovered(Some(1));

        list.render(block.inner(area), frame.buffer_mut(), &mut list_state);
    }

    fn handle_input(&mut self, cmd: LocalCommand) {
        match cmd {
            LocalCommand::MoveLocalCursor { dx, dy } => {

            },
            
            _ => (),
        }
    }
}
