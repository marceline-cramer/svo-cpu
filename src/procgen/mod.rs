pub mod terrain;

pub use crate::voxbuf::{ChildIndex, Node, NodeRef, VoxBuf};
pub use glam::Vec3A;
use std::time::Instant;

pub trait ProcGen {
    fn is_occupied(&self, pos: &Vec3A) -> bool;
}

pub fn generate_voxbuf<T>(procgen: T) -> VoxBuf
where
    T: ProcGen,
{
    let timer = Instant::now();

    const MAX_DEPTH: u32 = 4;

    let mut nodes = vec![Node::default()];
    let mut filled = 0;
    let origin = Vec3A::new(0.0, 0.0, 0.0);
    let mut stack = vec![(VoxBuf::ROOT_NODE, origin, 0 as u32)];

    while let Some((node_ref, stem, depth)) = stack.pop() {
        let cursor = nodes.len();
        nodes.resize_with(cursor + 8, Default::default);
        let node = nodes.get_mut(node_ref as usize).unwrap();
        let offset = VoxBuf::depth_to_offset(depth);

        if depth >= MAX_DEPTH {
            node.occupancy = 0x00;
            for i in 0..8 {
                if procgen.is_occupied(&stem) {
                    node.occupancy |= Node::index_to_mask(i as ChildIndex);
                    node.children[i] = (cursor + i) as NodeRef;
                    filled += 1;
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
                let origin = stem + Node::index_offset(index as ChildIndex, offset);
                *child = child_ref;
                stack.push((child_ref, origin, depth + 1));
            });
        }
    }

    println!("generated in {:?}", timer.elapsed());
    println!("{} voxels", filled);

    VoxBuf::from_nodes(nodes)
}
