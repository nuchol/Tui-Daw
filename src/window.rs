use std::collections::HashMap;

use crate::input::LocalCommand;

use ratatui::{
    layout::{ Rect, Direction, Layout, Constraint },
    Frame,
};

pub enum LayoutNode {
    Window(usize),

    Split {
        direction: Direction,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

pub struct WindowStack {
    focused: Option<usize>,
    windows: HashMap<usize, Box<dyn Window>>,
    last_window_id: usize,
}

impl WindowStack {
    pub fn new() -> Self {
        Self {
            focused: None,
            windows: HashMap::new(),
            last_window_id: 0,
        }
    }

    pub fn push_window<W>(&mut self, window: W)
    where W: Window + 'static {
        self.windows.insert(window.id(), Box::new(window));
    }

    pub fn create_window(&mut self) -> usize {
        self.last_window_id += 1;
        self.last_window_id
    }

    pub fn render_layout(&mut self, frame: &mut Frame, node: &LayoutNode, area: Rect) {
        match node {
            LayoutNode::Window(id) => {
                let window = self.windows.get_mut(&id).unwrap();
                let focused = self.focused == Some(*id);

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

                self.render_layout(frame, &first, layout[0]);
                self.render_layout(frame, &second, layout[1]);
            }
        }
    }

    pub fn handle_input(&mut self, cmd: LocalCommand) {
        if let Some(id) = self.focused {
            let window = self.windows.get_mut(&id).unwrap();
            window.handle_input(cmd);
        }
    }
}

pub trait Window {
    fn id(&self) -> usize;

    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_input(&mut self, cmd: LocalCommand);
}
