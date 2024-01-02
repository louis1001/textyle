use std::collections::HashSet;

pub mod sizing;
pub mod alignment;

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

    // Utilities
    pub fn max_x(&self) -> i64 {
        self.x + self.width as i64
    }

    pub fn max_y(&self) -> i64 {
        self.y + self.height as i64
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Clone)]
pub enum Layout {
    Text(String),
    Width(usize, Box<Layout>),
    Height(usize, Box<Layout>),
    TopPadding(usize, Box<Layout>),
    RightPadding(usize, Box<Layout>),
    BottomPadding(usize, Box<Layout>),
    LeftPadding(usize, Box<Layout>),
    VCenter(Box<Layout>),
    HCenter(Box<Layout>),
    VBottomAlign(Box<Layout>),
    HRightAlign(Box<Layout>),
    VTopAlign(Box<Layout>),
    HLeftAlign(Box<Layout>),
    Background(char, Box<Layout>),
    Border(usize, char, HashSet<alignment::Edge>, Box<Layout>),

    VerticalStack(alignment::HorizontalAlignment, Vec<Layout>),
    HorizontalStack(alignment::VerticalAlignment, Vec<Layout>),
}

#[derive(Clone, Debug)]
pub enum SizedNode {
    Text(String),
    Width(usize, SizedLayout),
    Height(usize, SizedLayout),
    TopPadding(usize, SizedLayout),
    RightPadding(usize, SizedLayout),
    BottomPadding(usize, SizedLayout),
    LeftPadding(usize, SizedLayout),
    VCenter(SizedLayout),
    HCenter(SizedLayout),
    VBottomAlign(SizedLayout),
    HRightAlign(SizedLayout),
    VTopAlign(SizedLayout),
    HLeftAlign(SizedLayout),
    Background(char, SizedLayout),
    Border(usize, char, HashSet<alignment::Edge>, SizedLayout),

    VerticalStack(alignment::HorizontalAlignment, Vec<SizedLayout>),
    HorizontalStack(alignment::VerticalAlignment, Vec<SizedLayout>),
}

#[derive(Clone, Debug)]
pub struct SizedLayout {
    pub node: Box<SizedNode>,
    pub sizing: sizing::ItemSizing
}

impl SizedLayout {
    fn new(node: SizedNode, sizing: sizing::ItemSizing) -> Self {
        SizedLayout { node: Box::new(node), sizing }
    }
}

impl Layout {
    fn calculate_line_size(&self, line: &str, bounds: &Rect) -> Rect {
        use unicode_segmentation::UnicodeSegmentation;
        let graphemes = line.graphemes(true).collect::<Vec<_>>();
        let rows = ((graphemes.len() as f64) / (bounds.width as f64)).ceil() as usize;

        if rows < 2 {
            Rect::sized(graphemes.len(), 1)
        } else {
            Rect::sized(bounds.width, rows)
        }
    }

