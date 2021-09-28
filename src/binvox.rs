// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use super::voxbuf::*;
use glam::Vec3A;
use std::io::{BufRead, Read};
use std::time::Instant;

pub fn import_binvox_svo(bv: &[u8]) -> VoxBuf {
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

    let mut vb = VoxBuf::from_nodes(nodes);
    let dummy_eye = Vec3A::new(3.0, 2.0, 1.0);

    print!("unprocessed:\n  ");
    vb.walk_all(&dummy_eye);

    print!("after culling unfilled:\n  ");
    vb.cull_unfilled();
    vb.walk_all(&dummy_eye);

    print!("after breadth-sorting:\n  ");
    vb.breadth_sort_nodes();
    vb.walk_all(&dummy_eye);

    print!("after depth-sorting:\n  ");
    vb.depth_sort_nodes();
    vb.walk_all(&dummy_eye);

    vb
}
