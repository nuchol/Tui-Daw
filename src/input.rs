use std::collections::HashMap;
use std::fmt;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Direction;

use crate::log;
use crate::windowpanes::{
    window::{Window, WindowPaneType},
    windowselect::WindowSelect,
};

#[derive(Default)]
pub enum Mode {
    #[default] Normal,
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
    Horizontal,
    Vertical,
    Next,

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

#[derive(Clone, Copy)]
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
    Yank { count: u32, motion: Motion },
    Paste { count: u32, motion: Motion },
    Undo { count: u32, motion: Motion },
    Redo { count: u32, motion: Motion },
    Mute { count: u32, motion: Motion },
    Solo { count: u32, motion: Motion },
    Delete { count: u32, motion: Motion },
    Bpm { bpm: u32 },
    OpenWindow { display: WindowPaneType, window: Box<dyn Window> },
    Theme { theme: String },
    Quit,
}

pub enum UniversalCommand {
    GotoStart,
    GotoEnd,
    ScrollUp(u32),
    ScrollDown(u32),
}

pub enum LocalCommand {
    Move { count: u32, dir: MoveDir, key: KeyCode },
    Operator { count: u32, dir: MoveDir, key: KeyCode, op: Operator },
    Confirm,
}

pub enum ResolvedCommand {
    Editor(EditorCommand),
    Universal(UniversalCommand),
    Local(LocalCommand)
}

#[derive(Default)]
pub struct Input {
    pub mode: Mode,

    pub count: u32,
    pub operator: Option<Operator>,

    pub command_buffer: String,
    pub command_cursor: usize,
}

impl Input {
    pub fn clear_op(&mut self) {
        self.count = 0;
        self.operator = None;
    }

    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    pub fn display_op(&self) -> String {
        let mut s = String::new();

        if self.count > 0 {
            s.push_str(&self.count.to_string());
        }

        if let Some(op) = self.operator {
            s.push_str(match op {
                Operator::Delete => "d",
                Operator::Yank => "y",
                Operator::Undo => "u",

                _ => "?",
            });
        }

        s
    }

    pub fn handle_keypress(
        &mut self,
        key: KeyCode
    ) -> Option<ResolvedCommand> {
        if key == KeyCode::Esc {
            self.clear_op();
            self.clear_command();
            self.mode = Mode::Normal;
            return None;
        }

        let action = match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
        };

