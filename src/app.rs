use crate::input::{
    Input, InputState, Mode, CommandState,
    ResolvedCommand, EditorCommand
};

use crate::widgets::{
    pianoroll::PianoRollState,
    splashscreen::SplashScreen,
    commandline::CommandLine,
};

use crate::window::{WindowManager, LayoutNode};

use color_eyre::eyre::{Ok, Result};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
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
        // let tick_rate = Duration::from_millis(16);
        // let mut last_tick = Instant::now();

        while state.running {
            // let timeout = tick_rate
            //     .checked_sub(last_tick.elapsed())
            //     .unwrap_or(Duration::ZERO);

            if event::poll(Duration::from_millis(16))? &&
                let Event::Key(key) = event::read()? &&
                let Some(cmd) = Input::handle_keypress(state, key.code) {
                    App::dispatch_command(state, cmd);
            }

            terminal.draw(|frame| App::render(frame, state))?;
            
            // if last_tick.elapsed() >= tick_rate {
            //     last_tick = Instant::now();
            // }
        }

        Ok(())
    }

    fn dispatch_command(state: &mut AppState, command: ResolvedCommand) {
        match command {
            ResolvedCommand::Editor(cmd) => App::execute_editor_command(state, cmd),
            ResolvedCommand::Local(cmd) => state.windows.handle_input(cmd),
        }
    }

    fn execute_editor_command(state: &mut AppState, command: EditorCommand) {
        match command {
            EditorCommand::Quit => state.running = false,
            EditorCommand::Split { direction } => { state.windows.split_current_window(direction, SplashScreen::default()); },
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
