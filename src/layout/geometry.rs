#[derive(Clone, Debug)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: i64, y: i64, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn sized(width: usize, height: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn from_size(size: &Size) -> Self {
        Self { x: 0, y: 0, width: size.width, height: size.height }
    }

    // Utilities
    pub fn max_x(&self) -> i64 {
        self.x + self.width as i64
    }

    pub fn max_y(&self) -> i64 {
        self.y + self.height as i64
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}

pub struct Vector {
    x: i64,
    y: i64
}

impl Vector {
    pub fn zero() -> Vector {
        Vector { x: 0, y: 0 }
    }
    
    pub fn new(x: i64, y: i64) -> Vector {
        Vector {
            x, y
        }
    }

    pub fn sub(vec1: &Vector, vec2: &Vector) -> Vector {
        Vector { x: vec1.x - vec2.x, y: vec1.y - vec2.y }
    }
}

impl Vector {
    pub fn magnitude(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}

#[derive(Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Size {
            width,
            height
        }
    }

    pub fn zero() -> Self {
        Size { width: 0, height: 0 }
    }

    pub fn to_vector(&self) -> Vector {
        Vector::new(self.width as i64, self.height as i64)
    }
}