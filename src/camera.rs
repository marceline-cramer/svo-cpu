use glam::{Mat4, Vec3A};
use minifb::Window;
use std::cmp::min;

pub struct Camera {
    pub eye: Vec3A,
    pub fb: Framebuffer,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3A::new(3.0, 2.0, 1.0),
            fb: Framebuffer::default(),
        }
    }
}

impl Camera {
    pub fn draw_voxel(&mut self, center: &Vec3A) {
        let x = ((center.y * 0.5 + 0.5) * (self.fb.width as f32)).round() as usize;
        let y = ((center.z * 0.5 + 0.5) * (self.fb.height as f32)).round() as usize;
        self.fb.data[y * self.fb.width + x] = 0xff0000ff;
    }
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
}

impl Default for Framebuffer {
    fn default() -> Self {
        let width = 1280;
        let height = 720;
        let data = vec![0; width * height];
        Self {
            width,
            height,
            data,
        }
    }
}

impl Framebuffer {
    fn rect(&mut self, c: u32, l: usize, t: usize, r: usize, b: usize) {
        let r = min(r, self.width - 1);
        let b = min(b, self.height - 1);
        let xt = t * self.width;
        let mut xl = xt + l;
        let mut xr = xt + r;
        for row in t..b {
            self.data[xl..xr].fill(c);
            xl += self.width;
            xr += self.width;
        }
    }

    pub fn update_window(&self, window: &mut Window) {
        window
            .update_with_buffer(&self.data, self.width, self.height)
            .unwrap();
    }
}