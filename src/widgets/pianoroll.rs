use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Color},
    text::Line,
    widgets::{StatefulWidget, Block, Borders, BorderType}
};

use crate::window::Window;
use crate::input::LocalCommand;

struct Note {
    freq: u32,
    start: u32,
    length: u32,
}

struct Pattern {
    notes: Vec<Note>
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
struct Pos2 {
    x: u16, y: u16,
}

#[derive(Default)]
pub struct PianoRoll;
impl StatefulWidget for PianoRoll {
    type State = PianoRollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for dy in 0..area.height {
            for dx in 0..area.width {
                let x = area.x + dx;
                let y = area.y + dy;

                if let Some(cell) = buf.cell_mut((x, y)) {
                    if state.selected == (Pos2 {x, y}) {
                        cell.set_style(Style::default().fg(Color::White));
                        cell.set_char('█');
                    } else {
                        cell.set_style(Style::default().fg(Color::DarkGray));
                        cell.set_char('·');
                    }
                }
            }
        }
    }
}

pub struct PianoRollState {
    id: usize,
    selected: Pos2, 
    note_size: u8,
    notes: Vec<Note>,
    zoom: u8,
}

impl PianoRollState {
    pub fn new(id: usize) -> Self {
        Self {
            id: id,
            selected: Pos2 { x: 2, y: 2 },
            note_size: 4,
            notes: Vec::new(), 
            zoom: 1,
        }
    }
}

impl Window for PianoRollState {
    fn id(&self) -> usize {
        self.id
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let piano_block = Block::bordered()
            .title(Line::from(" Piano Roll ").centered())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        frame.render_widget(piano_block.clone(), area);

        frame.render_stateful_widget(
            PianoRoll::default(),
            piano_block.inner(area),
            self,
        );
    }

    fn handle_input(&mut self, cmd: LocalCommand) {
        
    }
}

