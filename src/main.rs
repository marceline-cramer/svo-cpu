// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use argh::FromArgs;
use minifb::{Key, Window, WindowOptions};

use svo_cpu::binvox::import_binvox_svo as import_svo;
use svo_cpu::camera::SpinnyCamera as Camera;
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
        _ => Err("invalid model (must be one of [bunny, dragon, buddha])".into()),
    }
}

fn main() {
    let args: Args = argh::from_env();
    let vb = args.model;

    let mut cam = Camera::default();
    vb.draw(&mut cam);

    let mut window = Window::new(
        "Test - ESC to exit",
        cam.fb.width,
        cam.fb.height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cam.update();
        cam.fb.clear();
        vb.draw(&mut cam);
        cam.fb.update_window(&mut window);
    }
}