    pub fn resolve_size(&self, bounds: &Rect) -> SizedLayout {
        use Layout::*;
        use sizing::Sizing::*;

        match self {
            Text(t) => {
                let lines = t.lines();

                let mut width = 0usize;
                let mut height = 0usize;
                for line in lines {
                    let sz = self.calculate_line_size(line, bounds);
                    if sz.width > width {
                        width = sz.width;
                    }

                    height += sz.height;
                }

                let sizing = sizing::ItemSizing::new(Static(width), Static(height));

                SizedLayout::new(SizedNode::Text(t.clone()), sizing)
            }
            VCenter(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing::new(content_size.horizontal, Greedy(min_height));

                SizedLayout::new(SizedNode::VCenter(resolved), sizing)
            }
            VBottomAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VBottomAlign(resolved), sizing)
            }
            HCenter(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HCenter(resolved), sizing)
            }
            HRightAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HRightAlign(resolved), sizing)
            }
            VTopAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VTopAlign(resolved), sizing)
            }
            HLeftAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HRightAlign(resolved), sizing)
            }
            Width(size, node) => {
                let mut bounds = bounds.clone();
                bounds.width = *size;

                let resolved_content = node.resolve_size(&bounds);
                let mut frame = resolved_content.sizing.clone();
                frame.horizontal = Static(*size);

                SizedLayout::new(SizedNode::Width(*size, resolved_content), frame)
            }
            Height(size, node) => {
                let mut bounds = bounds.clone();
                bounds.height = *size;

                let resolved_content = node.resolve_size(&bounds);
                let mut frame = resolved_content.sizing.clone();
                frame.vertical = Static(*size);

                SizedLayout::new(SizedNode::Height(*size, resolved_content), frame)
            }
            TopPadding(n, node) => {
                let resolved = node.resolve_size(&bounds);
                let mut frame = resolved.sizing.clone();
                
                frame.vertical.clamped_add(*n);

                if frame.vertical.min_content_size() > bounds.height {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.height = bounds.height.checked_sub(*n).unwrap_or(0);

                    let resolved_content = node.resolve_size(&bounds);
                    let mut sizing = resolved_content.sizing.clone();

                    sizing.vertical.clamped_add(*n);

                    SizedLayout::new(SizedNode::TopPadding(*n, resolved_content), sizing)
                } else {
                    SizedLayout::new(SizedNode::TopPadding(*n, resolved), frame)
                }
            }
            RightPadding(n, node) => {
                let resolved = node.resolve_size(&bounds);
                let mut frame = resolved.sizing.clone();
                
                frame.horizontal.clamped_add(*n);
                if frame.horizontal.min_content_size() > bounds.width {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.width = bounds.width.checked_sub(*n).unwrap_or(0);

                    let resolved_content = node.resolve_size(&bounds);
                    frame = resolved_content.sizing.clone();
                    frame.horizontal.clamped_add(*n);

                    SizedLayout::new(SizedNode::RightPadding(*n, resolved_content), frame)
                } else {
                    SizedLayout::new(SizedNode::RightPadding(*n, resolved), frame)
                }
            }
            BottomPadding(n, node) => {
                let resolved = node.resolve_size(&bounds);
                let mut frame = resolved.sizing.clone();
                
                frame.vertical.clamped_add(*n);
                if frame.vertical.min_content_size() > bounds.height {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.height = bounds.height.checked_sub(*n).unwrap_or(0);

                    let resolved_content = node.resolve_size(&bounds);
                    let mut frame = resolved_content.sizing.clone();

                    frame.vertical.clamped_add(*n);

                    SizedLayout::new(SizedNode::BottomPadding(*n, resolved_content), frame)
                } else {
                    SizedLayout::new(SizedNode::BottomPadding(*n, resolved), frame)
                }
            }
            LeftPadding(n, node) => {
                let resolved = node.resolve_size(&bounds);
                let mut frame = resolved.sizing.clone();
                
                frame.horizontal.clamped_add(*n);
                if frame.horizontal.min_content_size() > bounds.width {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.width -= n;

                    let resolved_content = node.resolve_size(&bounds);
                    let mut frame = resolved_content.sizing.clone();

                    frame.horizontal.clamped_add(*n);

                    SizedLayout::new(SizedNode::LeftPadding(*n, resolved_content), frame)
                } else {
                    SizedLayout::new(SizedNode::LeftPadding(*n, resolved), frame)
                }
            }
            Background(c, node) => {
                let resolved_content = node.resolve_size(&bounds);
                let frame = resolved_content.sizing.clone();

                SizedLayout::new(SizedNode::Background(*c, resolved_content), frame)
            }
            Border(n, c, edges, node) => {
                let mut resolved_content = node.resolve_size(&bounds);
                let mut frame = resolved_content.sizing.clone();
                
                if edges.contains(&alignment::Edge::Top) {
                    frame.vertical.clamped_add(*n);
                }
                if edges.contains(&alignment::Edge::Bottom) {
                    frame.vertical.clamped_add(*n);
                }

                if frame.vertical.min_content_size() > bounds.height {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.height = bounds.height.checked_sub(*n).unwrap_or(0);

                    resolved_content = node.resolve_size(&bounds);
                    frame = resolved_content.sizing.clone();

                    frame.vertical.clamped_add(*n);
                }

                if edges.contains(&alignment::Edge::Left) {
                    frame.horizontal.clamped_add(*n);
                }
                if edges.contains(&alignment::Edge::Right) {
                    frame.horizontal.clamped_add(*n);
                }

                if frame.horizontal.min_content_size() > bounds.width {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.width = bounds.width.checked_sub(*n).unwrap_or(0);

                    resolved_content = node.resolve_size(&bounds);
                    frame = resolved_content.sizing.clone();

                    frame.horizontal.clamped_add(*n);
                }

                SizedLayout::new(SizedNode::Border(*n, *c, edges.clone(), resolved_content), frame)
            }

            VerticalStack(alignment, nodes) => {
                let mut result = sizing::ItemSizing { horizontal: Static(0), vertical: Static(0) };
                let mut resolved_children: Vec<SizedLayout> = vec![];

                for node in nodes {
                    let resolved_node = node.resolve_size(bounds);
                    let node_sizing = resolved_node.sizing.clone();
                    result.horizontal = match result.horizontal {
                        Static(j) => match node_sizing.horizontal {
                            Static(i) => Static(i.max(j)),
                            Greedy(i) => Greedy(i.max(j))
                        }
                        Greedy(j) => {
                            let i = node_sizing.horizontal.min_content_size();
                            Greedy(i.max(j))
                        }
                    };

                    result.vertical.clamped_accumulate(&node_sizing.vertical);
                    resolved_children.push(resolved_node);
                }

                SizedLayout::new(SizedNode::VerticalStack(alignment.clone(), resolved_children), result)
            }
            HorizontalStack(alignment, nodes) => {
                let mut result = sizing::ItemSizing { horizontal: Static(0), vertical: Static(0) };
                let mut resolved_children = vec![];

                for node in nodes {
                    let resolved_node = node.resolve_size(bounds);
                    let node_sizing = resolved_node.sizing.clone();
                    result.vertical = match result.vertical {
                        Static(j) => match node_sizing.vertical {
                            Static(i) => Static(i.max(j)),
                            Greedy(i) => Greedy(i.max(j))
                        }
                        Greedy(j) => {
                            let i = node_sizing.vertical.min_content_size();
                            Greedy(i.max(j))
                        }
                    };

                    result.horizontal.clamped_accumulate(&node_sizing.horizontal);

                    resolved_children.push(resolved_node);
                }

                SizedLayout::new(SizedNode::HorizontalStack(alignment.clone(), resolved_children), result)
            }
        }
    }
}

