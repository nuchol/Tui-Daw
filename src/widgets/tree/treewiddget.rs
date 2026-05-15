use std::marker::PhantomData;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::theme::{ResolvedTheme, ThemeKey};

use super::{
    flatten::flatten_visible,
    node::NodeKind,
    state::TreeState,
};

pub struct TreeWidget<'a, T> {
    block: Option<Block<'a>>,
    highlight_style: Style,
    base_style: Style,
    branch_style: Style,
    leaf_style: Style,
    collapsed_icon: &'a str,
    expanded_icon: &'a str,
    leaf_icon: &'a str,

    _comiler_happy: PhantomData<T>,
}

impl<'a, T> TreeWidget<'a, T> {
    pub fn new(theme: &ResolvedTheme) -> Self {
        Self {
            block: None,
            highlight_style: theme.get(ThemeKey::Cursor),
            base_style: theme.get(ThemeKey::Normal),
            branch_style: theme.get(ThemeKey::Normal),
            leaf_style: theme.get(ThemeKey::Normal),
            collapsed_icon: "",
            expanded_icon: "",
            leaf_icon: " ",

            _comiler_happy: PhantomData
        }
    }
}

impl<'a, T> TreeWidget<'a, T> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn base_style(mut self, style: Style) -> Self {
        self.base_style = style;
        self
    }

    pub fn branch_style(mut self, style: Style) -> Self {
        self.branch_style = style;
        self
    }

    pub fn leaf_style(mut self, style: Style) -> Self {
        self.leaf_style = style;
        self
    }

    pub fn collapsed_icon(mut self, icon: &'a str) -> Self {
        self.collapsed_icon = icon;
        self
    }

    pub fn expanded_icon(mut self, icon: &'a str) -> Self {
        self.expanded_icon = icon;
        self
    }

    pub fn leaf_icon(mut self, icon: &'a str) -> Self {
        self.leaf_icon = icon;
        self
    }
}

impl<'a, T> StatefulWidget for TreeWidget<'a, T> {
    type State = TreeState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let render_area = match self.block {
            None => area,
            Some(block) => {
                let inner = block.inner(area);
                block.render(area, buf);
                inner
            }
        };

        if render_area.height == 0 || render_area.width == 0 {
            return;
        }

        let flat = flatten_visible(state);
        
        let mut in_scope = false;
        for (i, item) in flat.iter().enumerate() {
            let node = match state.nodes.get(&item.id) {
                Some(n) => n,
                None => continue,
            };

            let is_selected = state.selected() == Some(item.id);

            let mut indent = if in_scope {"│ "} else {"  "}
                .repeat(item.depth.saturating_sub(1));

            if item.depth > 0 {
                if item.is_last {
                    in_scope = false;
                    indent.push_str("└ ");
                } else {
                    in_scope = true;
                    indent.push_str("│ ");
                }
            }

            let ispan = Span::styled(indent, self.base_style);

            let mut icon = match node.kind {
                NodeKind::Leaf => self.leaf_icon.to_string(),
                NodeKind::Branch { expanded } => if expanded {
                    self.expanded_icon.to_string()
                } else {
                    self.collapsed_icon.to_string()
                }
            };

            icon.push(' ');

            let row_style = if is_selected {
                self.highlight_style
            } else {
                let extra_style = match node.kind() {
                    NodeKind::Branch { .. } => self.branch_style,
                    NodeKind::Leaf => self.leaf_style,
                };

                self.base_style.patch(extra_style)
            };

            let label = Span::styled(node.label(), self.base_style);
            let line = Line::from_iter(vec![ispan, icon.into(), label])
                .style(row_style);

            let row_area = Rect {
                x: render_area.left(),
                y: render_area.top() + i as u16,
                width: render_area.width,
                height: 1,
            };

            line.render(row_area, buf);
        }
    }
}
