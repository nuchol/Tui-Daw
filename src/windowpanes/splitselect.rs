use crate::{
    widgets::tree::{node::NodeKind, state::TreeState, treewiddget::TreeWidget},
    windowpanes::{
    window::Window,
    windowregistry::*,
}};

use crate::input::{EditorCommand, LocalCommand};
use crate::theme::UIStyle;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Clear, Paragraph},
};

pub struct SplitSelect {
    tree_state: TreeState<()>,
    registry: Vec<WindowRegistryEntry>,
    direction: Direction,
}

impl SplitSelect {
    pub fn new(direction: Direction) -> Self {
        let registry = get_window_registry();
        // let mut buttons = Vec::new();
        // for entry in registry.iter() {
        //     match entry {
        //         WindowRegistryEntry::Category { name, children } => {
        //
        //         },
        //
        //         WindowRegistryEntry::Window { name, create } => {
        //             buttons.push(Button {
        //                 label: Line::from(*name).centered(),
        //                 height: 1,
        //                 style: Style::default(),
        //             });
        //         },
        //     }
        // }
        let mut s: TreeState<()> = TreeState::new();
        let root_a =
            s.add_root("root_a", (), NodeKind::Branch { expanded: true });
        let child_a1 =
            s.add_child(root_a, "child_a1", (), NodeKind::Leaf).unwrap();
        let child_a2 = s
            .add_child(root_a, "child_a2", (), NodeKind::Branch { expanded: false })
            .unwrap();
        let grandchild =
            s.add_child(child_a2, "grandchild", (), NodeKind::Leaf).unwrap();
        let root_b = s.add_root("root_b", (), NodeKind::Leaf);

        Self {
            direction, 
            tree_state: s,
            registry,
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
                let node = match self.tree_state.selected() {
                    Some(n) => n,
                    None => return None,
                };

                self.tree_state.toggle_expand(node).ok();

                // let button_index = self.list_state.get_hovered();
                // if button_index.is_none() { return None; }
                //
                // Some(EditorCommand::OpenWindow {
                //     display: WindowPaneType::Direction { direction: self.direction },
                //     window: match button_index.unwrap() {
                //         0 => Box::new(PianoRollPane::new()),
                //         _ => panic!("No Window type"),
                //     }
                // })

                None
            },

            _ => None,
        }
    }
}

