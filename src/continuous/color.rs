
#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

// accessors
impl Rgba {
    pub fn r(&self) -> f32 {
        self.r
    }

    pub fn g(&self) -> f32 {
        self.g
    }

    pub fn b(&self) -> f32 {
        self.b
    }

    pub fn a(&self) -> f32 {
        self.a
    }
}

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn clear() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}