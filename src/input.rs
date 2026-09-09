use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Direction;
use std::collections::HashMap;
use std::fmt;

use crate::log;
use crate::windowpanes::{
    window::{Window, WindowPaneType},
    windowselect::WindowSelect,
};

#[derive(Default)]
pub enum Mode {
    #[default]
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

#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    Delete,
    Yank,
    Undo,
    Confirm,
}

#[derive(PartialEq, Eq)]
pub enum Dir {
    Forward = 1,
    Backward = -1,
}

pub struct Motion {
    pub key: KeyCode,
    pub dir: Dir,
}

impl Motion {
    pub fn new(key: KeyCode, dir: Dir) -> Self {
        Self {key, dir}
    }
}

pub enum InputAction {
    KeyPress {
        count: u32,
        key: KeyCode,
    },

    SemanticOperation {
        count: u32,
        motion: Option<Motion>,
        operator: Operator,
    },

    Universal(UniversalCommand),
    Command(String),
}

pub enum EditorCommand {
    Undo { count: u32, },
    Redo { count: u32, },
    // Yank {
    //     count: u32,
    //     motion: Motion,
    // },
    // Paste {
    //     count: u32,
    //     motion: Motion,
    // },
    // Mute {
    //     count: u32,
    //     motion: Motion,
    // },
    // Solo {
    //     count: u32,
    //     motion: Motion,
    // },
    // Delete {
    //     count: u32,
    //     motion: Motion,
    // },
    // Bpm {
    //     bpm: u32,
    // },
    OpenWindow {
        display: WindowPaneType,
        window: Box<dyn Window>,
    },
    Theme(String),
    Quit,
}

pub enum UniversalCommand {
    Horizontal { count: u32, dir: Dir },
    Vertical { count: u32, dir: Dir },
    Next { count: u32, dir: Dir },

    Start,
    End,

    ScrollUp(u32),
    ScrollDown(u32),
}

pub enum LocalCommand {
    KeyPress {
        count: u32,
        key: KeyCode,
    },
    Operator {
        count: u32,
        motion: Option<Motion>,
        operator: Operator,
    },

    // TODO: Confirm is both op and local command.
    Confirm,
}

pub enum ResolvedCommand {
    Editor(EditorCommand),
    Universal(UniversalCommand),
    Local(LocalCommand),
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

    pub fn handle_keypress(&mut self, key: KeyCode) -> Option<ResolvedCommand> {
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

        if action.is_some() {
            self.clear_op();
            self.clear_command();
        }

        Self::resolve_action(action)
    }

    fn handle_normal_mode(&mut self, key: KeyCode) -> Option<InputAction> {
        let count = if self.count == 0 { 1 } else { self.count };
        match key {
            KeyCode::Char('i') => {
                self.clear_op();
                self.mode = Mode::Insert;
                None
            }

            KeyCode::Char(':') | KeyCode::Char(';') => {
                self.clear_op();
                self.clear_command();
                self.mode = Mode::Command;
                None
            }

            KeyCode::Char('.') => {
                // TODO: Repeat last command
                None
            }

            KeyCode::Char(c) if c.is_ascii_digit() => {
                let d = c.to_digit(10).unwrap();
                self.count = self.count.saturating_mul(10).saturating_add(d);

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

            KeyCode::Char('u') => Some(InputAction::SemanticOperation {
                count,
                operator: Operator::Undo,
                motion: None,                
            }),

            KeyCode::Enter => Some(InputAction::SemanticOperation {
                count,
                operator: Operator::Confirm,
                motion: None,
            }),

            // Physical Motions (universal)
            KeyCode::Char('h') => Some(InputAction::Universal(UniversalCommand::Horizontal { count, dir: Dir::Backward, })),
            KeyCode::Char('j') => Some(InputAction::Universal(UniversalCommand::Vertical { count, dir: Dir::Backward, })),
            KeyCode::Char('k') => Some(InputAction::Universal(UniversalCommand::Vertical { count, dir: Dir::Forward, })),
            KeyCode::Char('l') => Some(InputAction::Universal(UniversalCommand::Horizontal { count, dir: Dir::Forward, })),
            KeyCode::Char('n') => Some(InputAction::Universal(UniversalCommand::Next { count, dir: Dir::Forward, })),
            KeyCode::Char('N') => Some(InputAction::Universal(UniversalCommand::Next { count, dir: Dir::Backward, })),

            _ => Some(InputAction::KeyPress { count, key }),
        }
    }

    // TODO
    fn handle_insert_mode(&mut self, key: KeyCode) -> Option<InputAction> {
        return None;
    }

    fn resolve_action(action: Option<InputAction>) -> Option<ResolvedCommand> {
        match action? {
            InputAction::Command(cmd) => Self::resolve_command(cmd),
            InputAction::Universal(cmd) => Some(ResolvedCommand::Universal(cmd)),

            InputAction::SemanticOperation { count, motion, operator } => Some(ResolvedCommand::Local(
                LocalCommand::Operator { count, motion, operator }
            )),

            InputAction::KeyPress { count, key } => Some(ResolvedCommand::Local(
                LocalCommand::KeyPress {count, key }
            )),
        }
    }

    fn resolve_command(command: String) -> Option<ResolvedCommand> {
        let tokens: Vec<&str> = command.split(' ').collect();
        match tokens[0] {
            "q" | "quit" => Some(ResolvedCommand::Editor(EditorCommand::Quit)),

            // We want to split accross the opposite direction since
            // splitting adds another window on the 'direction' axis.
            "vsplit" => Some(ResolvedCommand::Editor(EditorCommand::OpenWindow {
                display: WindowPaneType::Popup,
                window: Box::new(WindowSelect::new(WindowPaneType::Direction {
                    direction: Direction::Horizontal,
                })),
            })),

            "hsplit" => Some(ResolvedCommand::Editor(EditorCommand::OpenWindow {
                display: WindowPaneType::Popup,
                window: Box::new(WindowSelect::new(WindowPaneType::Direction {
                    direction: Direction::Vertical,
                })),
            })),

            // TODO: Clearly not good
            "theme" => Some(ResolvedCommand::Editor(EditorCommand::Theme(
                tokens[1].to_string(),
            ))),

            _ => {
                log::log(
                    format!("Not a recognised command: {}", command.as_str()),
                    log::LogLevel::ERROR,
                );
                None
            }
        }
    }

    fn handle_command_mode(&mut self, key: KeyCode) -> Option<InputAction> {
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
                self.command_cursor = (self.command_cursor + 1).min(self.command_buffer.len());
                None
            }

            _ => None,
        }
    }
}
