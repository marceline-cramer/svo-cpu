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
                    node.data.color = 0xff0000ff;
                }
            } else {
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

        Self { nodes }
    }

    pub fn walk(&self, eye: &Vec3A) -> Vec<(NodeRef, Vec3A)> {
        let timer = Instant::now();

        let mut nodes = Vec::<(NodeRef, Vec3A)>::new();
        let mut stack = vec![(Self::ROOT_NODE, Vec3A::new(0.0, 0.0, 0.0), 0)];

        while let Some((node_ref, stem, depth)) = stack.pop() {
            let node = self.nodes.get(node_ref as usize).unwrap();
            if node.is_leaf() {
                nodes.push((node_ref, stem.into()));
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

        println!("walked in {:?}", timer.elapsed());

        nodes
    }

    pub fn draw(&self, camera: &mut Camera) {
        let timer = Instant::now();
        let walked = self.walk(&camera.eye);
        for (node_ref, voxel) in walked.iter() {
            let node = self.nodes.get(*node_ref as usize).unwrap();
            camera.draw_voxel(&voxel, node.data.color);
        }
        println!("done drawing in {:?}", timer.elapsed());
    }
}

#[derive(Clone, Debug)]
pub struct Payload {
    color: u32,
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            color: 0xff0000ff,
        }
    }
}

#[derive(Clone, Debug)]
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
