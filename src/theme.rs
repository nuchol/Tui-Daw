use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, BorderType},
    text::Line,
    layout::Rect,
};

pub struct UIStyle;
impl UIStyle {
    // Colours
    pub const BASE_COLOUR: Color = Color::Gray;
    pub const UNFOCUSED_COLOUR: Color = Color::White;

    pub const MAIN_COLOUR: Color = Color::Blue;
    pub const ACCENT_COLOUR: Color = Color::Yellow;

    // Style
    pub const BORDER_TYPE: BorderType = BorderType::Rounded;

    pub fn window_border(title: &str, focused: bool) -> Block<'_> {
        Block::bordered()
            .title(Line::from(title)
                .style(Style::default().fg(Self::UNFOCUSED_COLOUR))
                .centered())
            .borders(Borders::ALL)
            .border_type(Self::BORDER_TYPE)
            .border_style(Style::default().fg(
                if focused { Self::MAIN_COLOUR } else { Self::UNFOCUSED_COLOUR }))
    }

    pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let width = (area.width * percent_x) / 100;
        let height = (area.height * percent_y) / 100;
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;
        
        Rect { x, y, width, height }
    }
}
