use ratatui::{
    Frame, layout::Rect, widgets::Paragraph,
    text::{Line, Span},
    style::{Style, Modifier},
};

use crate::{AppState, input::Mode};


pub struct CommandLine;

impl CommandLine {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &AppState,
    ) {
        frame.render_widget(Paragraph::new(
            match state.mode {
                Mode::Normal => Self::normal_line(state, area.width),
                Mode::Insert => Line::from(Self::get_mode(state)),
                Mode::Command => Self::command_line(state),
            }), area
        );
    }

    fn normal_line(state: &AppState, width: u16) -> Line<'_> {
        let left = Self::get_mode(state);
        let right = state.input_state.display();
        let spacing = left.len() + right.len();
        
        Line::from(vec![
            Span::styled(left,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat((width as usize).saturating_sub(spacing))),
            Span::raw(right),
        ])
    }

    fn command_line(state: &AppState) -> Line<'_> {
        let cmd = &state.command_state;

        let cursor = cmd.cursor.min(cmd.buffer.len());

        let (before, after) = cmd.buffer.split_at(cursor);

        Line::from(vec![
            Span::raw(":"),
            Span::raw(before),
            Span::styled(
                after.chars().next().unwrap_or(' ' ).to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::raw(
                after.chars().skip(1).collect::<String>()
            ),
        ])
    }

    fn get_mode(state: &AppState) -> String {
        format!(
            "-- {} --",
            state.mode.to_string().to_uppercase()
        )
    }
}
