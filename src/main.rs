// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use minifb::{Key, Window, WindowOptions};

#[macro_use]
extern crate lazy_static;

mod binvox;
mod camera;
mod fb;
mod voxbuf;

const WIDTH: usize = 240;
const HEIGHT: usize = 180;

fn main() {
    let vb = binvox::import_binvox_svo(include_bytes!("stanford_bunny.binvox"));
    // let vb = voxbuf::VoxBuf::new_dummy();

    let mut cam = camera::Camera::default();
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
