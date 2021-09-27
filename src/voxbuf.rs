// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use super::camera::Camera;
use glam::{Vec3A, Vec4};
use std::convert::TryInto;
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

    pub fn from_nodes(nodes: Vec<Node>) -> Self {
        Self { nodes }
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
                let color = (node_ref << 2) & 0xffffff;
                /*const R_MASK: u32 = 0xff0000;
                const G_MASK: u32 = 0x00ff00;
                const B_MASK: u32 = 0x0000ff;
                let color = (((color & R_MASK) >> 1) & R_MASK)
                    | (((color & G_MASK) >> 1) & G_MASK)
                    | (((color & B_MASK) >> 1) & B_MASK);*/
                node.data.color = 0xff000000 | color;
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
        let origin = Vec3A::new(0.0, 0.0, 0.0);
        let mut stack = vec![(Self::ROOT_NODE, origin, 0)];

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
    pub color: u32,
}

impl Default for Payload {
    fn default() -> Self {
        Self { color: 0xff0000ff }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub occupancy: ChildMask,
    pub children: [NodeRef; 8],
    pub data: Payload,
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
            for (index, mask) in indices.iter() {
                if (occupancy & *mask) != 0 {
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
        let s = eye.cmpge(*stem).bitmask();
        let sx = (s & 0b001) as ChildOrder;
        let sy = ((s & 0b010) >> 1) as ChildOrder;
        let sz = ((s & 0b100) >> 2) as ChildOrder;
        let a = (*eye - *stem).abs();

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

fn calc_child_orders_sub(x: u8, y: u8, z: u8) -> [ChildIndex; 8] {
    let mut orders = [0 as u8; 8];
    let masks = (1 << x, 1 << y, 1 << z);
    for i in 0..8 {
        let mut c: u8 = 0x07;

        if (i & masks.0) != 0 {
            c = 0b110;
        }

        if (i & masks.1) != 0 {
            c &= 0b101;
        }

        if (i & masks.2) != 0 {
            c &= 0b011;
        }

        orders[i] = c;
    }
    orders
}

fn calc_child_orders_flip(x: u8, y: u8, z: u8) -> [[(ChildIndex, ChildMask); 8]; 8] {
    let mut orders = [[(0 as u8, 0 as u8); 8]; 8];
    let base = calc_child_orders_sub(x, y, z);
    for i in 0..8 {
        let order = &mut orders[i];
        for j in 0..8 {
            let index = j as u8 ^ base[i];
            order[j] = (index, Node::index_to_mask(index));
        }
    }
    orders
}

fn calc_child_orders() -> [[(ChildIndex, ChildMask); 8]; 48] {
    [
        calc_child_orders_flip(2, 1, 0),
        calc_child_orders_flip(2, 0, 1),
        calc_child_orders_flip(1, 2, 0),
        calc_child_orders_flip(0, 2, 1),
        calc_child_orders_flip(1, 0, 2),
        calc_child_orders_flip(0, 1, 2),
    ]
    .concat()
    .try_into()
    .unwrap()
}

lazy_static! {
    static ref SORTED_ORDER_INDICES: [[(ChildIndex, ChildMask); 8]; 48] = {
        let orders = calc_child_orders();
        orders
    };
}
