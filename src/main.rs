use minifb::{Key, Window, WindowOptions};

mod camera;
mod voxbuf;

const WIDTH: usize = 240;
const HEIGHT: usize = 180;

fn main() {
    let vb = voxbuf::VoxBuf::from_binvox(include_bytes!("stanford_bunny.binvox"));
    let walk = vb.walk(&glam::Vec3A::new(3.0, 2.0, 1.0));
    // println!("walk results: {:?}", walk);

    let mut cam = camera::Camera::default();

    let mut window = Window::new(
        "Test - ESC to exit",
        cam.fb.width,
        cam.fb.height,
        WindowOptions {
            scale: minifb::Scale::X4,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cam.fb.update_window(&mut window);
    }
}
