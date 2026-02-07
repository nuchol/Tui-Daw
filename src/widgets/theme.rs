use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType},
    text::Line,
};

pub struct UIStyle;
impl UIStyle {
    pub fn window_border(title: &str, focused: bool) -> Block<'_> {
        Block::bordered()
            .title(Line::from(title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(
                if focused { Color::LightGreen } else { Color::White }
            ))
    }
}
