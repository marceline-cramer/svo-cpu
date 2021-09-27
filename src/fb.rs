use minifb::Window;

type Bounds = (usize, usize, usize, usize);
type Point = (usize, usize);

type Pixel = u32;
type PixelSimd = packed_simd::u32x16;
pub type ColorBuffer = Framebuffer<Pixel>;

pub trait Target<P> {
    fn draw(&mut self, ptr: *mut P, p: P);
    fn test(&self, ptr: *const P) -> bool;
}

pub struct Framebuffer<P> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<P>,
}

impl Default for Framebuffer<Pixel> {
    fn default() -> Self {
        let width = 1280;
        let height = 720;
        let data = vec![0; width * height];
        Self {
            width,
            height,
            data,
        }
    }
}

impl Target<Pixel> for Framebuffer<Pixel> {
    fn draw(&mut self, ptr: *mut Pixel, p: Pixel) {
        // TODO: blending
        unsafe {
            *ptr = p;
        }
    }

    fn test(&self, ptr: *const Pixel) -> bool {
        unsafe { *ptr == 0 }
    }
}

impl Framebuffer<Pixel> {
    pub fn draw_point(&mut self, xy: Point, p: Pixel) {
        if let Some(offset) = self.calc_offset(xy) {
            let ptr = unsafe { self.data.as_mut_ptr().add(offset) };
            self.draw(ptr, p);
        }
    }

    pub fn test_point(&self, xy: Point) -> bool {
        if let Some(offset) = self.calc_offset(xy) {
            let ptr = unsafe { self.data.as_ptr().add(offset) };
            self.test(ptr)
        } else {
            false
        }
    }

    pub fn draw_rect(&mut self, b: Bounds, c: Pixel) {
        unsafe {
            let (l, t, r, mut b) = b;
            let width = r - l;
            let space = self.width - width;
            let start = t * self.width + l;
            let mut ptr = self.data.as_mut_ptr().add(start);
            while b > t {
                let mut r = r;
                while r > l {
                    self.draw(ptr, c);
                    ptr = ptr.add(1);
                    r -= 1;
                }
                ptr = ptr.add(space);
                b -= 1;
            }
        }
    }

    pub fn test_rect(&self, b: Bounds) -> bool {
        unsafe {
            let (l, t, r, mut b) = b;
            let width = r - l;
            let space = self.width - width;
            let start = t * self.width + l;
            let mut ptr = self.data.as_ptr().add(start);
            while b > t {
                let mut r = r;
                while r > l {
                    if self.test(ptr) {
                        return true;
                    }
                    ptr = ptr.add(1);
                    r -= 1;
                }
                ptr = ptr.add(space);
                b -= 1;
            }
            false
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn update_window(&mut self, window: &mut Window) {
        window
            .update_with_buffer(&self.data, self.width, self.height)
            .unwrap();
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
