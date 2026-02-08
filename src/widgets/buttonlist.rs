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
pub struct ButtonListState<'a> {
    items: Vec<Button<'a>>,
    hovered: Option<usize>,
}

impl<'a> ButtonListState<'a> {
    pub fn new(buttons: Vec<Button<'a>>) -> Self {
        Self {
            items: buttons,
            hovered: None,
        }
    }

    pub fn hovered(&mut self, hovered: Option<usize>) -> &mut Self {
        self.hovered = hovered;
        self
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
    hovered_style: Style,
    style: Style,
}

impl<'a> ButtonList<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            hovered_style: Style::default().add_modifier(Modifier::REVERSED),
            style: Style::default(),
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
            if i as u16 >= list_area.height { break; }

            let row = Rect {
                x: list_area.left() + 1,
                y: list_area.top() + current_height,
                width: list_area.width - 2,
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

