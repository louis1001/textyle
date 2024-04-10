#[derive(Clone, Debug)]
pub enum Sizing {
    Greedy(usize),
    Static(usize)
}

impl Sizing {
    pub fn clamped_accumulate(&mut self, other: &Sizing) {
        *self = match self {
            Sizing::Static(n) => {
                let mut result = other.clone();
                result.clamped_add(*n);

                result
            }
            Sizing::Greedy(n) => {
                Sizing::Greedy(*n + other.min_content_size())
            }
        }
    }

    pub fn clamped_add(&mut self, n: usize) {
        match self {
            Sizing::Static(sz) | Sizing::Greedy(sz) => {
                *sz = sz.checked_add(n).unwrap_or(*sz);
            }
        };
    }

    pub fn min_content_size(&self) -> usize {
        match self {
            Sizing::Static(sz) | Sizing::Greedy(sz) => *sz
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemSizing {
    pub horizontal: Sizing,
    pub vertical: Sizing
}

impl ItemSizing {
    pub fn new(horizontal: Sizing, vertical: Sizing) -> Self {
        ItemSizing { horizontal, vertical }
    }

    pub fn fit_into(&self, bounds: &super::Rect) -> super::Rect {
        let width = match self.horizontal {
            Sizing::Greedy(n) => bounds.width.max(n),
            Sizing::Static(n) => n
        };

        let height = match self.vertical {
            Sizing::Greedy(n) => bounds.height.max(n),
            Sizing::Static(n) => n
        };

        super::Rect::new(
            bounds.x,
            bounds.y,
            width,
            height
        )
    }
}