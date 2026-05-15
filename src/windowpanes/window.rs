use std::collections::HashMap;

use crate::input::{EditorCommand, LocalCommand};
use crate::theme::{ResolvedTheme, UIStyle};
use crate::windowpanes::splashscreen::SplashScreen;
use ratatui::widgets::Clear;
use ratatui::{
    layout::{ Rect, Direction, Layout, Constraint },
    Frame,
};

pub trait Window {
    fn title(&self) -> &str;
    fn render(&mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        theme: &ResolvedTheme
    );
    fn handle_input(&mut self, cmd: LocalCommand) -> Option<EditorCommand>;
}

pub enum LayoutNode {
    Window(usize),

    Split {
        direction: Direction,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

#[derive(Clone, Copy)]
pub enum WindowPaneType {
    Popup,
    Direction { direction: Direction },
}

pub struct WindowManager {
    focused: Option<usize>,
    windows: HashMap<usize, Box<dyn Window>>,
    layout_tree: LayoutNode,
    last_window_id: usize,
    popup_stack: Vec<usize>,
}

impl WindowManager {
    pub fn new() -> Self {
        let base_id = 0;
        let mut windows: HashMap<usize, Box<dyn Window>> = HashMap::new();
        windows.insert(base_id, Box::new(SplashScreen::default()));

        Self {
            focused: Some(base_id),
            windows: windows,
            layout_tree: LayoutNode::Window(base_id),
            last_window_id: base_id,
            popup_stack: Vec::new(),
        }
    }

    fn push_window(&mut self, window: Box<dyn Window>) -> usize {
        self.last_window_id += 1;
        self.windows.insert(self.last_window_id, window);
        self.last_window_id
    }

    fn remove_window(&mut self, id: usize) -> Option<usize> {
        if self.windows.remove(&id).is_some() {
            Some(id)
        } else {
            None
        }
    }

    pub fn push_popup(&mut self, window: Box<dyn Window>) {
        let id = self.push_window(window);
        self.popup_stack.push(id);
    }

    pub fn pop_popup(&mut self) -> Option<usize> {
        if let Some(id) = self.popup_stack.pop() {
            self.remove_window(id)
        } else {
            None
        }
    }

    pub fn is_popup_active(&self) -> bool {
        !self.popup_stack.is_empty()
    }

    pub fn split_current_window(
        &mut self,
        direction: Direction,
        new_window: Box<dyn Window>,
    ) -> bool 
    {
        let Some(focus) = self.focused else { return false };

        let old_id = match Self::get_focused_node(&mut self.layout_tree, focus) {
            Some(LayoutNode::Window(id)) => *id,
            _ => return false,
        };

        let new_id = self.push_window(new_window);

        if let Some(node) = Self::get_focused_node(
            &mut self.layout_tree, focus
        ) {
            // Splashscreen is id 0, should be more robust
            if old_id == 0 {
                *node = LayoutNode::Window(new_id);
                self.set_focuesed(new_id);
                return true;
            }

            *node = LayoutNode::Split {
                direction: direction,
                ratio: 0.5,
                first: Box::new(LayoutNode::Window(old_id)),
                second: Box::new(LayoutNode::Window(new_id)),
            };

            self.set_focuesed(new_id);

            return true;
        }

        false
    }

    pub fn render_layout(&mut self, frame: &mut Frame, area: Rect, theme: &ResolvedTheme) {
        let popup_id = self.popup_stack.last().copied();
        let focused = popup_id.or(self.focused);

        frame.render_widget(Clear, area);
        Self::do_render_layout(
            frame,
            &self.layout_tree,
            area,
            &mut self.windows,
            focused,
            theme,
        );

        if self.popup_stack.is_empty() { return; }

        let popup = self.windows.get_mut(&popup_id.unwrap()).unwrap();

        let popup_area = UIStyle::centered_rect(50, 50, area);
        let block = UIStyle::window_border(popup.title(), focused == popup_id, theme);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(&block, popup_area);
        popup.render(frame, block.inner(popup_area), true, theme);
    }

    fn do_render_layout(
        frame: &mut Frame,
        node: &LayoutNode,
        area: Rect,
        windows: &mut HashMap<usize, Box<dyn Window>>,
        focused: Option<usize>,
        theme: &ResolvedTheme,
    ) {
        match node {
            LayoutNode::Window(id) => {
                let window = windows.get_mut(&id).unwrap();
                let is_focused = focused == Some(*id);

                let block = UIStyle::window_border(window.title(), is_focused, theme);

                frame.render_widget(&block, area);

                window.render(frame, block.inner(area), is_focused, theme);
            },

            LayoutNode::Split { direction, ratio, first, second } => {
                let layout = Layout::default()
                    .direction(*direction)
                    .constraints(vec![
                        Constraint::Percentage((ratio * 100.0) as u16),
                        Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area);

                Self::do_render_layout(frame, &first,  layout[0], windows, focused, theme);
                Self::do_render_layout(frame, &second, layout[1], windows, focused, theme);
            }
        }
    }

    fn get_focused_node(node: &mut LayoutNode, focused: usize) -> Option<&mut LayoutNode> {
        match node {
            LayoutNode::Window(id) if *id == focused => Some(node),

            LayoutNode::Split { first, second, .. } => {
                Self::get_focused_node(first, focused)
                    .or_else(|| Self::get_focused_node(second, focused))
            }

            LayoutNode::Window(_) => None,
        }
    }

    pub fn set_focuesed(&mut self, id: usize) {
        self.focused = Some(id);
    }

    pub fn handle_input(&mut self, cmd: LocalCommand) -> Option<EditorCommand> {
        let window_id = self.popup_stack.last();
        let focused = window_id.copied().or(self.focused);

        if let Some(id) = focused {
            let window = self.windows.get_mut(&id).unwrap();
            return window.handle_input(cmd);
        }

        None
    }
}
