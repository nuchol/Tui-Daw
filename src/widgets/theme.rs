use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType},
    text::Line,
    layout::Rect,
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

    pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let width = (area.width * percent_x) / 100;
        let height = (area.height * percent_y) / 100;
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;
        
        Rect { x, y, width, height }
    }
}
