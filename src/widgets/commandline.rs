use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph
};

use crate::{AppState, input::Mode, log, theme::UIStyle};

pub struct CommandLine;

impl CommandLine {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &AppState,
    ) {

        frame.render_widget(Paragraph::new(
            Self::format_line(state, area.width)),
            area);
    }

    fn format_line(state: &AppState, width: u16) -> Line<'_> {
        let mode = Self::get_mode(state);

        let content = match state.mode {
            Mode::Normal | Mode::Insert => vec![match log::current() {
                Some((msg, level)) => Self::get_log(msg, level),
                None => Span::default(),
            }],

            Mode::Command => Self::command_line(state),
        };

        let right = state.input_state.display();

        let content_len: usize = content.iter().map(|s| s.content.len()).sum();
        let spacing = mode.content.len() + content_len + right.len();

        let mut spans = vec![mode];
        spans.push(Span::raw(" "));
        spans.extend(content);
        spans.push(Span::raw(" ".repeat((width as usize).saturating_sub(spacing))));
        spans.push(Span::raw(right));

        Line::from(spans)
    }

    fn command_line(state: &AppState) -> Vec<Span<'_>> {
        let cmd = &state.command_state;

        let cursor = cmd.cursor.min(cmd.buffer.len());

        let (before, after) = cmd.buffer.split_at(cursor);

        vec![
            Span::raw(":"),
            Span::raw(before),
            Span::styled(
                after.chars().next().unwrap_or(' ').to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Span::raw(
                after.chars().skip(1).collect::<String>()
            ),
        ]
    }

    fn get_log(msg: String, level: log::LogLevel) -> Span<'static> {
        Span::styled(msg, Style::default()
            .fg(UIStyle::ERROR_COLOUR)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::ITALIC))
    }

    fn get_mode(state: &AppState) -> Span<'_> {
        Span::styled(format!(" {} ",
            state.mode.to_string().to_uppercase()),
            Style::default()
                .fg(UIStyle::MAIN_COLOUR)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED))
    }
}
