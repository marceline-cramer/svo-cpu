use glam::Vec3A;

pub type ChildIndex = u8;
pub type ChildMask = u8;
pub type NodeRef = u32;

pub const INVALID_NODE: NodeRef = NodeRef::MAX;

pub struct VoxBuf {
    nodes: Vec<Node>,
}

impl VoxBuf {
    pub const ROOT_NODE: NodeRef = 0;

    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new_empty()],
        }
    }

    pub fn new_dummy() -> Self {
        let root_node = Node {
            occupancy: 0b00000001,
            children: [1, 0, 0, 0, 0, 0, 0, 0],
            data: Payload,
        };

        let leaf_node = Node::new_empty();

        Self {
            nodes: vec![root_node, leaf_node],
        }
    }

    pub fn walk(&self, eye: &Vec3A) -> Vec<NodeRef> {
        let mut nodes = Vec::<NodeRef>::new();
        // TODO: calculate child origins with descent
        let mut stack = vec![Self::ROOT_NODE];

        while let Some(node_ref) = stack.pop() {
            let node = self.nodes.get(node_ref as usize).unwrap();
            if node.is_leaf() {
                nodes.push(node_ref);
            } else {
                let order = Node::sorting_order(eye);
                for index in order.iter() {
                    let mask = Node::index_to_mask(*index);
                    if node.is_occupied(mask) {
                        let child = node.get_child(*index);
                        stack.push(child);
                    }
                }
            }
        }

        nodes
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

impl Node {
    pub fn new_empty() -> Self {
        Self {
            occupancy: 0,
            children: [0; 8],
            data: Payload,
        }
    }

    pub fn get_child(&self, index: ChildIndex) -> NodeRef {
        self.children[index as usize]
    }

    pub fn index_to_mask(index: ChildIndex) -> ChildMask {
        if index < 8 {
            1 << index
        } else {
            0
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.occupancy == 0
    }

    pub fn is_occupied(&self, mask: ChildMask) -> bool {
        (self.occupancy & mask) != 0
    }

    pub fn index_origin(index: ChildIndex) -> Vec3A {
        match index {
            0 => [-0.5, -0.5, -0.5],
            1 => [0.5, -0.5, -0.5],
            2 => [-0.5, 0.5, -0.5],
            3 => [0.5, 0.5, -0.5],
            4 => [-0.5, -0.5, 0.5],
            5 => [0.5, -0.5, 0.5],
            6 => [-0.5, 0.5, 0.5],
            7 => [0.5, 0.5, 0.5],
            _ => panic!("invalid child index"),
        }
        .into()
    }

    /// TODO: find literature on this
    pub fn sorting_order(eye: &Vec3A) -> [ChildIndex; 8] {
        [0, 1, 2, 3, 4, 5, 6, 7]
    }
}
