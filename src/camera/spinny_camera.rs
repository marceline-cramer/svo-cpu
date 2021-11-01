// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use super::{Camera, DrawConfig};
use crate::fb::ColorBuffer as Framebuffer;
use glam::{Mat4, Vec3};
use std::time::Instant;

pub struct SpinnyCamera {
    pub camera: Camera,
    pub draw_config: DrawConfig,
    start: Instant,
}

impl SpinnyCamera {
    pub fn new(fb: &Framebuffer) -> Self {
        let eye = Self::make_eye(0.0);
        let camera = Camera {
            eye: eye.into(),
            vp: Self::make_vp(&eye, fb.width, fb.height),
        };

        let draw_config = DrawConfig {
            min_rect: 0.5 / fb.px,
            max_rect: 6.0 / fb.px,
            max_test: 1024,
        };

        Self {
            camera,
            draw_config,
            start: Instant::now(),
        }
    }

    fn make_eye(step: f32) -> Vec3 {
        const R: f32 = 3.0;
        const H: f32 = 2.0;
        let angle = step;
        Vec3::new(angle.cos() * R, H, angle.sin() * R)
    }

    fn make_vp(eye: &Vec3, width: usize, height: usize) -> Mat4 {
        const FOV: f32 = 60.0;
        const NEAR: f32 = 0.1;
        const FAR: f32 = 100.0;
        let center = Vec3::new(0.0, -0.15, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let v = Mat4::look_at_lh(*eye, center, up);
        let aspect = (width as f32) / (height as f32);
        let p = Mat4::perspective_rh(FOV.to_radians(), aspect, NEAR, FAR);
        p * v
    }

    pub fn update(&mut self, fb: &Framebuffer) {
        let step = self.start.elapsed().as_micros() as f32 / 1_000_000.0;
        let eye = Self::make_eye(step);
        self.camera.eye = eye.into();
        self.camera.vp = Self::make_vp(&eye, fb.width, fb.height);
    }
}
