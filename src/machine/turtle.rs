pub struct Turtle {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl Turtle {
    pub fn new() -> Self {
        Self {
            x: 50.0,
            y: 512.0,
            angle: 0.0,
        }
    }

    pub fn forward(&mut self, d: f32) {
        self.x += f32::cos(self.angle.to_radians()) * d;
        self.y += f32::sin(self.angle.to_radians()) * d;
    }

    pub fn left(&mut self, a: f32) {
        self.angle -= a;
    }

    pub fn right(&mut self, a: f32) {
        self.angle += a;
    }
}
