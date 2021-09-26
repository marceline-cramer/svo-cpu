use glam::{Mat4, Vec3, Vec3A};
use minifb::Window;
use std::cmp::min;
use std::time::Instant;

pub struct Camera {
    pub eye: Vec3A,
    pub fb: Framebuffer,
    start: Instant,
    vp: Mat4,
}

impl Default for Camera {
    fn default() -> Self {
        let eye = Self::make_eye(0.0);
        let fb = Framebuffer::default();
        let vp = Self::make_vp(0.0, fb.width, fb.height);
        Self {
            eye: eye.into(),
            fb,
            start: Instant::now(),
            vp,
        }
    }
}

impl Camera {
    pub fn draw_voxel(&mut self, center: &Vec3A, color: u32) {
        let vertex = center.extend(1.0);
        let frag = self.vp * vertex;
        let frag: Vec3A = (frag / frag.w).into();
        // println!("frag: {:#?}", frag);
        if frag.x > -1.0 && frag.x < 1.0 && frag.y > -1.0 && frag.y < 1.0 {
            self.draw_point(&frag, color);
        }
    }

    pub fn draw_point(&mut self, center: &Vec3A, color: u32) {
        let x = ((center.x * 0.5 + 0.5) * (self.fb.width as f32)).round() as usize;
        let y = ((center.y * 0.5 + 0.5) * (self.fb.height as f32)).round() as usize;
        self.fb.point(color, x, y);
    }

    fn make_eye(step: f32) -> Vec3 {
        const R: f32 = 2.0;
        const H: f32 = 1.0;
        let angle = step;
        Vec3::new(angle.cos() * R, H, angle.sin() * R)
    }

    fn make_vp(step: f32, width: usize, height: usize) -> Mat4 {
        const FOV: f32 = 90.0;
        const NEAR: f32 = 0.1;
        const FAR: f32 = 100.0;
        let eye = Self::make_eye(step);
        let center = Vec3::new(0.0, -0.25, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let v = Mat4::look_at_lh(eye, center, up);
        let aspect = (width as f32) / (height as f32);
        let p = Mat4::perspective_rh(FOV.to_radians(), aspect, NEAR, FAR);
        p * v
    }

    pub fn update(&mut self) {
        let step = self.start.elapsed().as_micros() as f32 / 1_000_000.0;
        self.vp = Self::make_vp(step, self.fb.width, self.fb.height);
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
    fn point(&mut self, c: u32, x: usize, y: usize) {
        let offset = y * self.width + x;
        if offset < self.data.len() {
            self.data[y * self.width + x] = c;
        }
    }

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

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn update_window(&self, window: &mut Window) {
        window
            .update_with_buffer(&self.data, self.width, self.height)
            .unwrap();
    }
}
