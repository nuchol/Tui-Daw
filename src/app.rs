use crate::input::{
    Input, InputState, Mode, CommandState,
    ResolvedCommand, EditorCommand, LocalCommand
};

use crate::widgets::{
    pianoroll::{PianoRoll, PianoRollState},
    commandline::CommandLine,
};

use color_eyre::eyre::{Ok, Result};
use ratatui::prelude::*;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
    widgets::{Paragraph, Block, Borders, BorderType}
};

use std::time::{Duration, Instant};

pub struct AppState {
    pub running: bool,
    pub mode: Mode,
    pub input_state: InputState,
    pub command_state: CommandState,
    pub piano_roll: PianoRollState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            running: true,
            mode: Mode::Normal,
            input_state: InputState::new(),
            command_state: CommandState::default(),
            piano_roll: PianoRollState::new(),
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
            ResolvedCommand::Editor(cmd) => {
                match cmd {
                    EditorCommand::Quit => state.running = false,
                    _ => ()
                }
            },
            ResolvedCommand::Local(cmd) => {},
        }
    }

    fn execute_editor_command(state: &mut AppState, command: EditorCommand) {

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

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(base_layout[0]);

        let piano_block = rounded_block(" Piano Roll ");

        frame.render_widget(piano_block.clone(), layout[0]);

        frame.render_stateful_widget(
            PianoRoll::default(),
            piano_block.inner(layout[0]),
            &mut state.piano_roll,
        );

        frame.render_widget(
            Paragraph::new("Options")
                .block(rounded_block(" Options ")),
            layout[1]
        );
    }
}

fn rounded_block(title: &str) -> Block<'_> {
    return Block::bordered()
        .title(Line::from(title).centered())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
}

