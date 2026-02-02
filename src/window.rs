use std::collections::HashMap;

use crate::input::LocalCommand;

use ratatui::{
    layout::{ Rect, Direction, Layout, Constraint },
    Frame,
};

use crate::widgets::splashscreen::SplashScreen;

pub enum LayoutNode {
    Window(usize),

    Split {
        direction: Direction,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

pub struct WindowManager {
    focused: Option<usize>,
    windows: HashMap<usize, Box<dyn Window>>,
    pub layout_tree: LayoutNode,
    last_window_id: usize,
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
        }
    }

    pub fn push_window<W>(&mut self, window: W) -> usize
    where W: Window + 'static {
        self.last_window_id += 1;
        self.windows.insert(self.last_window_id, Box::new(window));
        self.last_window_id
    }

    pub fn render_layout(&mut self, frame: &mut Frame, area: Rect) {
        Self::do_render_layout(
            frame,
            &self.layout_tree,
            area,
            &mut self.windows,
            self.focused
        );
    }

    fn do_render_layout(
        frame: &mut Frame,
        node: &LayoutNode,
        area: Rect,
        windows: &mut HashMap<usize, Box<dyn Window>>,
        focused: Option<usize>,
    ) {
        match node {
            LayoutNode::Window(id) => {
                let window = windows.get_mut(&id).unwrap();
                let focused = focused == Some(*id);

                window.render(frame, area, focused);
            },

            LayoutNode::Split { direction, ratio, first, second } => {
                let layout = Layout::default()
                    .direction(*direction)
                    .constraints(vec![
                        Constraint::Percentage((ratio * 100.0) as u16),
                        Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                    ])
                    .split(area);

                Self::do_render_layout(frame, &first, layout[0], windows, focused);
                Self::do_render_layout(frame, &second, layout[1], windows, focused);
            }
        }
    }

    pub fn split_current_window<W>(
        &mut self,
        direction: Direction,
        new_window: W
    ) -> bool 
    where W: Window + 'static
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

    pub fn handle_input(&mut self, cmd: LocalCommand) {
        if let Some(id) = self.focused {
            let window = self.windows.get_mut(&id).unwrap();
            window.handle_input(cmd);
        }
    }
}

pub trait Window {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_input(&mut self, cmd: LocalCommand);
}
