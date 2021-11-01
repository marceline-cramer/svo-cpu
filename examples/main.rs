// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use argh::FromArgs;
use minifb::{Key, Window, WindowOptions};

use svo_cpu::binvox::import_binvox_svo as import_svo;
use svo_cpu::camera::spinny_camera::SpinnyCamera;
use svo_cpu::fb::ColorBuffer;
use svo_cpu::procgen::generate_voxbuf;
use svo_cpu::procgen::terrain::TerrainGen;
use svo_cpu::voxbuf::VoxBuf;

#[derive(FromArgs)]
/// CPU-based sparse voxel octree (SVO) rasterizer
struct Args {
    /// model to draw (defaults to "dragon")
    #[argh(option, default = "default_model()", from_str_fn(select_model))]
    model: VoxBuf,
}

fn default_model() -> VoxBuf {
    import_svo(include_bytes!("models/stanford_bunny.binvox"))
}

fn select_model(option: &str) -> Result<VoxBuf, String> {
    match option {
        "bunny" => Ok(default_model()),
        "dragon" => Ok(import_svo(include_bytes!("models/stanford_dragon.binvox"))),
        "buddha" => Ok(import_svo(include_bytes!("models/stanford_buddha.binvox"))),
        "terrain" => Ok(generate_voxbuf(TerrainGen::default())),
        _ => Err("invalid model (must be one of [bunny, dragon, buddha, terrain])".into()),
    }
}

fn main() {
    let args: Args = argh::from_env();
    let vb = args.model;

    let mut fb = ColorBuffer::default();
    let mut spinny_cam = SpinnyCamera::new(&fb);
    vb.draw(&spinny_cam.camera, &spinny_cam.draw_config, &mut fb);

    let mut window = Window::new(
        "Test - ESC to exit",
        fb.width,
        fb.height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        spinny_cam.update(&fb);
        fb.clear();
        vb.draw(&spinny_cam.camera, &spinny_cam.draw_config, &mut fb);
        window
            .update_with_buffer(&fb.data, fb.width, fb.height)
            .unwrap();
    }
}
