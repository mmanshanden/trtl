pub struct Turtle {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
}

impl Turtle {
    pub fn new() -> Self {
        Self {
            x: 50.0,
            y: 512.0,
            angle: 0.0,
        }
    }

    pub fn forward(&mut self, d: f64) {
        self.x += f64::cos(self.angle.to_radians()) * d;
        self.y += f64::sin(self.angle.to_radians()) * d;
    }

    pub fn left(&mut self, a: f64) {
        self.angle -= a;
    }

    pub fn right(&mut self, a: f64) {
        self.angle += a;
    }
}
