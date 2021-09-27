// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use super::camera::Camera;
use glam::{Vec3A, Vec4};
use std::io::{BufRead, Read};
use std::time::Instant;

pub type ChildIndex = u8;
pub type ChildMask = u8;
pub type ChildOrder = u8;
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
                node.for_kids_all_mut(|index, _mask, child| {
                    let index = index as usize;
                    let child_ref = (cursor + index) as NodeRef;
                    let offset = offsets[index];
                    *child = child_ref;
                    stack.push_back((offset.0, offset.1, offset.2, child_ref, lod - 1));
                });
            }
        }

        println!("converted in {:?}", timer.elapsed());
        println!("{} nodes", capacity);
        println!("{} voxels", filled);
        println!("{} voxels were treed", placed_voxels);

        let mut vb = Self { nodes };
        let dummy_eye = Vec3A::new(3.0, 2.0, 1.0);
        vb.walk_all(&dummy_eye);
        vb.cull_unfilled();
        vb.walk_all(&dummy_eye);
        vb.breadth_sort_nodes();
        vb.walk_all(&dummy_eye);
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

    /// breadth-sorts nodes
    /// also removes unused nodes
    pub fn breadth_sort_nodes(&mut self) {
        let timer = Instant::now();

        let mut nodes = Vec::<Node>::new();
        let mut cursor = 1;

        let mut queue = std::collections::VecDeque::<NodeRef>::new();
        queue.push_front(Self::ROOT_NODE);
        while let Some(node_ref) = queue.pop_back() {
            let mut node = self.nodes.get(node_ref as usize).unwrap().clone();

            node.for_kids_mut(|_index, child| {
                queue.push_front(*child);
                *child = cursor as NodeRef;
                cursor += 1;
            });

            if node.data.color != 0 {
                node.data.color = node_ref;
            }

            nodes.push(node);
        }

        println!(
            "breadth-sorted {} nodes to {} nodes in {:?}",
            self.nodes.len(),
            nodes.len(),
            timer.elapsed()
        );

        self.nodes = nodes;
    }

    pub fn walk<F>(&self, eye: &Vec3A, mut on_node: F)
    where
        F: FnMut(bool, &Payload, Vec4) -> bool,
    {
        let timer = Instant::now();

        let mut walked_num = 0;
        let mut leaf_num = 0;
        let mut stack = vec![(Self::ROOT_NODE, Vec3A::new(0.0, 0.0, 0.0), 0)];

        while let Some((node_ref, stem, depth)) = stack.pop() {
            walked_num += 1;
            let node = self.nodes.get(node_ref as usize).unwrap();
            let offset = 1.0 / ((2 << depth) as f32);
            let voxel = stem.extend(offset);

            let is_leaf = node.is_leaf();
            if is_leaf {
                leaf_num += 1
            };
            if on_node(is_leaf, &node.data, voxel) & !is_leaf {
                let order = Node::sorting_order(&eye, &stem);
                node.for_kids_ordered(order, |index, child| {
                    let origin = stem + Node::index_offset(index, offset);
                    stack.push((*child, origin, depth + 1));
                });
            }
        }

        println!(
            "walked {} nodes ({} leaves) in {:?}",
            walked_num,
            leaf_num,
            timer.elapsed()
        );
    }

    pub fn walk_all(&self, eye: &Vec3A) -> Vec<(Payload, Vec4)> {
        let mut nodes = Vec::<(Payload, Vec4)>::new();
        self.walk(eye, |is_leaf, data, voxel| {
            if is_leaf {
                nodes.push((*data, voxel));
            }
            true
        });
        nodes
    }

    pub fn draw(&self, camera: &mut Camera) {
        let timer = Instant::now();

        self.walk(&camera.eye.clone(), |is_leaf, data, voxel| {
            if is_leaf {
                camera.draw_voxel(&voxel, data.color);
                true
            } else {
                camera.test_voxel(&voxel)
            }
        });

        println!("done drawing in {:?}", timer.elapsed());
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

const HILBERT_ORDER: [(ChildIndex, ChildMask); 8] = [
    (0, 0x01),
    (1, 0x02),
    (3, 0x08),
    (2, 0x04),
    (6, 0x40),
    (7, 0x80),
    (5, 0x20),
    (4, 0x10),
];

const SORTED_ORDER_INDICES: [[ChildIndex; 8]; 48] = [[0, 1, 2, 3, 4, 5, 6, 7]; 48];

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

    pub fn for_kids_all<F>(&self, mut f: F)
    where
        F: FnMut(ChildIndex, ChildMask),
    {
        for (index, mask) in HILBERT_ORDER.iter() {
            f(*index, *mask);
        }
    }

    pub fn for_kids_all_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(ChildIndex, ChildMask, &mut NodeRef),
    {
        for (index, mask) in HILBERT_ORDER.iter() {
            f(*index, *mask, &mut self.children[*index as usize]);
        }
    }

    pub fn for_kids<F>(&self, mut f: F)
    where
        F: FnMut(ChildIndex, &NodeRef),
    {
        let occupancy = self.occupancy.clone();
        if !self.is_leaf() {
            for (index, mask) in HILBERT_ORDER.iter() {
                if (occupancy & mask) != 0 {
                    f(*index, &self.children[*index as usize]);
                }
            }
        }
    }

    pub fn for_kids_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(ChildIndex, &mut NodeRef),
    {
        let occupancy = self.occupancy.clone();
        if !self.is_leaf() {
            for (index, mask) in HILBERT_ORDER.iter() {
                if (occupancy & mask) != 0 {
                    f(*index, &mut self.children[*index as usize]);
                }
            }
        }
    }

    pub fn for_kids_ordered<F>(&self, order: ChildOrder, mut f: F)
    where
        F: FnMut(ChildIndex, &NodeRef),
    {
        if !self.is_leaf() {
            let occupancy = self.occupancy.clone();
            let indices = SORTED_ORDER_INDICES[order as usize];
            for index in indices.iter() {
                // TODO: statically cache this
                let mask = Self::index_to_mask(*index);
                if (occupancy & mask) != 0 {
                    f(*index, &self.children[*index as usize]);
                }
            }
        }
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

    /// based on: https://iquilezles.org/www/articles/volumesort/volumesort.htm
    pub fn sorting_order(eye: &Vec3A, stem: &Vec3A) -> ChildOrder {
        let s = eye.cmpgt(*stem).bitmask();
        let sx = (s & 0b001) as ChildOrder;
        let sy = ((s & 0b010) >> 1) as ChildOrder;
        let sz = ((s & 0b100) >> 2) as ChildOrder;
        let a = (*stem - *eye).abs();

        if a.x > a.y && a.x > a.z {
            if a.y > a.z {
                (sx << 2) | (sy << 1) | sz
            } else {
                8 + ((sx << 2) | (sz << 1) | sy)
            }
        } else if a.y > a.z {
            if a.x > a.z {
                16 + ((sy << 2) | (sx << 1) | sz)
            } else {
                24 + ((sy << 2) | (sz << 1) | sx)
            }
        } else {
            if a.x > a.y {
                32 + ((sz << 2) | (sx << 1) | sy)
            } else {
                40 + ((sz << 2) | (sy << 1) | sx)
            }
        }
    }
}
