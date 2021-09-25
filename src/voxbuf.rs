pub type ChildIndex = u8;
pub type ChildMask = u8;
pub type NodeRef = u32;

pub const INVALID_NODE: NodeRef = NodeRef::MAX;

pub struct VoxBuf {
    nodes: Vec<Node>,
}

impl VoxBuf {
    fn get_root(&self) -> Option<&Node> {
        self.nodes.get(0)
    }
}

#[derive(Clone, Debug)]
pub struct Payload;

#[derive(Clone, Debug)]
pub struct Node {
    occupancy: ChildMask,
    children: [NodeRef; 8],
    data: Payload,
}
