use minifb::{Key, Window, WindowOptions};

mod voxbuf;

const WIDTH: usize = 240;
const HEIGHT: usize = 180;

fn main() {
    let vb = voxbuf::VoxBuf::new_dummy();
    println!("walk results: {:#?}", vb.walk(&glam::Vec3A::new(3.0, 2.0, 1.0)));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
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
        for i in buffer.iter_mut() {
            *i = 0xff0000ff;
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
