use glam::Vec3A;

type Bounds = (usize, usize, usize, usize);
type Point = (usize, usize);

type Pixel = u32;
pub type ColorBuffer = Framebuffer<Pixel>;

pub trait Target<P> {
    fn draw(ptr: *mut P, p: P);
    fn test(ptr: *const P) -> bool;
}

pub struct Framebuffer<P> {
    pub width: usize,
    pub height: usize,
    pub px: f32,
    pub data: Vec<P>,
}

impl Default for Framebuffer<Pixel> {
    fn default() -> Self {
        let width = 1280;
        let height = 720;
        let px = std::cmp::min(width, height) as f32;
        let data = vec![0; width * height];
        Self {
            width,
            height,
            px,
            data,
        }
    }
}

fn pixel_to_simd(p: Pixel) -> packed_simd::u32x4 {
    let a = (p >> 24) & 0xff;
    let r = (p >> 16) & 0xff;
    let g = (p >> 8) & 0xff;
    let b = p & 0xff;
    packed_simd::u32x4::new(a, r, g, b)
}

fn simd_to_pixel(simd: packed_simd::u32x4) -> Pixel {
    unsafe {
        (simd.extract_unchecked(0) << 24) | // a
        (simd.extract_unchecked(1) << 16) | // r
        (simd.extract_unchecked(2) << 8) | // g
        simd.extract_unchecked(3) // b
    }
}

const ALPHA_BIAS: u32 = 24;
const MAX_PIXEL: packed_simd::u32x4 = packed_simd::u32x4::new(0xff, 0xff, 0xff, 0xff);

impl Target<Pixel> for Framebuffer<Pixel> {
    fn draw(ptr: *mut Pixel, p: Pixel) {
        unsafe {
            let dst = *ptr;
            if dst == 0 {
                *ptr = p;
            } else {
                let dst = pixel_to_simd(dst);
                let dst_a = dst.extract_unchecked(0);
                if dst_a < 255 {
                    let a = 256 - dst_a;
                    let dst = dst.replace_unchecked(0, dst_a + ALPHA_BIAS);
                    let src = pixel_to_simd(p);
                    let dst = MAX_PIXEL.min(((src * a) >> 8) + dst);
                    *ptr = simd_to_pixel(dst);
                }
            }
        }
    }

    fn test(ptr: *const Pixel) -> bool {
        unsafe { *ptr & 0xff000000 != 0xff000000 }
    }
}

impl Framebuffer<Pixel> {
    pub fn draw_point(&mut self, xy: Point, p: Pixel) {
        if let Some(offset) = self.calc_offset(xy) {
            let ptr = unsafe { self.data.as_mut_ptr().add(offset) };
            Self::draw(ptr, p);
        }
    }

    pub fn test_point(&self, xy: Point) -> bool {
        if let Some(offset) = self.calc_offset(xy) {
            let ptr = unsafe { self.data.as_ptr().add(offset) };
            Self::test(ptr)
        } else {
            false
        }
    }

    pub fn draw_rect(&mut self, b: Bounds, c: Pixel) {
        unsafe {
            let (l, t, r, b) = b;
            let w = r - l;
            let h = b - t;
            let space = self.width - w;
            let start = t * self.width + l;
            let mut ptr = self.data.as_mut_ptr().add(start);
            let mut y = 0;
            while y < h {
                let mut x = 0;
                while x < w {
                    Self::draw(ptr, c);
                    ptr = ptr.add(1);
                    x += 1;
                }
                ptr = ptr.add(space);
                y += 1;
            }
        }
    }

    pub fn test_rect(&self, b: Bounds) -> bool {
        unsafe {
            let (l, t, r, b) = b;
            let w = r - l;
            let h = b - t;
            let space = self.width - w;
            let start = t * self.width + l;
            let mut ptr = self.data.as_ptr().add(start);
            let mut y = 0;
            while y < h {
                let mut x = 0;
                while x < w {
                    if Self::test(ptr) {
                        return true;
                    }
                    ptr = ptr.add(1);
                    x += 1;
                }
                ptr = ptr.add(space);
                y += 1;
            }
            false
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}

impl<P> Framebuffer<P> {
    pub fn frag_xy(&self, frag: &Vec3A) -> (usize, usize) {
        let w = self.width as f32;
        let h = self.height as f32;
        let screen_pos = glam::Vec2::new(frag.x, frag.y) * 0.5 + 0.5;
        let screen_scale = glam::Vec2::new(w, h);
        let screen_pos = screen_pos * screen_scale;
        let screen_pos = screen_pos.floor();
        let x = screen_pos.x as usize;
        let y = screen_pos.y as usize;
        (x, y)
    }

    pub fn point_bounds(&self, center: &Vec3A) -> Option<(usize, usize, usize, usize)> {
        if center.x < (-center.z - 1.0) || center.y < (-center.z - 1.0) {
            return None;
        }

        let w = self.width as f32;
        let h = self.height as f32;

        if center.x > (1.0 + center.z) || center.y > (1.0 + center.z) {
            return None;
        }

        let screen_pos = glam::Vec2::new(center.x, center.y) * 0.5 + 0.5;
        let screen_scale = Vec3A::new(w, h, self.px);
        let screen_pos: Vec3A = screen_pos.extend(center.z).into();
        let screen_pos = screen_pos * screen_scale;

        let [x, y, r] = screen_pos.to_array();

        const MIN_MARGIN: usize = 1;
        const MAX_MARGIN: usize = 1;
        let l = (x - r).max(MIN_MARGIN as f32) as usize - MIN_MARGIN;
        let t = (y - r).max(MIN_MARGIN as f32) as usize - MIN_MARGIN;
        let b = ((y + r) as usize + MAX_MARGIN).min(self.height);
        let r = ((x + r) as usize + MAX_MARGIN).min(self.width);

        if l < r && t < b {
            Some((l, t, r, b))
        } else {
            None
        }
    }

    fn calc_offset(&self, xy: Point) -> Option<usize> {
        if xy.0 >= self.width || xy.1 >= self.height {
            None
        } else {
            Some(xy.1 * self.width + xy.0)
        }
    }
}
