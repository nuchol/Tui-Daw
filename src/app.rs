use crate::input::{
    VimInput, InputState, Mode, CommandState,
    ResolvedCommand, EditorCommand
};

use crate::widgets::{
    splitselect::SplitSelect,
    commandline::CommandLine,
};

use crate::window::WindowManager;

use color_eyre::eyre::{Ok, Result};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyCode},
    layout::{ Direction, Layout, Constraint },
};

use std::time::Duration;

pub struct AppState {
    pub running: bool,
    pub mode: Mode,
    pub input_state: InputState,
    pub command_state: CommandState,
    pub windows: WindowManager
}

impl AppState {
    pub fn new() -> Self {
        Self {
            running: true,
            mode: Mode::Normal,
            input_state: InputState::new(),
            command_state: CommandState::default(),
            windows: WindowManager::new(),
        }
    }
}

pub struct App;
impl App {
    pub fn run_loop(mut terminal: DefaultTerminal, state: &mut AppState) -> Result<()> {
        while state.running {
            if event::poll(Duration::from_millis(16))? &&
                let Event::Key(key) = event::read()? {
                Self::handle_keyevent(state, key);
            }

            terminal.draw(|frame| Self::render(frame, state))?;
        }

        Ok(())
    }

    fn handle_keyevent(state: &mut AppState, key: KeyEvent) {
        if state.windows.is_popup_active()
            && key.code == KeyCode::Esc {
            state.windows.pop_popup();
        }

        if let Some(cmd) = VimInput::handle_keypress(state, key.code) {
            match cmd {
                ResolvedCommand::Editor(editor_cmd) => {
                    Self::execute_editor_command(state, editor_cmd);
                },

                ResolvedCommand::Local(local_cmd) => {
                    state.windows.handle_input(local_cmd);
                },
            }
        }
    }

    fn execute_editor_command(state: &mut AppState, command: EditorCommand) {
        match command {
            EditorCommand::Quit => state.running = false,

            EditorCommand::Split { direction } => { 
                state.windows.push_popup(SplitSelect::new(direction));
            },
            
            _ => ()
        };
    }

    fn render(frame: &mut Frame, state: &mut AppState) {
        let base_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(100),
                Constraint::Length(1),
            ])
            .split(frame.area());

        CommandLine::render(frame, base_layout[1], state);

        state.windows.render_layout(frame, base_layout[0]);
    }
}
