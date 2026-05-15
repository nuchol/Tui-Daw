use std::time::Duration;

use crate::input::{
    VimInput, InputState, Mode, CommandState,
    ResolvedCommand, EditorCommand
};

use crate::log;
use crate::theme::{ResolvedTheme, ThemeKey, ThemeRegistry};
use crate::widgets::commandline::CommandLine;
use crate::windowpanes::window::{WindowManager, WindowPaneType};
use color_eyre::eyre::{Ok, Result};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyCode},
    layout::{ Direction, Layout, Constraint },
    style::Style,
};

pub struct AppState {
    pub running: bool,
    pub mode: Mode,
    pub input_state: InputState,
    pub command_state: CommandState,
    pub windows: WindowManager,
    pub theme_registry: ThemeRegistry,
    pub theme: ResolvedTheme,
}

impl AppState {
    pub fn new() -> Self {
        // TODO: Add a default theme and fall back to it if no theme file is found
        let src = std::fs::read_to_string("./res/themes/catpuccin.toml")
            .expect("Could not read theme file");
        let theme_registry = ThemeRegistry::from_toml(&src)
            .expect("Could not parse theme file");
        let theme = ResolvedTheme::from_registry(&theme_registry);

        Self {
            running: true,
            mode: Mode::Normal,
            input_state: InputState::new(),
            command_state: CommandState::default(),
            windows: WindowManager::new(),
            theme_registry,
            theme,
        }
    }

    pub fn get_style(&self, key: ThemeKey) -> Style {
        self.theme.get(key)
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
        // TODO: Since escape is being pressed for more than one frame,
        //       all popups are being popped, should be on key pressed.
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
                    if let Some(editor_cmd) = state.windows.handle_input(local_cmd) {
                        Self::execute_editor_command(state, editor_cmd);
                    }
                },
            }
        }
    }

    fn execute_editor_command(state: &mut AppState, command: EditorCommand) {
        match command {
            EditorCommand::Quit => state.running = false,
            EditorCommand::OpenWindow { display, window } => {
                state.windows.pop_popup();
                match display {
                    WindowPaneType::Popup => { state.windows.push_popup(window); }
                    WindowPaneType::Direction { direction } => {
                        state.windows.split_current_window(direction, window);
                    }
                }
            }
            EditorCommand::Theme { theme } => {
                log::log(format!("TODO: Set theme to \"{}\"", theme), log::LogLevel::INFO);
            }
            
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

        state.windows.render_layout(frame, base_layout[0], &state.theme);
    }
}
