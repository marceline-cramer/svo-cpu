// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use glam::{Vec3A, Vec4};

mod spinny_camera;

pub use spinny_camera::SpinnyCamera;

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
