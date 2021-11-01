// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use crate::fb::ColorBuffer;
use glam::{Mat4, Vec3A, Vec4};

pub mod spinny_camera;

pub struct Camera {
    pub eye: Vec3A,
    pub vp: Mat4,
}

pub struct DrawConfig {
    pub min_rect: f32,
    pub max_rect: f32,
    pub max_test: usize,
}

impl Camera {
    pub fn project_voxel(&self, center: &Vec4) -> Vec3A {
        let mut vertex = center.clone();
        vertex.w = 1.0;
        let mut frag = self.vp * vertex;
        frag.z = -center.w;
        (frag / frag.w).into()
    }

    pub fn draw_voxel(
        &self,
        fb: &mut ColorBuffer,
        c: &DrawConfig,
        is_leaf: bool,
        center: &Vec4,
        color: u32,
    ) -> bool {
        let projected = self.project_voxel(&center);
        if !is_leaf {
            if projected.z > c.max_rect {
                Self::test_rect(c.max_test, fb, &projected)
            } else if projected.z > c.min_rect {
                Self::draw_rect(fb, &projected, color);
                false
            } else {
                Self::draw_point(fb, &projected, color);
                false
            }
        } else {
            if projected.z < c.min_rect {
                Self::draw_point(fb, &projected, color);
                false
            } else {
                Self::draw_rect(fb, &projected, color);
                false
            }
        }
    }

    pub fn test_rect(max_test: usize, fb: &mut ColorBuffer, projected: &Vec3A) -> bool {
        if let Some(bounds) = fb.point_bounds(projected) {
            let area = (bounds.2 - bounds.0) * (bounds.3 - bounds.1);
            if area < max_test {
                fb.test_rect(bounds)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn draw_rect(fb: &mut ColorBuffer, projected: &Vec3A, color: u32) {
        if let Some(bounds) = fb.point_bounds(projected) {
            fb.draw_rect(bounds, color);
        }
    }

    pub fn draw_point(fb: &mut ColorBuffer, projected: &Vec3A, color: u32) {
        let xy = fb.frag_xy(projected);
        fb.draw_point(xy, color);
    }
}
