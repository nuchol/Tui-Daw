
// We use a dedicated opaque type to controll node construction
// NodeIds can only be constructed within the tree module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) usize);
impl NodeId {
    // Returns the raw numeric value of this id
    pub fn raw(self) -> usize {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Leaf,
    Branch { expanded: bool },
}

impl NodeKind {
    pub fn is_branch(&self) -> bool {
        matches!(self, NodeKind::Branch { .. })
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, NodeKind::Leaf)
    }

    pub fn is_expanded(&self) -> bool {
        matches!(self, NodeKind::Branch { expanded: true })
    }

    pub fn is_collapsed(&self) -> bool {
        matches!(self, NodeKind::Branch { expanded: false })
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    pub(super) id: NodeId,
    pub(super) label: String,
    pub(super) kind: NodeKind,
    pub(super) parent: Option<NodeId>,

    // Always empty for leaf nodes
    pub(super) children: Vec<NodeId>,

    // The caller's domain data associated with this node
    pub(super) data: T,
}

impl<T> TreeNode<T> {
    pub fn id(&self) -> NodeId {
        self.id
    }
 
    pub fn label(&self) -> &str {
        &self.label
    }
 
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
 
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }
 
    pub fn data(&self) -> &T {
        &self.data
    }
 
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
 
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }
 
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
 
    pub fn is_childless(&self) -> bool {
        self.children.is_empty()
    }
 
    pub(super) fn set_label(&mut self, new_label: impl Into<String>) -> String {
        let old = std::mem::replace(&mut self.label, new_label.into());
        old
    }
}
