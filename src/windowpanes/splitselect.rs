use crate::{
    widgets::tree::{node::NodeId, node::NodeKind, state::TreeState, treewiddget::TreeWidget},
    windowpanes::{
        window::{Window, WindowPaneType},
        windowregistry::*,
    }
};

use crate::input::{EditorCommand, LocalCommand};
use crate::theme::UIStyle;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Clear, Paragraph},
};

type Creator = fn() -> Box<dyn Window>;

pub struct SplitSelect {
    tree_state: TreeState<Creator>,
    direction: Direction,
}

impl SplitSelect {
    pub fn new(direction: Direction) -> Self {
        let registry = get_window_registry();
        let mut s: TreeState<Creator> = TreeState::new();

        for entry in registry.iter() {
            match entry {
                WindowRegistryEntry::Category { name, children } => {
                    let root = s.add_root(*name, None, NodeKind::Branch { expanded: true });
                    Self::create_window_tree(root, children, &mut s);
                },

                WindowRegistryEntry::Window { name, create } => {
                    s.add_root(*name, Some(*create), NodeKind::Branch { expanded: true });
                },
            }
        }

        Self {
            direction, 
            tree_state: s,
        }
    }

    fn create_window_tree(parent: NodeId, roots: &Vec<WindowRegistryEntry>, s: &mut TreeState<Creator>) {
        for entry in roots.iter() {
            match entry {
                WindowRegistryEntry::Category { name, children } => {
                    let root = s.add_child(parent, *name, None, NodeKind::Branch { expanded: false }).unwrap();
                    Self::create_window_tree(root, children, s);
                },

                WindowRegistryEntry::Window { name, create } => {
                    let _ = s.add_child(parent, *name, Some(*create), NodeKind::Leaf);
                },
            }
        }
    }
}

impl Window for SplitSelect {
    fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = UIStyle::window_border(" New Window ", focused);

        let list_area = UIStyle::centered_rect(50, 50, area);
        frame.render_widget(Clear, block.inner(list_area));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .margin(1)
            .split(block.inner(list_area));

        frame.render_widget(&block, list_area);
        frame.render_widget(Paragraph::new("Lorum Ipsum")
            .centered(), layout[0]);
        frame.render_stateful_widget(
            TreeWidget::new()
                .collapsed_icon("")
                .expanded_icon("")
                .leaf_icon("󰎄")
                .branch_style(Style::default().fg(UIStyle::MAIN_COLOUR))
                .leaf_style(Style::default().fg(UIStyle::ACCENT_COLOUR)),
            layout[1], &mut self.tree_state);

        // "", "", "󰉖", "󰷏",
    }

    fn handle_input(&mut self, cmd: LocalCommand) -> Option<EditorCommand> {
        match cmd {
            LocalCommand::MoveLocalCursor { dx: _, dy } => {
                let count = dy.abs() as usize;

                if dy < 0 {
                    self.tree_state.select_next(count);
                } else {
                    self.tree_state.select_prev(count);
                }

                None
            },

            LocalCommand::Confirm => {
                let node_id = match self.tree_state.selected() {
                    Some(n) => n,
                    None => return None,
                };

                self.tree_state.toggle_expand(node_id).ok();

                let node = match self.tree_state.raw_selected() {
                    Some(n) => n,
                    None => return None,
                };

                match node.data() {
                    Some(func) => {
                        Some(EditorCommand::OpenWindow {
                            display: WindowPaneType::Direction { direction: self.direction },
                            window: func(),
                        })
                    },

                    None => None
                }
            },

            _ => None,
        }
    }
}

