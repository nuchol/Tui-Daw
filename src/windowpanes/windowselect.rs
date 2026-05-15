use crate::{
    theme::{ResolvedTheme, ThemeKey},
    widgets::tree::{
        node::{NodeId, NodeKind},
        state::TreeState,
        treewiddget::TreeWidget,
    },
    windowpanes::{
        window::{Window, WindowPaneType},
        windowregistry::*,
    }
};

use crate::input::{EditorCommand, LocalCommand};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
};

type Creator = fn() -> Box<dyn Window>;

pub struct WindowSelect {
    tree_state: TreeState<Creator>,
    pane_type: WindowPaneType,
}

impl WindowSelect {
    pub fn new(pane_type: WindowPaneType) -> Self {
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
            tree_state: s,
            pane_type, 
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

impl Window for WindowSelect {
    fn title(&self) -> &str {
        " New Window "
    }
    fn render(&mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        theme: &ResolvedTheme
    ) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .margin(1)
            .split(area);

        frame.render_widget(Paragraph::new("Lorum Ipsum")
            .centered(), layout[0]);
        frame.render_stateful_widget(
            TreeWidget::new(theme)
                .collapsed_icon("")
                .expanded_icon("")
                .leaf_icon("󰎄")
                .branch_style(theme.get(ThemeKey::FileTreeDir))
                .leaf_style(theme.get(ThemeKey::FileTreeWindow)),
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
                let node_id = self.tree_state.selected()?;
                self.tree_state.toggle_expand(node_id).ok();

                self.tree_state.raw_selected()
                    .and_then(|node| *node.data())
                    .map(|func| EditorCommand::OpenWindow {
                        display: self.pane_type,
                        window: func(),
                    })
            },

            _ => None,
        }
    }
}

