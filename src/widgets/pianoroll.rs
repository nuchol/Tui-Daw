use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Color},
    widgets::StatefulWidget,
};

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

enum Direction {
    Horizontal,
    Vertical,
}

pub struct PianoRollState {
    selected: Pos2, 
    note_size: u8,
    notes: Vec<Note>,
    zoom: u8,
}

#[derive(Default)]
pub struct PianoRoll;

impl PianoRollState {
    pub fn new() -> Self {
        return Self {
            selected: Pos2 { x: 2, y: 2 },
            note_size: 4,
            notes: Vec::new(), 
            zoom: 1,
        };
    }
}

impl PianoRollState {
    pub fn move_cursor(self: &mut Self) {
        self.selected.x += 1;
    }
}

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