        Self::resolve_action(action)
    }

    fn handle_normal_mode(
        &mut self,
        key: KeyCode,
    ) -> Option<InputAction> {
        match key {
            KeyCode::Char('i') => {
                self.clear_op();
                self.mode = Mode::Insert;
                None
            },

            KeyCode::Char(':') | KeyCode::Char(';') => {
                self.clear_op();
                self.clear_command();
                self.mode = Mode::Command;
                None
            },

            KeyCode::Char('.') => {
                // TODO: Repeat last command
                None
            }

            KeyCode::Char(c) if c.is_ascii_digit() => {
                let d = c.to_digit(10).unwrap();
                self.count = self.count
                    .saturating_mul(10).saturating_add(d);

                None
            }

            KeyCode::Char('d') => {
                self.operator = Some(Operator::Delete);
                None
            }

            KeyCode::Char('y') => {
                self.operator = Some(Operator::Yank);
                None
            }

            KeyCode::Char('u') => {
                self.operator = Some(Operator::Undo);
                self.emit_action(None)
            }

            KeyCode::Enter => {
                self.operator = Some(Operator::Confirm);
                self.emit_action(None)
            }

            // Physical Motions (universal)
            KeyCode::Char('h') => self.emit_action(Some(Motion::new(Move::Horizontal, MoveDir::Backward))),
            KeyCode::Char('j') => self.emit_action(Some(Motion::new(Move::Vertical, MoveDir::Backward))),
            KeyCode::Char('k') => self.emit_action(Some(Motion::new(Move::Vertical, MoveDir::Forward))),
            KeyCode::Char('l') => self.emit_action(Some(Motion::new(Move::Horizontal, MoveDir::Forward))),
            KeyCode::Char('n') => self.emit_action(Some(Motion::new(Move::Next, MoveDir::Forward))),
            KeyCode::Char('N') => self.emit_action(Some(Motion::new(Move::Next, MoveDir::Backward))),

            // TODO: Semantic Motions (implemented by window)
            KeyCode::Char('w') => self.emit_action(Some(Motion::new(Move::Beat, MoveDir::Forward))),
            KeyCode::Char('b') => self.emit_action(Some(Motion::new(Move::Beat, MoveDir::Backward))),
            KeyCode::Char('W') => self.emit_action(Some(Motion::new(Move::Bar, MoveDir::Forward))),
            KeyCode::Char('B') => self.emit_action(Some(Motion::new(Move::Bar, MoveDir::Backward))),
            KeyCode::Char('s') => self.emit_action(Some(Motion::new(Move::Subdivision, MoveDir::Forward))),
            KeyCode::Char('S') => self.emit_action(Some(Motion::new(Move::Subdivision, MoveDir::Backward))),

            _ => None,
        }
    }

    fn emit_action(
        &mut self,
        motion: Option<Motion>,
    ) -> Option<InputAction> {
        let count = if self.count == 0 { 1 } else { self.count };

        let action = match self.operator.take() {
            Some(op) => InputAction::Operation {
                count,
                operator: op,
                motion,
            },

            None => InputAction::Move {count, motion},
        };

        self.clear_op();
        Some(action)
    }

    fn handle_insert_mode(
        &mut self,
        key: KeyCode
    ) -> Option<InputAction> {
        return None;
    }

    fn resolve_action(
        action: Option<InputAction>
    ) -> Option<ResolvedCommand> {
        match action {
            Some(InputAction::Move { count, motion }) => {
                Self::resolve_move(count, motion.unwrap())
            }

            Some(InputAction::Operation {
                count,
                operator,
                motion,
            }) => Self::resolve_operation(count, operator, motion),

            Some(InputAction::Command(cmd)) => Self::resolve_command(cmd),

            None => None
        }
    }

    fn resolve_move(
        count: u32,
        motion: Motion,
    ) -> Option<ResolvedCommand> {
        let cmd = match motion.move_type {
            Move::Vertical => LocalCommand::MoveLocalCursor { dx: 0, dy: count as i32 * motion.dir as i32 },
            Move::Horizontal => LocalCommand::MoveLocalCursor { dx: count as i32 * motion.dir as i32, dy: 0 },

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

            // TODO: Clearly not good
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
        &mut self,
        key: KeyCode
    ) -> Option<InputAction> {
        match key {
            KeyCode::Enter => {
                let cmd = self.command_buffer.clone();
                self.clear_command();
                self.mode = Mode::Normal;

                Some(InputAction::Command(cmd))
            }

            KeyCode::Char(c) => {
                self.command_buffer.insert(self.command_cursor, c);
                self.command_cursor += 1;
                None
            }

            KeyCode::Delete => {
                if (0..self.command_buffer.len()).contains(&self.command_cursor) {
                    self.command_buffer.remove(self.command_cursor);
                }

                None
            }

            KeyCode::Backspace => {
                if self.command_buffer.len() > 0 && self.command_cursor > 0 {
                    self.command_cursor -= 1;
                    self.command_buffer.remove(self.command_cursor);
                } else {
                    self.clear_command();
                    self.mode = Mode::Normal;
                }

                None
            }

            KeyCode::Left => {
                self.command_cursor = self.command_cursor.saturating_sub(1);
                None
            }

            KeyCode::Right => {
                self.command_cursor = (self.command_cursor + 1)
                    .min(self.command_buffer.len());
                None
            }

            _ => None
        }
    }
}

