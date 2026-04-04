use super::flatten::flatten_visible;
use super::node::{NodeId, NodeKind, TreeNode};

use std::collections::HashMap;
use std::io::{Error, ErrorKind};

pub struct TreeState<T> {
    pub(super) nodes: HashMap<NodeId, TreeNode<T>>,
    pub(super) roots: Vec<NodeId>,
    pub(super) selected: Option<NodeId>,
    pub(super) next_id: usize,
}

fn error_not_found(id: NodeId) -> Error {
    Error::new(ErrorKind::NotFound, format!("No node id: {}", id.raw()))
}

fn error_not_dir(id: NodeId) -> Error {
    Error::new(ErrorKind::NotADirectory, format!("Node {} is not a branch", id.raw()))
}

impl<T> TreeState<T> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            roots: Vec::new(),
            selected: None,
            next_id: 0,
        }
    }

    // Mint a new NodeId, this is the only place that this can be done.
    fn new_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        id
    }

    pub fn add_root(
        &mut self,
        label: impl Into<String>,
        data: T,
        kind: NodeKind
    ) -> NodeId {
        let id = self.new_id();
        let node = TreeNode {
            id,
            label: label.into(),
            kind,
            children: Vec::new(),
            data,
            parent: None,
        };

        self.nodes.insert(id, node);
        self.roots.push(id);

        id
    }

    pub fn add_child(
        &mut self,
        parent_id: NodeId,
        label: impl Into<String>,
        data: T,
        kind: NodeKind,
    ) -> Result<NodeId, Error> {
        match self.nodes.get(&parent_id) {
            None => return Err(error_not_found(parent_id)),
            Some(parent) if parent.kind.is_leaf() => {
                return Err(error_not_dir(parent_id));
            }
            Some(_) => {}
        }

        let id = self.new_id();
        let node = TreeNode {
            id,
            label: label.into(),
            kind,
            children: Vec::new(),
            data,
            parent: Some(parent_id),
        };

        self.nodes.insert(id, node);
        self.nodes.get_mut(&parent_id).unwrap().children.push(id);

        Ok(id)
    }

    pub fn remove_node(&mut self, id: NodeId) -> std::io::Result<()> {
        if !self.nodes.contains_key(&id) {
            return Err(error_not_found(id));
        }

        // Collect the full subtree rooted at `id` so we can remove every node
        // in a single pass after the traversal.
        let mut to_remove: Vec<NodeId> = Vec::new();
        self.collect_subtree(id, &mut to_remove);

        // Clear selection if it falls inside the subtree being removed.
        if let Some(sel) = self.selected {
            if to_remove.contains(&sel) {
                self.selected = None;
            }
        }

        // Unlink `id` from its parent's children list or from the root list.
        let parent_id = self.nodes[&id].parent;
        match parent_id {
            Some(pid) => {
                if let Some(parent) = self.nodes.get_mut(&pid) {
                    parent.children.retain(|&c| c != id);
                }
            }
            None => {
                self.roots.retain(|&r| r != id);
            }
        }

        // Drop every node in the subtree.
        for node_id in to_remove {
            self.nodes.remove(&node_id);
        }

        Ok(())
    }

    fn collect_subtree(&self, root: NodeId, out: &mut Vec<NodeId>) {
        out.push(root);

        if let Some(node) = self.nodes.get(&root) {
            for &child in &node.children {
                self.collect_subtree(child, out);
            }
        }
    }

    pub fn expand(&mut self, id: NodeId) -> Result<(), Error> {
        let node = self.nodes.get_mut(&id).ok_or(error_not_found(id))?;

        if let NodeKind::Branch { ref mut expanded } = node.kind {
            *expanded = true;
        }

        Ok(())
    }

    pub fn collapse(&mut self, id: NodeId) -> Result<(), Error> {
        let node = self.nodes.get_mut(&id).ok_or(error_not_found(id))?;

        if let NodeKind::Branch { ref mut expanded } = node.kind {
            *expanded = false;
        }

        Ok(())
    }

    pub fn toggle_expand(&mut self, id: NodeId) -> Result<(), Error> {
        let node = self.nodes.get_mut(&id).ok_or(error_not_found(id))?;

        if let NodeKind::Branch { ref mut expanded } = node.kind {
            *expanded = !(*expanded);
        }

        Ok(())
    }


    pub fn select_next(&mut self, count: usize) {
        let flat = flatten_visible(&self);
        if flat.is_empty() {
            return;
        }

        let next_index = match self.selected {
            None => 0,
            Some(sel) => {
                match flat.iter().position(|x| x.id == sel) {
                    Some(pos) => (pos + count).min(flat.len() - 1),

                    // Current selection is not visible then go to first item.
                    None => 0,
                }
            }
        };

        self.selected = Some(flat[next_index].id);
    }
 
    pub fn select_prev(&mut self, count: usize) {
        let flat = flatten_visible(&self);
        if flat.is_empty() {
            return;
        }

        let prev_index = match self.selected {
            None => flat.len() - 1,
            Some(sel) => {
                match flat.iter().position(|x| x.id == sel) {
                    Some(pos) => pos.saturating_sub(count),
                    None => flat.len() - 1,
                }
            }
        };

        self.selected = Some(flat[prev_index].id);
    }

    pub fn select_node(&mut self, id: NodeId) -> Result<(), Error> {
        if !self.nodes.contains_key(&id) {
            return Err(error_not_found(id));
        }

        // Ensure that the node is visible
        let mut cursor = self.nodes[&id].parent;
        while let Some(ancestor_id) = cursor {
            let _ = self.expand(ancestor_id);
            cursor = self.nodes[&ancestor_id].parent;
        }

        self.selected = Some(id);
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn selected(&self) -> Option<NodeId> {
        self.selected
    }
}
