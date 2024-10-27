pub struct Tile {
    width: i32,
    height: i32,
    buffer: Vec<u32>,
}

impl Tile {
    
}

pub struct Canvas {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let len = width * height * 4;

        Self {
            width: width as i32,
            height: height as i32,
            pixels: vec![255; len as usize],
        }
    }

    pub fn get_pixel_data(&self) -> &Vec<u8> {
        &self.pixels
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        self.width = new_width;
        self.height = new_height;
    }

    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = 255;
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        
        let idx = x + y * self.width;
        let idx = (idx << 2) as usize;

        self.pixels[idx] = r;
        self.pixels[idx + 1] = g;
        self.pixels[idx + 2] = b;
        self.pixels[idx + 3] = 255;//0xFF << 24 | b << 16 | g << 8 | r; // abgr
    }

    pub fn draw_line(&mut self, x: i32, y: i32, x2: i32, y2: i32, r: u8, g: u8, b: u8) {
        let mut x = x;
        let mut y = y;

        let w = x2 - x ;
        let h = y2 - y ;
        let dx1 = i32::signum(w);
        let dy1 = i32::signum(h);
        let mut dx2 = i32::signum(w);
        let mut dy2 = 0;

        let mut longest = i32::abs(w);
        let mut shortest = i32::abs(h);

        if longest <= shortest {
            longest = i32::abs(h) ;
            shortest = i32::abs(w) ;
            dy2 = i32::signum(h);
            dx2 = 0;    
        }

        let mut numerator = longest >> 1 ;
        
        for _ in 0..longest + 1 {
            self.set_pixel(x, y, r, g, b);

            numerator += shortest;

            if numerator >= longest {
                numerator -= longest ;
                x += dx1 ;
                y += dy1 ;
            } else {
                x += dx2 ;
                y += dy2 ;
            }
        }
    }
}
