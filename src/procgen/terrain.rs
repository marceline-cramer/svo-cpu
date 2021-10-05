use super::*;

pub struct TerrainGen {}

impl Default for TerrainGen {
    fn default() -> Self {
        Self {}
    }
}

impl ProcGen for TerrainGen {
    fn is_occupied(&self, pos: &Vec3A) -> bool {
        let i: i32 = rand::random();
        let i = i & 0xff;
        let i = (i as f32) / 127.0 - 1.0;
        (i + pos.y) < 0.0
    }
}
