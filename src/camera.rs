// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use glam::{Mat4, Vec3, Vec3A, Vec4};
use minifb::Window;
use std::cmp::min;
use std::time::Instant;

pub struct Camera {
    pub eye: Vec3A,
    pub fb: Framebuffer,
    start: Instant,
    px: f32,
    min_point: f32,
    max_test: f32,
    vp: Mat4,
}

impl Default for Camera {
    fn default() -> Self {
        let eye = Self::make_eye(0.0);
        let fb = Framebuffer::default();
        let px = min(fb.width, fb.height) as f32;
        let min_point = 0.5 / px;
        let max_test = 16.0 / px;
        let vp = Self::make_vp(&eye, fb.width, fb.height);
        Self {
            eye: eye.into(),
            fb,
            start: Instant::now(),
            px,
            min_point,
            max_test,
            vp,
        }
    }
}

impl Camera {
    fn project_voxel(&self, center: &Vec4) -> Vec3A {
        let mut vertex = center.clone();
        vertex.w = 1.0;
        let mut frag = self.vp * vertex;
        frag.z = -center.w;
        (frag / frag.w).into()
    }

    fn frag_xy(&self, frag: Vec3A) -> (usize, usize) {
        let w = self.fb.width as f32;
        let h = self.fb.height as f32;
        let screen_pos = glam::Vec2::new(frag.x, frag.y) * 0.5 + 0.5;
        let screen_scale = glam::Vec2::new(w, h);
        let screen_pos = screen_pos * screen_scale;
        let screen_pos = screen_pos.floor();
        let x = screen_pos.x as usize;
        let y = screen_pos.y as usize;
        (x, y)
    }

    pub fn test_voxel(&mut self, center: &Vec4) -> bool {
        let frag = self.project_voxel(center);
        if frag.z < self.min_point {
            let xy = self.frag_xy(frag);
            self.fb.test_point(xy)
        } else {
            self.test_point(&frag)
        }
    }

    pub fn draw_voxel(&mut self, center: &Vec4, color: u32) {
        let frag = self.project_voxel(center);
        if frag.z < self.min_point {
            let xy = self.frag_xy(frag);
            self.fb.draw_point(xy, color);
        } else {
            self.draw_point(&frag, color);
        }
    }

    fn point_bounds(&self, center: &Vec3A) -> (usize, usize, usize, usize) {
        let w = self.fb.width as f32;
        let h = self.fb.height as f32;
        let screen_pos = glam::Vec2::new(center.x, center.y) * 0.5 + 0.5;
        let screen_scale = Vec3A::new(w, h, self.px);
        let screen_pos: Vec3A = screen_pos.extend(center.z).into();
        let screen_pos = screen_pos * screen_scale;

        let [x, y, r] = screen_pos.to_array();

        let l = (x - r).floor() as usize;
        let t = (y - r).floor() as usize;
        let b = (y + r).ceil() as usize;
        let r = (x + r).ceil() as usize;
        (l, t, r, b)
    }

    pub fn test_point(&self, center: &Vec3A) -> bool {
        if center.z > self.max_test {
            return true;
        }

        let bounds = self.point_bounds(center);
        self.fb.test_rect(bounds)
    }

    pub fn draw_point(&mut self, center: &Vec3A, color: u32) {
        let bounds = self.point_bounds(center);
        self.fb.draw_rect(color, bounds);
    }

    fn make_eye(step: f32) -> Vec3 {
        const R: f32 = 2.0;
        const H: f32 = 1.0;
        let angle = step;
        Vec3::new(angle.cos() * R, H, angle.sin() * R)
    }

    fn make_vp(eye: &Vec3, width: usize, height: usize) -> Mat4 {
        const FOV: f32 = 90.0;
        const NEAR: f32 = 0.1;
        const FAR: f32 = 100.0;
        let center = Vec3::new(0.0, -0.25, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let v = Mat4::look_at_lh(*eye, center, up);
        let aspect = (width as f32) / (height as f32);
        let p = Mat4::perspective_rh(FOV.to_radians(), aspect, NEAR, FAR);
        p * v
    }

    pub fn update(&mut self) {
        let step = self.start.elapsed().as_micros() as f32 / 1_000_000.0;
        let eye = Self::make_eye(step);
        self.eye = eye.into();
        self.vp = Self::make_vp(&eye, self.fb.width, self.fb.height);
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
    fn draw_point(&mut self, xy: (usize, usize), c: u32) {
        let offset = xy.1 * self.width + xy.0;
        if offset < self.data.len() {
            unsafe {
                *self.data.as_mut_ptr().add(offset) = c;
            }
        }
    }

    fn test_point(&self, xy: (usize, usize)) -> bool {
        let offset = xy.1 * self.width + xy.0;
        if offset < self.data.len() {
            unsafe { *self.data.as_ptr().add(offset) == 0 }
        } else {
            false
        }
    }

    fn draw_rect(&mut self, c: u32, b: (usize, usize, usize, usize)) {
        unsafe {
            let (l, t, r, mut b) = b;
            let width = r - l;
            let space = self.width - width;
            let start = t * self.width + l;
            let mut ptr = self.data.as_mut_ptr().add(start);
            while b > t {
                let mut r = r;
                while r > l {
                    // TODO: blending
                    *ptr |= c;
                    ptr = ptr.add(1);
                    r -= 1;
                }
                ptr = ptr.add(space);
                b -= 1;
            }
        }
    }

    fn test_rect(&self, b: (usize, usize, usize, usize)) -> bool {
        unsafe {
            let (l, t, r, mut b) = b;
            let width = r - l;
            let space = self.width - width;
            let start = t * self.width + l;
            let mut ptr = self.data.as_ptr().add(start);
            let mut wrote = false;
            while b > t {
                let mut r = r;
                while r > l {
                    wrote |= *ptr == 0;
                    ptr = ptr.add(1);
                    r -= 1;
                }
                ptr = ptr.add(space);
                b -= 1;
            }
            wrote
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn update_window(&mut self, window: &mut Window) {
        /*for row_index in 1..self.height {
            let start = row_index * self.width;
            let end = start + self.width;
            let mut all_empty = true;
            for index in start..end {
                let pixel = self.data[index];
                if pixel == 0 {
                    self.data[index] = self.data[index - self.width];
                } else {
                    all_empty = false;
                }
            }

            if all_empty {
                break;
            }
        }

        for row in self.data.chunks_mut(self.width) {
            let mut x = self.width;
            while x > 0 {
                x -= 1;
                if row[x] != 0 {
                    break;
                }
            }

            let mut old: u32 = 0;
            for pixel in row[0..x].iter_mut() {
                let new = *pixel;
                if new == 0 {
                    *pixel = old;
                } else {
                    old = new;
                }
            }
        }*/

        window
            .update_with_buffer(&self.data, self.width, self.height)
            .unwrap();
    }
}
