use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::StatefulWidget
};

use crate::{input::EditorCommand, log, theme::{ResolvedTheme, ThemeKey}};
use crate::windowpanes::window::Window;
use crate::input::LocalCommand;

struct Note {
    freq: u32,
    start: u32,
    length: u32,
}

struct Pattern {
    notes: Vec<Note>
}

pub struct PianoRoll {
    cursor_x: u16,
    cursor_y: u16,
    note_size: u8,
    notes: Vec<Note>,
    zoom: u8,
}

impl PianoRoll {
    pub fn new() -> Self {
        Self {
            cursor_x: 10,
            cursor_y: 10,
            note_size: 4,
            notes: Vec::new(), 
            zoom: 1,
        }
    }
}

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

impl Window for PianoRoll {
    fn title(&self) -> &str { " Piano Roll " }

    fn render(&mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        theme: &ResolvedTheme
    ) {
        frame.render_stateful_widget(
            PianoRollWidget::new(theme),
            area,
            self,
        );
    }

    fn handle_input(&mut self, cmd: LocalCommand) -> Option<EditorCommand> {
        None
    }
}

enum NoteDisplayKind {
    All, None, WhiteOnly
}

pub struct PianoRollWidget {
    white_style: (Style, Style),
    black_style: (Style, Style),
    white_names: bool,
    black_names: bool,
}

impl PianoRollWidget {
    pub fn new(theme: &ResolvedTheme) -> Self {
        Self {
            white_style: (theme.get(ThemeKey::PianoRollWhiteKey), 
                theme.get(ThemeKey::PianoRollWhiteKeyPressed)),
            black_style: (theme.get(ThemeKey::PianoRollBlackKey),
                theme.get(ThemeKey::PianoRollBlackKeyPressed)),
            white_names: true,
            black_names: true,
        }
    }

    pub fn white_style(mut self, style: (Style, Style)) -> Self {
        self.white_style = style;
        self
    }

    pub fn black_style(mut self, style: (Style, Style)) -> Self {
        self.black_style = style;
        self
    }

    pub fn render_note_names(mut self, white: bool, black: bool) -> Self {
        self.white_names = white;
        self.black_names = black;
        self
    }

    pub fn render_white_note_names(mut self, render: bool) -> Self {
        self.white_names = render;
        self
    }

    pub fn render_black_note_names(mut self, render: bool) -> Self {
        self.black_names = render;
        self
    }

    fn render_piano_keys(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for row in 0..area.height {
            let midi_note = 20 + row;
            let note = (midi_note % 12) as usize;
            let octave = (midi_note as i32 / 12) - 1;
            let note_name = NOTE_NAMES[note];
            let is_black = note_name.len() == 2;
            let pressed = vec![46, 48, 51].contains(&midi_note);

            let y = area.y + row;

            let label = format!("{}{}", note_name, octave);
            let offset_x = area.width - label.len() as u16 - (is_black as u16);
            let key_end = area.x + offset_x + label.len() as u16;

            let base_style = if is_black { self.black_style } else { self.white_style };
            let style = if pressed { base_style.1 } else { base_style.0 };

            // Background fill
            for x in area.x..(area.x + area.width) {
                buf[(x, y)].set_style(
                    if x < key_end { style }
                    else { self.white_style.0 }
                );
            }

            if !(if is_black { self.black_names } else { self.white_names }) {
                continue;
            }

            for (i, ch) in label.chars().enumerate() {
                let x = area.x + offset_x + i as u16;

                if x < area.x + area.width {
                    buf[(x, y)].set_char(ch);
                }
            }
        }
    }
}

impl StatefulWidget for PianoRollWidget {
    type State = PianoRoll;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // for dy in 0..area.height {
        //     for dx in 0..area.width {
        //         let x = area.x + dx;
        //         let y = area.y + dy;
        //
        //         // 
        //         if let Some(cell) = buf.cell_mut((x, y)) {
        //             if dx == 3 {
        //                 cell.set_style(Style::default().fg(Color::DarkGray));
        //                 cell.set_char('┆');// ┋┊
        //             } else if state.cursor_x == x && state.cursor_y == y {
        //                 cell.set_style(Style::default().fg(Color::Green));
        //                 cell.set_char('');
        //             } else {
        //                 cell.set_style(Style::default().fg(Color::DarkGray));
        //                 cell.set_char('·');
        //             }
        //         }
        //     }
        // }
        
        let keys_area = Rect {
            width: 7,
            ..area
        };

        self.render_piano_keys(keys_area, buf, state);
    }
}

