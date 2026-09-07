use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph
};

use crate::{
    input::{Input, Mode}, log, theme::{ResolvedTheme, ThemeKey}
};

pub struct CommandLine;
impl CommandLine {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        input: &Input,
        theme: &ResolvedTheme,
    ) {
        frame.render_widget(Paragraph::new(
            Self::format_line(input, area.width, theme)),
            area,
        );
    }

    fn format_line<'a>(
        input: &'a Input,
        width: u16,
        theme: &ResolvedTheme,
    ) -> Line<'a> {
        let mode_span = Self::get_mode(&input.mode, theme);

        let content = match input.mode {
            Mode::Normal | Mode::Insert => vec![match log::current() {
                Some((msg, level)) => Self::get_log(msg, level, theme),
                None => Span::default(),
            }],

            Mode::Command => Self::command_line(
                &input.command_buffer,
                input.command_cursor,
                theme
            ),
        };

        let right = input.display_op();
        let content_len: usize = content.iter().map(|s| s.content.len()).sum();
        let mode_len: usize = mode_span.iter().map(|s| s.content.len()).sum();
        let spacing = mode_len + content_len + right.len();

        let mut spans = mode_span;
        spans.push(Span::raw(" "));
        spans.extend(content);
        spans.push(Span::raw(" ".repeat((width as usize).saturating_sub(spacing))));
        spans.push(Span::raw(right));

        Line::from(spans)
    }

    fn command_line<'a>(
        cmd: &'a str,
        cursor: usize,
        theme: &ResolvedTheme
    ) -> Vec<Span<'a>> {
        let (before, after) = cmd.split_at(cursor.min(cmd.len()));

        vec![
            Span::raw(":"),
            Span::raw(before.to_string()),
            Span::styled(
                after.chars().next().unwrap_or(' ').to_string(),
                theme.get(ThemeKey::Cursor)
            ),
            Span::raw(
                after.chars().skip(1).collect::<String>()
            ),
        ]
    }

    fn get_log(
        msg: String,
        level: log::LogLevel,
        theme: &ResolvedTheme,
    ) -> Span<'static> {
        Span::styled(msg, match level {
            log::LogLevel::INFO => theme.get(ThemeKey::Normal),
            log::LogLevel::WARN => theme.get(ThemeKey::WarnMsg),
            log::LogLevel::ERROR => theme.get(ThemeKey::ErrorMsg),
        })
    }

    fn get_mode<'a>(
        mode: &'a Mode,
        theme: &ResolvedTheme
    ) -> Vec<Span<'a>> {
        let style = match mode {
            Mode::Normal => theme.get(ThemeKey::ModeNormal),
            Mode::Insert => theme.get(ThemeKey::ModeInsert),
            Mode::Command => theme.get(ThemeKey::ModeCommand),
        };

        vec![Span::styled(format!(" {} ",
            mode.to_string().to_uppercase()), style),
            Span::styled("", Style::default().fg(style.bg.unwrap_or(Color::Reset))),
        ]
    }
}
