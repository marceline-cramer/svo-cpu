// noise functions borrowed from minetest

const NOISE_MAGIC_X: isize = 1619;
const NOISE_MAGIC_Y: isize = 31337;
const NOISE_MAGIC_Z: isize = 52591;
const NOISE_MAGIC_SEED: isize = 1013;

pub fn noise2d(x: isize, y: isize, seed: isize) -> f32 {
    let n = (NOISE_MAGIC_X * x + NOISE_MAGIC_Y * y + NOISE_MAGIC_SEED * seed) & 0x7fffffff;
    let n = (n >> 13) ^ n;
    let n = (n * (n * n * 60493 + 199990303) + 1376312589) & 0x7fffffff;
    1.0 - (n as f32 / 0x40000000 as f32)
}

pub fn noise3d(x: isize, y: isize, z: isize, seed: isize) -> f32 {
    let n = (NOISE_MAGIC_X * x + NOISE_MAGIC_Y * y + NOISE_MAGIC_Z * z + NOISE_MAGIC_SEED * seed)
        & 0x7fffffff;
    let n = (n >> 13) ^ n;
    let n = (n * (n * n * 60493 + 19990303) + 1376312589) & 0x7fffffff;
    1.0 - (n as f32 / 0x40000000 as f32)
}
