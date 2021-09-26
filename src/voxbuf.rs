use super::camera::Camera;
use glam::Vec3A;
use std::io::{BufRead, Read};
use std::time::Instant;

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
            nodes: vec![Node::default()],
        }
    }

    pub fn new_dummy() -> Self {
        let root_node = Node {
            occupancy: 0b00000001,
            children: [1, 0, 0, 0, 0, 0, 0, 0],
            data: Payload::default(),
        };

        let leaf_node = Node::default();

        Self {
            nodes: vec![root_node, leaf_node],
        }
    }

    pub fn from_binvox(bv: &[u8]) -> Self {
        let timer = Instant::now();

        let mut reader = std::io::BufReader::new(bv);

        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        if !header.starts_with("#binvox 1") {
            panic!("data is not a binvox");
        }

        let mut dim = String::new();
        reader.read_line(&mut dim).unwrap();
        if !dim.starts_with("dim 256 256 256") {
            unimplemented!("unsupported dimension");
        }

        let dim = 256;
        let dim2 = dim * dim;

        let mut junk = String::new();
        reader.read_line(&mut junk).unwrap(); // translate
        reader.read_line(&mut junk).unwrap(); // scale
        reader.read_line(&mut junk).unwrap(); // data

        let mut rle_data = Vec::<u8>::new();
        let rle_size = reader.read_to_end(&mut rle_data).unwrap() / 2;

        let capacity = dim * dim * dim;
        let mut data = vec![0 as u8; capacity];

        let mut filled: usize = 0;
        let mut cur = 0;
        for pair in 0..rle_size {
            let value = rle_data[pair << 1];
            let count = rle_data[pair << 1 | 1] as usize;
            if value != 0 {
                filled += count;
            }
            if cur + count > capacity {
                panic!("voxel data RLE overflow");
            }
            for _ in 0..count {
                data[cur] = value;
                cur += 1;
            }
        }

        let mut nodes = vec![Node::default()];

        let mut stack = std::collections::VecDeque::<(
            u16,     // 0: x
            u16,     // 1: y
            u16,     // 2: z
            NodeRef, // 3: node
            u8,      // 4: lod
        )>::new();

        stack.push_front((0, 0, 0, 0, 7));

        let mut placed_voxels = 0;

        while let Some(iter) = stack.pop_back() {
            let parent = iter.3 as usize;
            let lod = iter.4;

            let cursor = nodes.len();
            nodes.resize_with(cursor + 8, Default::default);
            let node = nodes.get_mut(parent).unwrap();

            let mut offsets: [(u16, u16, u16); 8] = [(0, 0, 0); 8];
            for i in 0..8 {
                offsets[i] = (
                    iter.0 | ((i & 1) << lod) as u16,
                    iter.1 | (((i & 2) >> 1) << lod) as u16,
                    iter.2 | (((i & 4) >> 2) << lod) as u16,
                );
            }

            if lod == 0 {
                node.occupancy = 0x00;
                for i in 0..8 {
                    let offset = offsets[i];
                    let index =
                        (offset.2 as usize * dim2) + (offset.1 as usize * dim) + offset.0 as usize;
                    if data[index] != 0 {
                        node.occupancy |= Node::index_to_mask(i as ChildMask);
                        node.children[i] = (cursor + i) as NodeRef;
                        placed_voxels += 1;
                    }
                }

                if node.occupancy == 0x00 {
                    node.data.color = 0;
                }
            } else {
                node.data.color = 0xff00ffff;
                node.occupancy = 0xff;
                for i in 0..8 {
                    let child = (cursor + i) as NodeRef;
                    let offset = offsets[i];
                    node.children[i] = child;
                    stack.push_back((offset.0, offset.1, offset.2, child, lod - 1));
                }
            }
        }

        println!("converted in {:?}", timer.elapsed());
        println!("{} nodes", capacity);
        println!("{} voxels", filled);
        println!("{} voxels were treed", placed_voxels);

        let mut vb = Self { nodes };
        let dummy_eye = Vec3A::new(3.0, 2.0, 1.0);
        vb.walk(&dummy_eye);
        vb.cull_unfilled();
        // vb.walk(&dummy_eye);
        // vb.sort_nodes();
        vb.walk(&dummy_eye);
        vb
    }

    pub fn cull_unfilled(&mut self) {
        let timer = Instant::now();
        self.cull_unfilled_children(Self::ROOT_NODE);
        println!("culled unfilled branches in {:?}", timer.elapsed());
    }

    fn cull_unfilled_children(&mut self, parent_ref: NodeRef) -> bool {
        let mut parent = *self.nodes.get(parent_ref as usize).unwrap();

        if parent.is_leaf() {
            if parent.data.color == 0 {
                return false;
            }
        } else {
            for index in 0..8 {
                let mask = Node::index_to_mask(index);
                if parent.is_occupied(mask) {
                    let child_index = parent.children[index as usize];
                    if !self.cull_unfilled_children(child_index) {
                        let mask = !Node::index_to_mask(index);
                        parent.occupancy &= mask;
                    }
                }
            }

            if parent.is_leaf() {
                return false;
            }

            self.nodes[parent_ref as usize] = parent;
        }

        true
    }

    /// depth-sorts nodes
    /// also removes unused nodes
    pub fn sort_nodes(&mut self) {
        let timer = Instant::now();

        let mut nodes = Vec::<Node>::new();
        let mut cursor = 1;

        let mut stack = vec![Self::ROOT_NODE];
        while let Some(node_ref) = stack.pop() {
            let mut node = self.nodes.get(node_ref as usize).unwrap().clone();

            if !node.is_leaf() {
                for index in 0..8 {
                    let mask = Node::index_to_mask(index);
                    if node.is_occupied(mask) {
                        stack.push(node.children[index as usize]);
                        node.children[index as usize] = cursor;
                        cursor += 1;
                    }
                }
            }

            nodes.push(node);
        }

        println!(
            "depth-sorted {} nodes to {} nodes in {:?}",
            self.nodes.len(),
            nodes.len(),
            timer.elapsed()
        );

        self.nodes = nodes;
    }

    pub fn walk(&self, eye: &Vec3A) -> Vec<(Payload, Vec3A)> {
        let timer = Instant::now();

        let mut walked_num = 0;
        let mut nodes = Vec::<(Payload, Vec3A)>::new();
        let mut stack = vec![(Self::ROOT_NODE, Vec3A::new(0.0, 0.0, 0.0), 0)];

        while let Some((node_ref, stem, depth)) = stack.pop() {
            walked_num += 1;
            let node = self.nodes.get(node_ref as usize).unwrap();
            if node.is_leaf() {
                nodes.push((node.data, stem.into()));
            } else {
                let order = Node::sorting_order(eye);
                let offset = 1.0 / ((2 << depth) as f32);
                for index in order.iter() {
                    let mask = Node::index_to_mask(*index);
                    if node.is_occupied(mask) {
                        let child = node.get_child(*index);
                        let origin = stem + Node::index_offset(*index, offset);
                        stack.push((child, origin, depth + 1));
                    }
                }
            }
        }

        println!("walked {} nodes in {:?}", walked_num, timer.elapsed());

        nodes
    }

    pub fn draw(&self, camera: &mut Camera) {
        let walked = self.walk(&camera.eye);
        let timer = Instant::now();
        let leaf_num = walked.len();
        for (voxel, center) in walked.iter() {
            // println!("voxel: {:#?}", voxel);
            camera.draw_voxel(&center, voxel.color);
        }
        println!("done drawing {} leaves in {:?}", leaf_num, timer.elapsed());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Payload {
    color: u32,
}

impl Default for Payload {
    fn default() -> Self {
        Self { color: 0xff0000ff }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    occupancy: ChildMask,
    children: [NodeRef; 8],
    data: Payload,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            occupancy: 0,
            children: [0; 8],
            data: Payload::default(),
        }
    }
}

impl Node {
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

    pub fn index_offset(index: ChildIndex, offset: f32) -> Vec3A {
        match index {
            0 => [-offset, -offset, -offset],
            1 => [offset, -offset, -offset],
            2 => [-offset, offset, -offset],
            3 => [offset, offset, -offset],
            4 => [-offset, -offset, offset],
            5 => [offset, -offset, offset],
            6 => [-offset, offset, offset],
            7 => [offset, offset, offset],
            _ => panic!("invalid child index"),
        }
        .into()
    }

    pub fn index_origin(index: ChildIndex) -> Vec3A {
        Self::index_offset(index, 0.5)
    }

    /// TODO: find literature on this
    pub fn sorting_order(eye: &Vec3A) -> [ChildIndex; 8] {
        [0, 1, 2, 3, 4, 5, 6, 7]
    }
}
