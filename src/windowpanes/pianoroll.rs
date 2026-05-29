use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::StatefulWidget
};

use crate::{input::{EditorCommand, Motion}, theme::{ResolvedTheme, ThemeKey}};
use crate::windowpanes::window::Window;
use crate::input::LocalCommand;

const MIDI_MAX: u8 = 127;
// Pulses Per Quarter Note (Ticks Per Beat)
const PPQ: u32 = 960;
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

struct Note {
    pitch: u8,
    start_tick: u32,
    // Duration in ticks
    duration: u32,
}

struct Pattern {
    notes: Vec<Note>,
    length: u32, // in ticks
}

pub struct PianoRoll {
    cursor: (u32, u8), // (tick, note)
    note_size: u32, // in ticks
    notes: Vec<Note>, // Ordered by tick
    zoom: u8,
    scroll: (u32, u8), // (ticks, notes)
    cells_per_beat: u16,
    beats_per_bar: u16,
    ticks_per_beat: u32,
}

impl PianoRoll {
    pub fn new() -> Self {
        Self {
            cursor: (0, 10),
            note_size: 4,
            notes: Self::test_notes(),
            zoom: 1,
            scroll: (0, 50),
            cells_per_beat: 4,
            beats_per_bar: 4,
            ticks_per_beat: PPQ,
        }
    }

    fn test_notes() -> Vec<Note> {
        vec![
            Note {pitch: 67, start_tick: PPQ * 0, duration: PPQ * 4},
            Note {pitch: 68, start_tick: PPQ * 1, duration: (PPQ as f32 * 0.5) as u32},
            Note {pitch: 60, start_tick: PPQ * 2, duration: PPQ * 2},
        ]
    }

    fn handle_motion(&mut self, count: u32, motion: Motion) {
        let (mut x, mut y) = self.cursor;

        // ticks/bar = ticks/beat * beats/bar;
        let ticks_per_bar = self.beats_per_bar as u32 * self.ticks_per_beat;
        match motion {
            // Motion::Bar => x += ticks_per_bar,
            Motion::Bar => x = (x / ticks_per_bar + count) * ticks_per_bar,
            Motion::Beat => x = (x / self.ticks_per_beat + count) * self.ticks_per_beat,
            Motion::Subdivision => (),

            _ => return,
        };

        self.cursor = (x, y);
    }
}

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
        match cmd {
            LocalCommand::MoveLocalCursor { dx, dy } => {
                let ticks_per_cell = (PPQ / self.cells_per_beat as u32) as i32;
                self.cursor.0 = self.cursor.0.saturating_add_signed(dx * ticks_per_cell);
                self.cursor.1 = (self.cursor.1 as i32 - dy)
                                .clamp(0, i8::MAX as i32) as u8;

                None
            },

            LocalCommand::MoveByMotion { count, motion } => {
                self.handle_motion(count, motion);
                None
            },

            _ => None,
        }
    }
}

pub struct PianoRollWidget {
    cursor_style: Style,
    white_style: (Style, Style),
    black_style: (Style, Style),
    bar_div_style: Style,
    beat_div_style: Style,
    sub_div_style: Style,
    note_style: Style,
    note_accent_style: Style,
    white_names: bool,
    black_names: bool,
}

