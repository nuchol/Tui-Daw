use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line},
    widgets::{Block, StatefulWidget},
    prelude::*,
};

pub struct Button<'a> {
    pub label: Line<'a>,
    pub height: u16,
    pub style: Style,
}

#[derive(Default)]
pub struct ButtonListState {
    hovered: Option<usize>,
}

impl ButtonListState {
    pub fn hovered(&mut self, hovered: Option<usize>) -> &mut Self {
        self.hovered = hovered;
        self
    }
}

pub struct ButtonList<'a> {
    items: Vec<Button<'a>>,
    block: Option<Block<'a>>,
    hovered_style: Style,
    style: Style,
}

impl<'a> ButtonList<'a> {
    pub fn new(items: Vec<Button<'a>>) -> Self {
        Self {
            items,
            block: None,
            hovered_style: Style::default().add_modifier(Modifier::REVERSED),
            style: Style::default(),
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            block: None,
            hovered_style: Style::default().add_modifier(Modifier::REVERSED),
            style: Style::default(),
        }
    }

    pub fn add_button(&mut self, button: Button<'a>) {
        self.items.push(button);
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
}

impl StatefulWidget for ButtonList<'_> {
    type State = ButtonListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);

        let list_area = self.block.inner_if_some(area);
        self.block.render(list_area, buf);

        if list_area.height == 0 { return; }

        let mut current_height = 1;
        for (i, button) in self.items.iter().enumerate() {
            if i as u16 >= list_area.height { break; }

            let row = Rect {
                x: list_area.left() + 1,
                y: list_area.top() + current_height,
                width: list_area.width,
                height: button.height,
            };

            current_height += button.height;

            let button_style = self.style.patch(button.style);
            buf.set_style(row, button_style);

            if state.hovered == Some(i) {
                buf.set_style(row, self.hovered_style);
            }

            button.label.clone().render(row, buf);
        }
    }
}

