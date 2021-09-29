// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use super::fb::ColorBuffer as Framebuffer;
use glam::{Mat4, Vec3, Vec3A, Vec4};
use std::cmp::min;
use std::time::Instant;

pub trait Camera {
    fn get_eye(&self) -> Vec3A;
    fn project_voxel(&self, center: &Vec4) -> Vec3A;
    fn frag_xy(&self, frag: &Vec3A) -> (usize, usize);
    fn test_voxel(&self, center: &Vec4) -> bool;
    fn draw_voxel(&mut self, center: &Vec4, color: u32);
    fn point_bounds(&self, center: &Vec3A) -> (usize, usize, usize, usize);
    fn draw_point(&mut self, center: &Vec3A, color: u32);
    fn test_point(&self, center: &Vec3A) -> bool;
}

pub struct SpinnyCamera {
    pub eye: Vec3A,
    pub fb: Framebuffer,
    start: Instant,
    px: f32,
    min_point: f32,
    max_test: f32,
    vp: Mat4,
}

impl Default for SpinnyCamera {
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

impl SpinnyCamera {
    fn make_eye(step: f32) -> Vec3 {
        const R: f32 = 2.0;
        const H: f32 = 1.0;
        let angle = step;
        Vec3::new(angle.cos() * R, H, angle.sin() * R)
    }

    fn make_vp(eye: &Vec3, width: usize, height: usize) -> Mat4 {
        const FOV: f32 = 65.0;
        const NEAR: f32 = 0.1;
        const FAR: f32 = 100.0;
        let center = Vec3::new(0.0, -0.15, 0.0);
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

impl Camera for SpinnyCamera {
    fn get_eye(&self) -> Vec3A {
        self.eye
    }

    fn project_voxel(&self, center: &Vec4) -> Vec3A {
        let mut vertex = center.clone();
        vertex.w = 1.0;
        let mut frag = self.vp * vertex;
        frag.z = -center.w;
        (frag / frag.w).into()
    }

    fn frag_xy(&self, frag: &Vec3A) -> (usize, usize) {
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

    fn draw_voxel(&mut self, center: &Vec4, color: u32) {
        let frag = self.project_voxel(center);
        if frag.z < self.min_point {
            let xy = self.frag_xy(&frag);
            self.fb.draw_point(xy, color);
        } else {
            self.draw_point(&frag, color);
        }
    }

    fn test_voxel(&self, center: &Vec4) -> bool {
        let frag = self.project_voxel(center);
        if frag.z < self.min_point {
            let xy = self.frag_xy(&frag);
            self.fb.test_point(xy)
        } else {
            self.test_point(&frag)
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

        const MIN_MARGIN: usize = 0;
        const MAX_MARGIN: usize = 1;
        let l = (x - r) as usize - MIN_MARGIN;
        let t = (y - r) as usize - MIN_MARGIN;
        let b = (y + r) as usize + MAX_MARGIN;
        let r = (x + r) as usize + MAX_MARGIN;
        (l, t, r, b)
    }

    fn draw_point(&mut self, center: &Vec3A, color: u32) {
        let bounds = self.point_bounds(center);
        self.fb.draw_rect(bounds, color);
    }

    fn test_point(&self, center: &Vec3A) -> bool {
        if center.z > self.max_test {
            return true;
        }

        let bounds = self.point_bounds(center);
        self.fb.test_rect(bounds)
    }
}