impl Layout {
    pub fn text(content: &str) -> Layout {
        Layout::Text(content.to_string())
    }

    pub fn center(self) -> Layout {
        Layout::VCenter(Box::new(Layout::HCenter(Box::new(self))))
    }

    pub fn center_vertically(self) -> Layout {
        Layout::VCenter(Box::new(self))
    }

    pub fn center_horizontally(self) -> Layout {
        Layout::HCenter(Box::new(self))
    }

    pub fn width(self, n: usize) -> Layout {
        Layout::Width(n, Box::new(self))
    }
    
    pub fn padding_top(self, n: usize) -> Layout {
        Layout::TopPadding(n, Box::new(self))
    }
    
    pub fn padding_bottom(self, n: usize) -> Layout {
        Layout::BottomPadding(n, Box::new(self))
    }

    pub fn padding_left(self, n: usize) -> Layout {
        Layout::LeftPadding(n, Box::new(self))
    }
    
    pub fn padding_right(self, n: usize) -> Layout {
        Layout::RightPadding(n, Box::new(self))
    }

    pub fn padding_horizontal(self, n: usize) -> Layout {
        self.padding_left(n).padding_right(n)
    }

    pub fn padding_vertical(self, n: usize) -> Layout {
        self.padding_top(n).padding_bottom(n)
    }

    pub fn padding(self, n: usize) -> Layout {
        self
            .padding_top(n)
            .padding_right(n)
            .padding_bottom(n)
            .padding_left(n)
    }

    pub fn align_right(self) -> Layout {
        Layout::HRightAlign(Box::new(self))
    }

    pub fn align_left(self) -> Layout {
        Layout::HLeftAlign(Box::new(self))
    }

    pub fn align_top(self) -> Layout {
        Layout::VTopAlign(Box::new(self))
    }

    pub fn align_bottom(self) -> Layout {
        Layout::VBottomAlign(Box::new(self))
    }

    pub fn border(self, n: usize, c: char, edges: HashSet<alignment::Edge>) -> Layout {
        Layout::Border(n, c, edges, Box::new(self))
    }

    pub fn background(self, c: char) -> Layout {
        Layout::Background(c, Box::new(self))
    }
}