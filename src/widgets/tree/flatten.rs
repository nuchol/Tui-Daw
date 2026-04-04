use crate::widgets::tree::{node::NodeId, state::TreeState};

use super::node::{TreeNode, NodeKind};

pub struct FlatNode {
    pub id: NodeId,
    pub depth: usize,
    pub is_last: bool,
}

pub fn flatten_visible<T>(state: &TreeState<T>) -> Vec<FlatNode> {
    let mut result = Vec::new();

    let len = state.roots.len();
    for (i, id) in state.roots.iter().enumerate() {
        let mut out = Vec::new();
        flatten_subtree(state, *id, 0, i == len - 1, &mut out);
        result.append(&mut out);
    }

    result
}

fn flatten_subtree<T>(
    state: &TreeState<T>,
    id: NodeId,
    depth: usize,
    is_last: bool,
    out: &mut Vec<FlatNode>,
) {
    let node = match state.nodes.get(&id) {
        Some(n) => n,
        None => return,
    };

    out.push(FlatNode { id, depth, is_last });

    if node.kind().is_expanded() {
        let len = node.children().len();
        for (i, child_id) in node.children().iter().enumerate() {
            flatten_subtree(state, *child_id, depth + 1, i == len - 1, out);
        }
    }
}
