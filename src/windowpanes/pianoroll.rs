use ratatui::{
    Frame, buffer::Buffer,
    crossterm::{event::KeyCode},
    layout::Rect,
    style::Style,
    widgets::StatefulWidget
};

use crate::{
    input::{Dir, EditorCommand, LocalCommand, UniversalCommand},
    theme::{ResolvedTheme, ThemeKey},
    windowpanes::window::Window,
    log::log,
};

const MIDI_MAX: u8 = 127;
// Pulses Per Quarter Note (Ticks Per Beat)
const PPQ: u32 = 960;
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F",
    "F#", "G", "G#", "A", "A#", "B",
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

enum PianoRollMotion {
    Beat(Dir),
    Bar(Dir),
    End(Dir),
    Subdivision(Dir),
}

pub struct PianoRoll {
    cursor: (u32, u8), // (tick, note)
    notes: Vec<Note>, // Ordered by tick
    note_size: u32, // in ticks
    zoom: u8,
    scroll: (u32, u8), // (ticks, notes)
    viewport: (u16, u16),
    cells_per_beat: u16,
    beats_per_bar: u16,
    ticks_per_beat: u32,
}

impl PianoRoll {
    pub fn new() -> Self {
        Self {
            cursor: (0, 67),
            note_size: 4,
            notes: Self::test_notes(),
            zoom: 1,
            scroll: (0, 45),
            viewport: (0, 0),
            cells_per_beat: 4,
            beats_per_bar: 4,
            ticks_per_beat: PPQ,
        }
    }

    fn cursor_pitch(&self) -> u8 {
        MIDI_MAX - (self.cursor.1 + self.scroll.1)
    }

    fn ticks_per_cell(&self) -> u32 {
        (self.ticks_per_beat / self.cells_per_beat as u32).max(1)
    }

    fn cells_per_bar(&self) -> u16 {
        self.cells_per_beat * self.beats_per_bar
    }

    fn test_notes() -> Vec<Note> {
        let mut notes = vec![
            Note {pitch: 67, start_tick: PPQ * 0, duration: PPQ * 4},
            Note {pitch: 68, start_tick: PPQ * 2, duration: PPQ * 2},
            Note {pitch: 69, start_tick: PPQ * 7, duration: PPQ * 6},
            Note {pitch: 70, start_tick: PPQ * 9, duration: PPQ * 1},
            Note {pitch: 71, start_tick: PPQ * 11, duration: PPQ * 5},
            Note {pitch: 68, start_tick: PPQ * 1, duration: PPQ / 2},
            Note {pitch: 60, start_tick: PPQ * 2, duration: PPQ * 2},
        ];
        notes.sort_by(|a, b| a.start_tick.cmp(&b.start_tick));
        notes
    }

    fn sync_scroll(&mut self, padding: (u32, u8)) {
        let (width, height) = self.viewport;
        if width == 0 || height == 0 { return; }

        let tpc = self.ticks_per_cell();

        // Horizontal Scrolling
        let pad_x = padding.0.min((width - 1) as u32 / 2);
        let cursor_x = self.cursor.0 / tpc;
        let mut first_x = self.scroll.0 / tpc;
        
        if cursor_x < first_x + pad_x {
            first_x = cursor_x.saturating_sub(pad_x);
        } else if cursor_x + pad_x >= first_x + width as u32 {
            first_x = (cursor_x + pad_x + 1).saturating_sub(width as u32);
        }

        self.scroll.0 = first_x * tpc;

        // Vertical Scrolling
        let rows = MIDI_MAX as u16 + 1;
        let pad_y = (padding.1 as u16).min((width - 1) / 2);
        let cursor_y = self.cursor.1 as u16;
        let mut first_y = self.scroll.1 as u16;

        if cursor_y < first_y + pad_y {
            first_y = cursor_y.saturating_sub(pad_y);
        } else if cursor_y + pad_y >= first_y + height{
            first_y = (cursor_y + pad_y + 1).saturating_sub(height)
        }

        self.scroll.1 = first_y.min(rows.saturating_sub(height)) as u8;
    }

