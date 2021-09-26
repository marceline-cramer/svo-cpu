use glam::{Mat4, Vec3, Vec3A, Vec4};
use minifb::Window;
use std::cmp::min;
use std::time::Instant;

pub struct Camera {
    pub eye: Vec3A,
    pub fb: Framebuffer,
    start: Instant,
    min_point: f32,
    vp: Mat4,
}

impl Default for Camera {
    fn default() -> Self {
        let eye = Self::make_eye(0.0);
        let fb = Framebuffer::default();
        let min_point = 0.5 / min(fb.width, fb.height) as f32;
        let vp = Self::make_vp(0.0, fb.width, fb.height);
        Self {
            eye: eye.into(),
            fb,
            start: Instant::now(),
            min_point,
            vp,
        }
    }
}

impl Camera {
    pub fn draw_voxel(&mut self, center: &Vec4, color: u32) -> bool {
        let mut vertex = center.clone();
        vertex.w = 1.0;
        let mut frag = self.vp * vertex;
        frag.z = -center.w;
        let frag: Vec3A = (frag / frag.w).into();
        // println!("frag: {:#?}", frag);
        if frag.z < self.min_point {
            let w = self.fb.width as f32;
            let h = self.fb.height as f32;
            let screen_pos = glam::Vec2::new(frag.x, frag.y) * 0.5 + 0.5;
            let screen_scale = glam::Vec2::new(w, h);
            let screen_pos = screen_pos * screen_scale;
            let screen_pos = screen_pos.floor();
            let x = screen_pos.x as usize;
            let y = screen_pos.y as usize;
            self.fb.point(color, x, y)
        } else {
            self.draw_point(&frag, color)
        }
    }

    pub fn draw_point(&mut self, center: &Vec3A, color: u32) -> bool {
        const MAX_TEST: f32 = 0.1;
        if color == 0 && center.z > MAX_TEST {
            return true;
        }

        let w = self.fb.width as f32;
        let h = self.fb.height as f32;
        let screen_pos = glam::Vec2::new(center.x, center.y) * 0.5 + 0.5;
        let screen_scale = Vec4::new(w, h, w, h);
        let screen_pos = screen_pos.extend(center.z).extend(center.z);
        let screen_pos = screen_pos * screen_scale;

        let [x, y, rx, ry] = screen_pos.to_array();

        let l = (x - rx).floor() as usize;
        let t = (y - ry).floor() as usize;
        let b = (y + ry).ceil() as usize;
        let r = (x + rx).ceil() as usize;
        self.fb.rect(color, l, t, r, b)
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
    fn point(&mut self, c: u32, x: usize, y: usize) -> bool {
        let offset = y * self.width + x;
        if offset < self.data.len() {
            let old_c = self.data[offset];
            self.data[offset] = old_c | c;
            old_c == 0
        } else {
            false
        }
    }

    fn rect(&mut self, c: u32, l: usize, t: usize, r: usize, b: usize) -> bool {
        // let r = min(r, self.width - 1);
        // let b = min(b, self.height - 1);
        let xt = t * self.width;
        let mut xl = xt + l;
        let mut xr = xt + r;
        let mut wrote = false;
        for _ in t..b {
            for pixel in self.data[xl..xr].iter_mut() {
                let old = *pixel;
                wrote = wrote | (old == 0);
                *pixel = old | c;
                // *pixel += 8;
                // *pixel = c;
            }
            xl += self.width;
            xr += self.width;
        }
        wrote
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
