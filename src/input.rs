use std::collections::HashMap;
use std::fmt;
use ratatui::crossterm::event::KeyCode;

use crate::AppState;

pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
            Mode::Command => write!(f, "Command"),
        }
    }
}

pub enum Motion {
    None,

    Left,
    Right,
    Up,
    Down,

    Beat,
    Bar,

    Start,
    End,
}

pub enum Operator {
    Delete,
    Yank,
    Paste,
    Undo,
    Redo,
    Mute,
    Solo,
}

pub enum InputAction {
    Move {
        count: usize,
        motion: Motion,
    },
    
    Operation {
        count: usize,
        operator: Operator,
        motion: Motion,
    },

    Command(String),
}

pub enum EditorCommand {
    Delete { count: usize, motion: Motion },
    Yank { count: usize, motion: Motion },
    Paste { count: usize, motion: Motion },
    Undo { count: usize, motion: Motion },
    Redo { count: usize, motion: Motion },
    Mute { count: usize, motion: Motion },
    Solo { count: usize, motion: Motion },
    Bpm { bpm: u32 },
    Quit,
}

pub enum LocalCommand {
    MoveLocalCursor { dx: i32, dy: i32 },
}

pub enum ResolvedCommand {
    Editor(EditorCommand),
    Local(LocalCommand)
}

pub struct Input;
impl Input {
    pub fn handle_keypress(
        state: &mut AppState,
        key: KeyCode
    ) -> Option<ResolvedCommand> {
        if key == KeyCode::Esc {
            state.input_state.clear();
            state.mode = Mode::Normal;
            return None;
        }

        let action = match state.mode {
            Mode::Normal => handle_normal_mode(state, key),
            Mode::Insert => handle_insert_mode(state, key),
            Mode::Command => handle_command_mode(state, key),
        };

        resolve_action(state, action)
    }
}

pub struct InputState {
    pub count: usize,
    pub operator: Option<Operator>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            count: 0,
            operator: None,
        }
    }

    pub fn clear(self: &mut Self) {
        self.count = 0;
        self.operator = None;
    }

    pub fn display(&self) -> String {
        let mut s = String::new();

        if self.count > 0 {
            s.push_str(&self.count.to_string());
        }

        if let Some(op) = &self.operator {
            s.push_str(match op {
                Operator::Delete => "d",
                Operator::Yank => "y",
                Operator::Paste => "p",
                Operator::Mute => "m",
                Operator::Solo => "s",
                Operator::Undo => "u",

                _ => "?",
            });
        }

        s
    }
}

#[derive(Default)]
pub struct CommandState {
    pub buffer: String,
    pub cursor: usize,
}

impl CommandState {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

fn handle_normal_mode(
    state: &mut AppState,
    key: KeyCode
) -> Option<InputAction> {
    match key {
        KeyCode::Char('i') => {
            state.input_state.clear();
            state.mode = Mode::Insert;
            None
        },

        KeyCode::Char(':') | KeyCode::Char(';') => {
            state.input_state.clear();
            state.command_state.clear();
            state.mode = Mode::Command;
            None
        },

        KeyCode::Char(c) if c.is_ascii_digit() => {
            let d = c.to_digit(10).unwrap() as usize;
            state.input_state.count = state.input_state.count * 10 + d;
            None
        }

        KeyCode::Char('d') => {
            state.input_state.operator = Some(Operator::Delete);
            None
        }

        KeyCode::Char('y') => {
            state.input_state.operator = Some(Operator::Yank);
            None
        }

        KeyCode::Char('u') => {
            state.input_state.operator = Some(Operator::Undo);
            emit_action(&mut state.input_state, Motion::None)
        }

        KeyCode::Char('h') => emit_action(&mut state.input_state, Motion::Left),
        KeyCode::Char('j') => emit_action(&mut state.input_state, Motion::Down),
        KeyCode::Char('k') => emit_action(&mut state.input_state, Motion::Up),
        KeyCode::Char('l') => emit_action(&mut state.input_state, Motion::Right),

        _ => None,
    }
}

fn emit_action(
    state: &mut InputState,
    motion: Motion
) -> Option<InputAction> {
    let count = if state.count == 0 { 1 } else { state.count };

    let action = match state.operator.take() {
        Some(op) => InputAction::Operation {
            count,
            operator: op,
            motion,
        },

        None => InputAction::Move {count, motion},
    };

    state.clear();
    Some(action)
}

fn handle_insert_mode(
    state: &mut AppState,
    key: KeyCode
) -> Option<InputAction> {
    return None;
}

fn resolve_action(
    state: &mut AppState,
    action: Option<InputAction>
) -> Option<ResolvedCommand> {
    match action {
        Some(InputAction::Move { count, motion }) => {
            resolve_move(state, count, motion)
        }

        Some(InputAction::Operation {
            count,
            operator,
            motion,
        }) => resolve_operation(state, count, operator, motion),

        Some(InputAction::Command(cmd)) => resolve_command(state, cmd),

        None => None
    }
}

fn resolve_move(
    state: &AppState,
    count: usize,
    motion: Motion,
) -> Option<ResolvedCommand> {
    None
}

fn resolve_operation(
    state: &AppState,
    count: usize,
    operator: Operator,
    motion: Motion,
) -> Option<ResolvedCommand> {
    match operator {
        Operator::Delete => Some(ResolvedCommand::Editor(
            EditorCommand::Delete { count, motion }
        )),

        Operator::Yank => Some(ResolvedCommand::Editor(
            EditorCommand::Yank { count, motion }
        )),

        Operator::Mute => Some(ResolvedCommand::Editor(
            EditorCommand::Mute { count, motion }
        )),

        _ => None,
    }
}

fn resolve_command(
    state: &AppState,
    command: String,
) -> Option<ResolvedCommand> {
    match command.as_str() {
        "q" | "quit" => Some(ResolvedCommand::Editor(EditorCommand::Quit)),
        _ => None,
    }
}

fn handle_command_mode(
    state: &mut AppState,
    key: KeyCode
) -> Option<InputAction> {
    let command = &mut state.command_state;

    match key {
        KeyCode::Enter => {
            let cmd = command.buffer.clone();
            command.clear();
            state.mode = Mode::Normal;

            Some(InputAction::Command(cmd))
        }

        KeyCode::Char(c) => {
            command.buffer.insert(command.cursor, c);
            command.cursor += 1;
            None
        }

        KeyCode::Delete => {
            if (0..command.buffer.len()).contains(&command.cursor) {
                command.buffer.remove(command.cursor);
            }

            None
        }

        KeyCode::Backspace => {
            if command.buffer.len() > 0 {
                if command.cursor > 0 {
                    command.cursor -= 1;
                    command.buffer.remove(command.cursor);
                }
            } else {
                command.clear();
                state.mode = Mode::Normal;
            }

            None
        }

        KeyCode::Left => {
            command.cursor = command.cursor.saturating_sub(1);
            None
        }

        KeyCode::Right => {
            command.cursor = (command.cursor + 1).min(command.buffer.len());
            None
        }


        _ => None
    }
}
