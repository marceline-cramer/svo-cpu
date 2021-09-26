use minifb::{Key, Window, WindowOptions};

mod camera;
mod voxbuf;

const WIDTH: usize = 240;
const HEIGHT: usize = 180;

fn main() {
    let vb = voxbuf::VoxBuf::from_binvox(include_bytes!("stanford_bunny.binvox"));

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
        cam.fb.update_window(&mut window);
        vb.draw(&mut cam);
    }
}