impl PianoRollWidget {
    pub fn new(theme: &ResolvedTheme) -> Self {
        Self {
            cursor_style: theme.get(ThemeKey::Cursor),
            white_style: (theme.get(ThemeKey::PianoRollWhiteKey), 
                theme.get(ThemeKey::PianoRollWhiteKeyPressed)),
            black_style: (theme.get(ThemeKey::PianoRollBlackKey),
                theme.get(ThemeKey::PianoRollBlackKeyPressed)),
            bar_div_style: theme.get(ThemeKey::PainoRollBeatSeparator),
            beat_div_style: theme.get(ThemeKey::PainoRollBeatSeparator),
            sub_div_style: theme.get(ThemeKey::PainoRollSubDivSeparator),
            note_style: theme.get(ThemeKey::PianoRollNote),
            note_accent_style: theme.get(ThemeKey::PianoRollNoteAccent),
            white_names: true,
            black_names: false,
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

    fn ticks_to_cells(tick: u32, state: &PianoRoll) -> u16 {
        let ticks_per_cell = PPQ / state.cells_per_beat as u32;
        (tick / ticks_per_cell) as u16
    }

    fn render_piano_keys(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for row in 0..area.height {
            // TODO: Remove hard coding
            let midi_note = MIDI_MAX - (state.scroll.1 + row as u8);
            let pressed = vec![46, 48, 51].contains(&midi_note);

            let note = (midi_note % 12) as usize;
            let octave = (midi_note as i32 / 12) - 1;
            let note_name = NOTE_NAMES[note];
            let is_black = note_name.len() == 2;

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

    fn render_vertical_lines(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for col in 0..area.width {
            // Absolute column index from the beginning of the piece.
            let abs_col = col + Self::ticks_to_cells(state.scroll.0, state);

            let is_bar = abs_col % (state.beats_per_bar * state.cells_per_beat) == 0;
            let is_beat = abs_col % state.cells_per_beat == 0;

            let (line_char, style) = if is_bar {
                ('▎', self.bar_div_style)
            } else if is_beat {
                ('▏', self.beat_div_style)
            } else {
                ('▏', self.sub_div_style)
            };

            let x = area.x + col;
            for row in 0..area.height {
                let y = area.y + row;
                let cell = &mut buf[(x, y)];

                // Only stamp the line glyph on empty background cells so that note
                // blocks drawn later can freely overwrite it.
                if cell.symbol() == " " {
                    cell.set_style(style);
                    cell.set_char(line_char);
                }
            }
        }
    }

    fn render_bar_numbers(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for col in 0..area.width {
            // Absolute column index from the beginning of the piece.
            let abs_col = col + Self::ticks_to_cells(state.scroll.0, state);

            let is_bar = abs_col % (state.beats_per_bar * state.cells_per_beat) == 0;
            let is_num = (abs_col.saturating_sub(1)) % (state.beats_per_bar * state.cells_per_beat) == 0;

            let (label, style) = if is_bar {
                ("▎".into(), self.bar_div_style)
            } else if is_num {
                let bar_num = abs_col / (state.beats_per_bar * state.cells_per_beat) + 1;
                (bar_num.to_string(), self.bar_div_style)
            } else {
                continue;
            };

            for (i, ch) in label.chars().enumerate() {
                let x = area.x + col + i as u16;

                if x < area.x + area.width {
                    buf[(x, area.y)]
                        .set_char(ch)
                        .set_style(style);
                }
            }
        }
    }

    fn render_notes(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for note in &state.notes {
            let row = MIDI_MAX as i32 - (state.scroll.1 as i32 + note.pitch as i32);

            // note's pitch is not visible
            if row < 0 || row > area.height as i32 {
                continue;
            }

            let y = area.y + row as u16;

            let ticks_per_cell = PPQ as u16 / state.cells_per_beat;
            let start_cell = Self::ticks_to_cells(note.start_tick - state.scroll.0, state);
            let length = (note.duration as u16 / ticks_per_cell).max(1);

            // note is not visible
            if start_cell + length <= 0 || start_cell >= area.width {
                continue;
            }

            let note_name = NOTE_NAMES[(note.pitch % 12) as usize];
            let octave = (note.pitch as i32 / 12) - 1;
            let label_len = note_name.len() + octave.to_string().len() + 1;

            let label = if label_len > length as usize {
                "▌".into()
            } else {
                format!("▌{}{}", note_name, octave)
            };

            let note_str = format!("{label:<length$}", length = length as usize);

            let mut style = self.note_accent_style;
            for (i, ch) in note_str.chars().enumerate() {
                if i != 0 { style = self.note_style; }
                let x = area.x + start_cell + i as u16;

                if x < area.x + area.width {
                    buf[(x, y)]
                        .set_char(ch)
                        .set_style(style);
                }
            }
        }
    }
}

impl StatefulWidget for PianoRollWidget {
    type State = PianoRoll;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let key_width = 7;
        let bar_number_height = 1;

        let grid_area = Rect {
            y: area.y + bar_number_height,
            x: area.x + key_width,
            width: area.width - key_width,
            height: area.height - bar_number_height,
        };

        let barnum_area = Rect {
            y: area.y,
            height: bar_number_height,
            ..grid_area
        };

        let keys_area = Rect {
            x: area.x,
            width: key_width,
            ..grid_area
        };

        self.render_bar_numbers(barnum_area, buf, state);
        self.render_piano_keys(keys_area, buf, state);
        self.render_vertical_lines(grid_area, buf, state);
        self.render_notes(grid_area, buf, state);

        let cursor_x = Self::ticks_to_cells(state.cursor.0 - state.scroll.0, state);
        buf[(cursor_x + grid_area.x, state.cursor.1 as u16 + grid_area.y)]
            .set_style(self.cursor_style)
            .set_char(' ');
    }
}


