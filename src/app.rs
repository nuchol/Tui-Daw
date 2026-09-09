use std::time::Duration;

use crate::log;
use crate::theme::{ResolvedTheme, ThemeKey, ThemeRegistry};
use crate::widgets::commandline::CommandLine;
use crate::windowpanes::window::{WindowManager, WindowPaneType};
use crate::input::{Input, ResolvedCommand, EditorCommand};
use color_eyre::eyre::{Ok, Result};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyCode},
    layout::{ Direction, Layout, Constraint },
    style::Style,
};

pub struct App {
    pub running: bool,
    pub input: Input,
    pub windows: WindowManager,
    pub theme: ResolvedTheme,
}

impl App {
    pub fn new() -> Self {
        // TODO: Add a default theme and fall back to it if no theme file is found
        let src = std::fs::read_to_string("./res/themes/catppuccin.toml")
            .expect("Could not read theme file");
        let theme_registry = ThemeRegistry::from_toml(&src)
            .expect("Could not parse theme file");
        let theme = ResolvedTheme::from_registry(&theme_registry);

        Self {
            running: true,
            input: Input::default(),
            windows: WindowManager::new(),
            theme,
        }
    }

    pub fn run_loop(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            if event::poll(Duration::from_millis(16))? &&
                let Event::Key(key) = event::read()? {
                self.handle_keyevent(key);
            }

            terminal.draw(|frame| self.render(frame))?;
        }

        Ok(())
    }

    fn handle_keyevent(&mut self, key: KeyEvent) {
        // TODO: Since escape is being pressed for more than one frame,
        //       all popups are being popped, should be on key pressed.
        if self.windows.is_popup_active()
            && key.code == KeyCode::Esc && key.kind == KeyEventKind::Press {
            self.windows.pop_popup();
        }

        if let Some(resolved) = self.input.handle_keypress(key.code) {
            match resolved {
                ResolvedCommand::Editor(cmd) => self.execute_editor_command(cmd),
                ResolvedCommand::Universal(cmd) => self.windows.handle_universal(cmd),
                ResolvedCommand::Local(cmd) => {
                    if let Some(editor_cmd) = self.windows.handle_input(cmd) {
                        self.execute_editor_command(editor_cmd);
                    }
                }
            }
        }
    }

    fn execute_editor_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Quit => self.running = false,
            EditorCommand::OpenWindow { display, window } => {
                self.windows.pop_popup();
                match display {
                    WindowPaneType::Popup => { self.windows.push_popup(window); }
                    WindowPaneType::Direction { direction } => {
                        self.windows.split_current_window(direction, window);
                    }
                }
            }
            EditorCommand::Theme(theme) => {
                log::log(format!("TODO: Set theme to \"{}\"", theme), log::LogLevel::INFO);
            }
            
            _ => ()
        };
    }

    fn render(&mut self, frame: &mut Frame) {
        let base_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(100),
                Constraint::Length(1),
            ])
            .split(frame.area());

        CommandLine::render(frame, base_layout[1], &self.input, &self.theme);

        self.windows.render_layout(frame, base_layout[0], &self.theme);
    }

    pub fn get_style(&self, key: ThemeKey) -> Style {
        self.theme.get(key)
    }
}
