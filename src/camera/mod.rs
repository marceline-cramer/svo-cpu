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
    pub max_test: f32,
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
                if projected.z < c.max_test {
                    Self::test_rect(fb, &projected)
                } else {
                    true
                }
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

    pub fn test_rect(fb: &mut ColorBuffer, projected: &Vec3A) -> bool {
        let bounds = fb.point_bounds(projected);
        fb.test_rect(bounds)
    }

    pub fn draw_rect(fb: &mut ColorBuffer, projected: &Vec3A, color: u32) {
        let bounds = fb.point_bounds(projected);
        fb.draw_rect(bounds, color);
    }

    pub fn draw_point(fb: &mut ColorBuffer, projected: &Vec3A, color: u32) {
        let xy = fb.frag_xy(projected);
        fb.draw_point(xy, color);
    }
}
