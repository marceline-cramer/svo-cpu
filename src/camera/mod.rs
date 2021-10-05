// SPDX-License-Identifier: MIT
// Copyright (c) 2021 Marceline Cramer

use glam::{Vec3A, Vec4};

mod spinny_camera;

pub use spinny_camera::SpinnyCamera;

pub trait Camera {
    fn get_eye(&self) -> Vec3A;
    fn handle_voxel(&mut self, is_leaf: bool, center: &Vec4, color: u32) -> bool;
}
