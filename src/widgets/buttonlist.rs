use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, StatefulWidget}
};

pub struct Button<'a> {
    pub label: Line<'a>,
    pub height: u16,
    pub style: Style,
}

#[derive(Default)]
pub struct ButtonListState<'a> {
    items: Vec<Button<'a>>,
    hovered: Option<usize>,
    selected: Option<usize>,
}

impl<'a> ButtonListState<'a> {
    pub fn new(buttons: Vec<Button<'a>>) -> Self {
        Self {
            items: buttons,
            hovered: None,
            selected: None
        }
    }

    pub fn hovered(mut self, hovered: Option<usize>) -> Self {
        self.hovered = hovered;
        self
    }

    pub fn get_hovered(&self) -> Option<usize> {
        self.hovered
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn get_selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selected(&mut self, selected: Option<usize>) {
        self.selected = selected
    }

    pub fn get_num_items(&self) -> usize {
        self.items.len()
    }

    pub fn add_button(&mut self, button: Button<'a>) {
        self.items.push(button);
    }

    pub fn first_button(&mut self) {
        if self.items.is_empty() { self.hovered = None; return; }

        self.hovered = Some(0);
    }

    pub fn last_button(&mut self) {
        if self.items.is_empty() { self.hovered = None; return; }

        self.hovered = Some(self.items.len().saturating_sub(1));
    }

    pub fn next_button(&mut self) {
        if self.items.is_empty() { self.hovered = None; return; }

        self.hovered = Some(self.hovered
            .map_or(0, |i| (i + 1).min(self.items.len() - 1))
        );
    }

    pub fn previous_button(&mut self) {
        if self.items.is_empty() { self.hovered = None; return; }
        
        self.hovered = Some(self.hovered
            .map_or(0, |i| i.saturating_sub(1)));
    }

    pub fn jump_buttons(&mut self, count: i32) {
        if self.items.is_empty() { self.hovered = None; return; }
        
        self.hovered = Some(self.hovered
            .map_or(0, |i| (i.saturating_add_signed(count as isize))
            .min(self.items.len() - 1)));
    }

    pub fn no_button(&mut self) {
        self.hovered = None;
    }
}

pub struct ButtonList<'a> {
    block: Option<Block<'a>>,
    style: Style,
    hovered_style: Style,
    padding: u16,
    hovered_character: &'a str,
}

impl<'a> ButtonList<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            hovered_style: Style::default().add_modifier(Modifier::REVERSED),
            style: Style::default(),
            padding: 0,
            hovered_character: "",
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ButtonList<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> ButtonList<'a> {
        self.style = style;
        self
    }

    pub fn hovered_style(mut self, style: Style) -> ButtonList<'a> {
        self.hovered_style = style;
        self
    }

    pub fn padding(mut self, padding: u16) -> ButtonList<'a> {
        self.padding = padding;
        self
    }

    pub fn hovered_character(mut self, character: &'a str) -> ButtonList<'a> {
        self.hovered_character = character;
        self
    }
}

impl<'a> StatefulWidget for ButtonList<'a> {
    type State = ButtonListState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);

        let list_area = self.block.inner_if_some(area);
        self.block.render(list_area, buf);

        if list_area.height == 0 { return; }

        let mut current_height = 1;
        for (i, button) in state.items.iter().enumerate() {
            if (i * self.padding as usize) as u16 >= list_area.height { break; }

            let row = Rect {
                x: list_area.left(),
                y: list_area.top() + current_height,
                width: list_area.width,
                height: button.height,
            };

            current_height += button.height + self.padding;

            let button_style = self.style.patch(button.style);
            buf.set_style(row, button_style);

            let mut label = button.label.clone();
            if state.hovered == Some(i) {
                let mut spans = label.spans;
                let style = spans.first().map(|s| s.style).unwrap_or_default();
                spans.insert(0, Span::styled(self.hovered_character, style));
                label = Line::from(spans);

                buf.set_style(row, self.hovered_style);
            }

            label.clone().render(row, buf);
        }
    }
}

