use minifb::Window;

type Bounds = (usize, usize, usize, usize);
type Point = (usize, usize);
pub type ColorBuffer = Framebuffer<u32>;

pub trait Target<P> {
    fn draw(&mut self, ptr: *mut P, p: P);
    fn test(&self, ptr: *const P) -> bool;
}

pub struct Framebuffer<P>
where
    P: Copy,
{
    pub width: usize,
    pub height: usize,
    pub data: Vec<P>,
}

impl<P> Target<P> for Framebuffer<P>
where
    P: Copy + From<u8> + std::cmp::PartialEq,
{
    fn draw(&mut self, ptr: *mut P, p: P) {
        // TODO: blending
        unsafe {
            *ptr = p;
        }
    }

    fn test(&self, ptr: *const P) -> bool {
        unsafe { *ptr == P::from(0) }
    }
}

impl<P> Framebuffer<P>
where
    P: Copy + Clone + From<bool>,
{
    pub fn clear(&mut self) {
        self.data.fill(false.into());
    }
}

impl<P> Framebuffer<P>
where
    P: Copy + From<u8> + std::cmp::PartialEq,
{
    fn calc_offset(&self, xy: Point) -> Option<usize> {
        let offset = xy.1 * self.width + xy.0;
        if offset < self.data.len() {
            Some(offset)
        } else {
            None
        }
    }

    pub fn draw_point(&mut self, xy: Point, p: P) {
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

    pub fn draw_rect(&mut self, b: Bounds, c: P) {
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

    // TODO: rewrite with pointers so that .test() can be used
    /*pub fn fill_holes(&mut self) {
        for row_index in 1..self.height {
            let start = row_index * self.width;
            let end = start + self.width;
            let mut all_empty = true;
            for index in start..end {
                let pixel = self.data[index];
                if !(pixel.into()) {
                    self.data[index] = self.data[index - self.width];
                } else {
                    all_empty = false;
                }
            }

            if all_empty {
                break;
            }
        }

        for row in self.data.chunks_mut(self.width) {
            let mut x = self.width;
            while x > 0 {
                x -= 1;
                if row[x].into() {
                    break;
                }
            }

            let mut old: P = false.into();
            for pixel in row[0..x].iter_mut() {
                let new = *pixel;
                if !(new.into()) {
                    *pixel = old;
                } else {
                    old = new;
                }
            }
        }
    }*/
}

impl Default for ColorBuffer {
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

impl ColorBuffer {
    pub fn update_window(&mut self, window: &mut Window) {
        window
            .update_with_buffer(&self.data, self.width, self.height)
            .unwrap();
    }
}