    fn handle_motion(&mut self, count: u32, motion: PianoRollMotion) -> Option<EditorCommand> {
        let (mut x, mut y) = self.cursor;

        let ticks_per_bar = self.beats_per_bar as u32 * self.ticks_per_beat;
        match motion {
            PianoRollMotion::Bar(dir) => x = (x / ticks_per_bar)
                .saturating_add_signed(count as i32 * dir as i32)
                * ticks_per_bar,


            PianoRollMotion::Beat(dir) => x = (x / self.ticks_per_beat)
                .saturating_add_signed(count as i32 * dir as i32)
                * self.ticks_per_beat,

            PianoRollMotion::Subdivision(dir) => (),

            // TODO: needs to find next note end not next note start.
            PianoRollMotion::End(dir) => x = 
                self.get_next_note(self.cursor_pitch(), dir)
                    .map_or(self.cursor.0, |n| n.start_tick.saturating_add(n.duration)),
        };

        self.cursor = (x, y);
        None
    }

    fn get_next_note(&self, pitch: u8, dir: Dir) -> Option<&Note> {
        match dir {
            Dir::Forward => {
                let split = self.notes.partition_point(|n| n.start_tick <= self.cursor.0);
                self.notes[split..].iter().find(|n| n.pitch == pitch)
            }
            Dir::Backward => {
                let split = self.notes.partition_point(|n| n.start_tick < self.cursor.0);
                self.notes[..split].iter().rev().find(|n| n.pitch == pitch)
            }
        }
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
    
    fn handle_universal(&mut self, cmd: UniversalCommand) {
        let ticks_per_cell = (PPQ / self.cells_per_beat as u32) as i32;
        match cmd {
            UniversalCommand::Horizontal { count, dir } => {
                let dx = count as i32 * dir as i32;
                self.cursor.0 = self.cursor.0.saturating_add_signed(dx * ticks_per_cell);
            },

            UniversalCommand::Vertical { count, dir } => {
                let dy = count as i32 * dir as i32;
                self.cursor.1 = (self.cursor.1 as i32 - dy)
                                .clamp(0, MIDI_MAX as i32) as u8;
            },

            // Go to next note (n/N)
            UniversalCommand::Next { count, dir } => self.cursor.0 = 
                self.get_next_note(self.cursor_pitch(), dir)
                    .map_or(self.cursor.0, |n| n.start_tick),

            _ => ()
        }
    }

    fn handle_input(&mut self, cmd: LocalCommand) -> Option<EditorCommand> {
        match cmd {
            LocalCommand::Operator { .. } => None,
            LocalCommand::KeyPress { count, key } => match key {
                KeyCode::Char('w') => self.handle_motion(count, PianoRollMotion::Beat(Dir::Forward)),
                KeyCode::Char('b') => self.handle_motion(count, PianoRollMotion::Beat(Dir::Backward)),
                KeyCode::Char('W') => self.handle_motion(count, PianoRollMotion::Bar(Dir::Forward)),
                KeyCode::Char('B') => self.handle_motion(count, PianoRollMotion::Bar(Dir::Backward)),
                KeyCode::Char('e') => self.handle_motion(count, PianoRollMotion::End(Dir::Forward)),
                KeyCode::Char('E') => self.handle_motion(count, PianoRollMotion::End(Dir::Backward)),
                KeyCode::Char('s') => self.handle_motion(count, PianoRollMotion::Subdivision(Dir::Forward)),
                KeyCode::Char('S') => self.handle_motion(count, PianoRollMotion::Subdivision(Dir::Backward)),

                KeyCode::Enter => None,

                _ => None,
            },
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
        
        state.viewport = (grid_area.width, grid_area.height);
        state.sync_scroll((state.cells_per_bar() as u32, 5));

        let bar_num_area = Rect {
            y: area.y,
            height: bar_number_height,
            ..grid_area
        };

        let keys_area = Rect {
            x: area.x,
            width: key_width,
            ..grid_area
        };

        self.render_bar_numbers(bar_num_area, buf, state);
        self.render_piano_keys(keys_area, buf, state);
        self.render_vertical_lines(grid_area, buf, state);
        self.render_notes(grid_area, buf, state);

        let cursor_x = Self::ticks_to_cells(
            state.cursor.0.saturating_sub(state.scroll.0),
            state
        );
        let cursor_y = state.cursor.1.saturating_sub(state.scroll.1) as u16;

        buf[(cursor_x + grid_area.x, cursor_y + grid_area.y)]
            .set_style(self.cursor_style);
            // .set_char(' ');
    }
}


impl PianoRollWidget {
    pub fn new(theme: &ResolvedTheme) -> Self {
        Self {
            cursor_style: theme.get(ThemeKey::Cursor),
            white_style: (theme.get(ThemeKey::PianoRollWhiteKey), 
                theme.get(ThemeKey::PianoRollWhiteKeyPressed)),
            black_style: (theme.get(ThemeKey::PianoRollBlackKey),
                theme.get(ThemeKey::PianoRollBlackKeyPressed)),
            bar_div_style: theme.get(ThemeKey::PianoRollBeatSeparator),
            beat_div_style: theme.get(ThemeKey::PianoRollBeatSeparator),
            sub_div_style: theme.get(ThemeKey::PianoRollSubDivSeparator),
            note_style: theme.get(ThemeKey::PianoRollNote),
            note_accent_style: theme.get(ThemeKey::PianoRollNoteAccent),
            white_names: true,
            black_names: false,
        }
    }

    fn ticks_to_cells(tick: u32, state: &PianoRoll) -> u16 {
        let ticks_per_cell = PPQ / state.cells_per_beat as u32;
        (tick / ticks_per_cell) as u16
    }

    // TDOD: Fix scrolling
    fn render_piano_keys(&self, area: Rect, buf: &mut Buffer, state: &PianoRoll) {
        for row in 0..area.height {
            // TODO: Remove hard coding
            let midi_note = MIDI_MAX - (state.scroll.1 + row as u8);
            let pressed = vec![46, 48, 51].contains(&midi_note);

            let note = (midi_note % 12) as usize;
            let octave = (midi_note as i32 / 12) - 1;
            let note_name = NOTE_NAMES[note];
            let is_black = note_name.len() == 2;

            let label = format!("{}{}", note_name, octave);
            let offset_x = area.width - label.len() as u16 - (is_black as u16);
            let key_end = area.x + offset_x + label.len() as u16;

            let base_style = if is_black { self.black_style } else { self.white_style };
            let style = if pressed { base_style.1 } else { base_style.0 };

            // Background fill
            let y = area.y + row;
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

            let start_cell = (note.start_tick as i32 - state.scroll.0 as i32)
                / state.ticks_per_cell() as i32;
            let end_cell = ((note.start_tick + note.duration) as i32 - state.scroll.0 as i32)
                / state.ticks_per_cell() as i32;

            // note is not visible
            if row < 0 || row >= area.height as i32 
                || end_cell <= 0 || start_cell >= area.width as i32 {
                continue;
            }
            
            let note_name = NOTE_NAMES[(note.pitch % 12) as usize];
            let octave = (note.pitch as i32 / 12) - 1;
            let label_len = note_name.len() + octave.to_string().len() + 1;

            let length = (end_cell - start_cell).max(1) as usize;
            let label = if label_len > length {
                "▌".into()
            } else {
                format!("▌{}{}", note_name, octave)
            };

            let note_str = format!("{label:<length$}");
            let mut style = self.note_accent_style;

            let y = area.y + row as u16;
            for (i, ch) in note_str.chars().enumerate() {
                if i != 0 { style = self.note_style; }

                let x = start_cell + i as i32;
                if x < 0 { continue; }
                if x >= area.width as i32 { break; }

                buf[(area.x + x as u16, y)]
                    .set_char(ch)
                    .set_style(style);
            }
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
}
