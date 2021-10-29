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
    pub data: Vec<P>,
}

impl Default for Framebuffer<Pixel> {
    fn default() -> Self {
        let width = 1600;
        let height = 900;
        let data = vec![0; width * height];
        Self {
            width,
            height,
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
            if r >= self.width || b >= self.height {
                return;
            }
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
    fn calc_offset(&self, xy: Point) -> Option<usize> {
        let offset = xy.1 * self.width + xy.0;
        if offset < self.data.len() {
            Some(offset)
        } else {
            None
        }
    }
}
