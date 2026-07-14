use std::collections::HashMap;
use std::fmt;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Direction;

use crate::{AppState, log};
use crate::windowpanes::{
    window::{Window, WindowPaneType},
    windowselect::WindowSelect,
};

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

pub enum Move {
    None,

    HStep,
    VStep,

    Beat,
    Bar,
    Subdivision,
    Note,

    Start,
    End,
}

pub struct Motion {
    pub move_type: Move,
    pub dir: MoveDir,
}

impl Motion {
    fn new(move_type: Move, dir: MoveDir) -> Self {
        Self { move_type, dir, }
    }
}

pub enum Operator {
    Delete,
    Yank,
    Undo,
    Confirm,
}

#[derive(PartialEq, Eq)]
pub enum MoveDir {
    Forward = 1, 
    Backward = -1,
}

pub enum InputAction {
    Move {
        count: u32,
        motion: Option<Motion>,
    },
    
    Operation {
        count: u32,
        operator: Operator,
        motion: Option<Motion>,
    },

    Command(String),
}

pub enum EditorCommand {
    Yank   { count: u32, motion: Motion },
    Paste  { count: u32, motion: Motion },
    Undo   { count: u32, motion: Motion },
    Redo   { count: u32, motion: Motion },
    Mute   { count: u32, motion: Motion },
    Solo   { count: u32, motion: Motion },
    Delete { count: u32, motion: Motion },
    Bpm { bpm: u32 },
    OpenWindow { display: WindowPaneType, window: Box<dyn Window> },
    Theme { theme: String },
    Quit,
}

pub enum LocalCommand {
    MoveLocalCursor { dx: i32, dy: i32 },
    MoveByMotion { count: u32, motion: Motion },
    Confirm,
}

pub enum ResolvedCommand {
    Editor(EditorCommand),
    Local(LocalCommand)
}

pub struct VimInput;
impl VimInput {
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

        resolve_action(action)
    }
}

pub struct InputState {
    pub count: u32,
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

        KeyCode::Char('.') => {
            // TODO: Repeat last command
            None
        }

        KeyCode::Char(c) if c.is_ascii_digit() => {
            let d = c.to_digit(10).unwrap();
            state.input_state.count = state.input_state.count
                .saturating_mul(10).saturating_add(d);

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
            emit_action(&mut state.input_state, None)
        }

        KeyCode::Enter => {
            state.input_state.operator = Some(Operator::Confirm);
            emit_action(&mut state.input_state, None)
        }

        KeyCode::Char('h') => emit_action(&mut state.input_state, Some(Motion::new(Move::HStep, MoveDir::Backward))),
        KeyCode::Char('j') => emit_action(&mut state.input_state, Some(Motion::new(Move::VStep, MoveDir::Backward))),
        KeyCode::Char('k') => emit_action(&mut state.input_state, Some(Motion::new(Move::VStep, MoveDir::Forward))),
        KeyCode::Char('l') => emit_action(&mut state.input_state, Some(Motion::new(Move::HStep, MoveDir::Forward))),

        KeyCode::Char('W') => emit_action(&mut state.input_state, Some(Motion::new(Move::Bar, MoveDir::Forward))),
        KeyCode::Char('B') => emit_action(&mut state.input_state, Some(Motion::new(Move::Bar, MoveDir::Backward))),
        KeyCode::Char('w') => emit_action(&mut state.input_state, Some(Motion::new(Move::Note, MoveDir::Forward))),
        KeyCode::Char('b') => emit_action(&mut state.input_state, Some(Motion::new(Move::Note, MoveDir::Backward))),

        KeyCode::Char('s') => emit_action(&mut state.input_state, Some(Motion::new(Move::Subdivision, MoveDir::Forward))),
        KeyCode::Char('S') => emit_action(&mut state.input_state, Some(Motion::new(Move::Subdivision, MoveDir::Backward))),

        _ => None,
    }
}

fn emit_action(
    state: &mut InputState,
    motion: Option<Motion>,
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
    action: Option<InputAction>
) -> Option<ResolvedCommand> {
    match action {
        Some(InputAction::Move { count, motion }) => {
            resolve_move(count, motion.unwrap())
        }

        Some(InputAction::Operation {
            count,
            operator,
            motion,
        }) => resolve_operation(count, operator, motion),

        Some(InputAction::Command(cmd)) => resolve_command(cmd),

        None => None
    }
}

fn resolve_move(
    count: u32,
    motion: Motion,
) -> Option<ResolvedCommand> {
    let cmd = match motion.move_type {
        Move::VStep => LocalCommand::MoveLocalCursor { dx: 0, dy: count as i32 * motion.dir as i32 },
        Move::HStep => LocalCommand::MoveLocalCursor { dx: count as i32 * motion.dir as i32, dy: 0 },

        // Any motion that must be handled by window.
        _ => LocalCommand::MoveByMotion { count, motion },
    };

    Some(ResolvedCommand::Local(cmd))
}

fn resolve_operation(
    count: u32,
    operator: Operator,
    motion: Option<Motion>,
) -> Option<ResolvedCommand> {
    match operator {
        // Operator::Delete => Some(ResolvedCommand::Editor(
        //     EditorCommand::Delete { count, motion }
        // )),
        //
        // Operator::Yank => Some(ResolvedCommand::Editor(
        //     EditorCommand::Yank { count, motion }
        // )),
        //
        // Operator::Mute => Some(ResolvedCommand::Editor(
        //     EditorCommand::Mute { count, motion }
        // )),

        Operator::Confirm => Some(
            ResolvedCommand::Local(LocalCommand::Confirm)
        ),

        _ => None,
    }
}

fn resolve_command(
    command: String,
) -> Option<ResolvedCommand> {
    let tokens: Vec<&str> = command.split(' ').collect();
    match tokens[0] {
        "q" | "quit" => Some(ResolvedCommand::Editor(EditorCommand::Quit)),

        // We want to split accross the opposite direction since
        // splitting adds another window on the 'direction' axis.
        "vsplit" => Some(ResolvedCommand::Editor(
            EditorCommand::OpenWindow { 
                display: WindowPaneType::Popup,
                window: Box::new(WindowSelect::new(
                    WindowPaneType::Direction { direction: Direction::Horizontal }
                ))
            }
        )),

        "hsplit" => Some(ResolvedCommand::Editor(
            EditorCommand::OpenWindow { 
                display: WindowPaneType::Popup,
                window: Box::new(WindowSelect::new(
                    WindowPaneType::Direction { direction: Direction::Vertical }
                ))
            }
        )),

        "theme" => Some(ResolvedCommand::Editor(
            EditorCommand::Theme { theme: tokens[1].to_string() }
        )),

        _ => {
            log::log(
                format!("Not a recognised command: {}", command.as_str()),
                log::LogLevel::ERROR);
            None
        },
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
