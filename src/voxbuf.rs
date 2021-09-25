use glam::Vec3A;

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

impl Node {
    pub fn index_to_mask(index: ChildIndex) -> ChildMask {
        if index < 8 {
            1 << index
        } else {
            0
        }
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

    pub fn sorting_order(eye: &Vec3A) -> [ChildIndex; 8] {
        /*if eye.x > 0.0 {
            if eye.y > 0.0 {
                if eye.z > 0.0 {
                } else {
                }
            } else {
                if eye.z > 0.0 {
                } else {
            }
        } else {
            if eye.y > 0.0 {
                if eye.z > 0.0 {
                } else {
                }
            } else {
                if eye.z > 0.0 {
                } else {
            }
        }*/
        [0, 1, 2, 3, 4, 5, 6, 7]
    }
}
