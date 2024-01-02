use std::{io::Write, collections::HashSet};

use unicode_segmentation::UnicodeSegmentation;

// Taken from https://docs.rs/map-macro/latest/src/map_macro/lib.rs.html#140-144
macro_rules! hash_set {
    {$($v: expr),* $(,)?} => {
        std::collections::HashSet::from([$($v,)*])
    };
}

#[derive(Clone, Debug)]
struct Rect {
    x: i64,
    y: i64,
    width: usize,
    height: usize,
}

impl Rect {
    fn new(x: i64, y: i64, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn zero() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    fn sized(width: usize, height: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    // Utilities
    fn max_x(&self) -> i64 {
        self.x + self.width as i64
    }

    fn max_y(&self) -> i64 {
        self.y + self.height as i64
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}

struct Canvas {
    bounds: Rect,
    contents: Vec<String>,
}

impl Canvas {
    fn new() -> Self {
        Canvas {
            bounds: Rect::zero(),
            contents: Vec::new(),
        }
    }

    fn create_in_bounds(bounds: &Rect) -> Self {
        Canvas {
            bounds: bounds.clone(),
            contents: vec![" ".to_string(); bounds.width * bounds.height],
        }
    }

    fn create(width: usize, height: usize) -> Self {
        Canvas {
            bounds: Rect::sized(width, height),
            contents: vec![" ".to_string(); width * height],
        }
    }

    fn write(&mut self, grapheme: &str, x: usize, y: usize) {
        if x >= self.bounds.width || y >= self.bounds.height { return; }

        let index = y * self.bounds.width + x;

        self.contents[index] = grapheme.to_string();
    }

    fn draw_rect(&mut self, bounds: &Rect, grapheme: &str) {
        for x in bounds.x..(bounds.x + bounds.width as i64) {
            for y in bounds.y..(bounds.y + bounds.height as i64) {
                if x < 0 || x >= self.bounds.width as i64 { continue; }
                if y < 0 || y >= self.bounds.height as i64 { continue; }

                let i = y * (self.bounds.width as i64) + x;
                self.contents[i as usize] = grapheme.to_string();
            }
        }
    }

    fn clear_with(&mut self, grapheme: &str) {
        self.draw_rect(&self.bounds.clone(), grapheme);
    }
}

#[derive(Clone, Debug)]
enum Sizing {
    Greedy(usize),
    Static(usize)
}

impl Sizing {
    fn clamped_accumulate(&mut self, other: &Sizing) {
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

    fn clamped_add(&mut self, n: usize) {
        match self {
            Sizing::Static(sz) | Sizing::Greedy(sz) => {
                *sz = sz.checked_add(n).unwrap_or(*sz);
            }
        };
    }

    fn min_content_size(&self) -> usize {
        match self {
            Sizing::Static(sz) | Sizing::Greedy(sz) => *sz
        }
    }
}

#[derive(Clone, Debug)]
struct ItemSizing {
    horizontal: Sizing,
    vertical: Sizing
}

impl ItemSizing {
    fn fit_into(&self, bounds: &Rect) -> Rect {
        let width = match self.horizontal {
            Sizing::Greedy(n) => bounds.width.max(n),
            Sizing::Static(n) => n
        };

        let height = match self.vertical {
            Sizing::Greedy(n) => bounds.height.max(n),
            Sizing::Static(n) => n
        };

        Rect::new(
            bounds.x,
            bounds.y,
            width,
            height
        )
    }
}

#[derive(Clone, Debug)]
enum HorizontalAlignment {
    Left,
    Center,
    Right
}

#[derive(Clone, Debug)]
enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

#[derive(Debug, Clone, std::hash::Hash, PartialEq, Eq)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left
}

impl Edge {
    fn all() -> HashSet<Edge> {
        hash_set!(Edge::Top, Edge::Right, Edge::Bottom, Edge::Left)
    }

    fn horizontal() -> HashSet<Edge> {
        hash_set!(Edge::Right, Edge::Left)
    }

    fn vertical() -> HashSet<Edge> {
        hash_set!(Edge::Top, Edge::Bottom)
    }
}

#[derive(Clone)]
enum Layout {
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
    Border(usize, char, HashSet<Edge>, Box<Layout>),

    VerticalStack(HorizontalAlignment, Vec<Layout>),
    HorizontalStack(VerticalAlignment, Vec<Layout>),
}

#[derive(Clone, Debug)]
enum SizedNode {
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
    Border(usize, char, HashSet<Edge>, SizedLayout),

    VerticalStack(HorizontalAlignment, Vec<SizedLayout>),
    HorizontalStack(VerticalAlignment, Vec<SizedLayout>),
}

#[derive(Clone, Debug)]
struct SizedLayout {
    node: Box<SizedNode>,
    sizing: ItemSizing
}

impl SizedLayout {
    fn new(node: SizedNode, sizing: ItemSizing) -> Self {
        SizedLayout { node: Box::new(node), sizing }
    }
}

impl Layout {
    fn calculate_line_size(&self, line: &str, bounds: &Rect) -> Rect {
        let graphemes = line.graphemes(true).collect::<Vec<_>>();
        let rows = ((graphemes.len() as f64) / (bounds.width as f64)).ceil() as usize;

        if rows < 2 {
            Rect::sized(graphemes.len(), 1)
        } else {
            Rect::sized(bounds.width, rows)
        }
    }

    fn resolve_size(&self, bounds: &Rect) -> SizedLayout {
        use Layout::*;
        use Sizing::*;

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

                let sizing = ItemSizing{ horizontal: Static(width),  vertical: Static(height) };

                SizedLayout::new(SizedNode::Text(t.clone()), sizing)
            }
            VCenter(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VCenter(resolved), sizing)
            }
            VBottomAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VBottomAlign(resolved), sizing)
            }
            HCenter(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HCenter(resolved), sizing)
            }
            HRightAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HRightAlign(resolved), sizing)
            }
            VTopAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VTopAlign(resolved), sizing)
            }
            HLeftAlign(node) => {
                let resolved = node.resolve_size(bounds);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

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
                
                if edges.contains(&Edge::Top) {
                    frame.vertical.clamped_add(*n);
                }
                if edges.contains(&Edge::Bottom) {
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

                if edges.contains(&Edge::Left) {
                    frame.horizontal.clamped_add(*n);
                }
                if edges.contains(&Edge::Right) {
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
                let mut result = ItemSizing { horizontal: Static(0), vertical: Static(0) };
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
                let mut result = ItemSizing { horizontal: Static(0), vertical: Static(0) };
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

    fn text(content: &str) -> Layout {
        Layout::Text(content.to_string())
    }

    fn center(self) -> Layout {
        Layout::VCenter(Box::new(Layout::HCenter(Box::new(self))))
    }

    fn center_vertically(self) -> Layout {
        Layout::VCenter(Box::new(self))
    }

    fn center_horizontally(self) -> Layout {
        Layout::HCenter(Box::new(self))
    }

    fn width(self, n: usize) -> Layout {
        Layout::Width(n, Box::new(self))
    }
    
    fn padding_top(self, n: usize) -> Layout {
        Layout::TopPadding(n, Box::new(self))
    }
    
    fn padding_bottom(self, n: usize) -> Layout {
        Layout::BottomPadding(n, Box::new(self))
    }

    fn padding_left(self, n: usize) -> Layout {
        Layout::LeftPadding(n, Box::new(self))
    }
    
    fn padding_right(self, n: usize) -> Layout {
        Layout::RightPadding(n, Box::new(self))
    }

    fn padding_horizontal(self, n: usize) -> Layout {
        self.padding_left(n).padding_right(n)
    }

    fn padding_vertical(self, n: usize) -> Layout {
        self.padding_top(n).padding_bottom(n)
    }

    fn padding(self, n: usize) -> Layout {
        self
            .padding_top(n)
            .padding_right(n)
            .padding_bottom(n)
            .padding_left(n)
    }

    fn align_right(self) -> Layout {
        Layout::HRightAlign(Box::new(self))
    }

    fn align_left(self) -> Layout {
        Layout::HLeftAlign(Box::new(self))
    }

    fn align_top(self) -> Layout {
        Layout::VTopAlign(Box::new(self))
    }

    fn align_bottom(self) -> Layout {
        Layout::VBottomAlign(Box::new(self))
    }

    fn border(self, n: usize, c: char, edges: HashSet<Edge>) -> Layout {
        Layout::Border(n, c, edges, Box::new(self))
    }

    fn background(self, c: char) -> Layout {
        Layout::Background(c, Box::new(self))
    }
}

impl Canvas {
    fn render(&mut self, layout: &SizedLayout, bounds: &Rect) {
        use SizedNode::*;
        let layout = layout.clone();

        match *layout.node {
            Text(content) => {
                let graphemes = content.graphemes(true).collect::<Vec<_>>();
                let mut x = bounds.x as usize;
                let mut y = bounds.y as usize;
                for g in graphemes {
                    if g == "\n" {
                        y += 1;
                        x = bounds.x as usize;
                        continue;
                    } else if g == " " {
                        // Don't write anything
                    } else {
                        self.write(g, x, y);
                    }

                    x += 1;
                    if (x - bounds.x as usize) >= bounds.width {
                        y += 1;
                        x = bounds.x as usize;
                    }
                }
            }
            Width(_, node) | Height(_, node) => {
                let frame = node.sizing.fit_into(bounds);

                self.render(&node, &frame);
            }
            VCenter(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let center_pos = bounds.y as usize + bounds.height / 2;
                let center_start = center_pos - content_rect.height / 2;
                content_rect.y = center_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.render(&n, &content_bounds);
            }
            HCenter(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let center_pos = bounds.x as usize + bounds.width / 2;
                let center_start = center_pos - content_rect.width / 2;
                content_rect.x = center_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.render(&n, &content_bounds);
            }
            VBottomAlign(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let bottom_most = bounds.y as usize + bounds.height;
                let top_start = bottom_most - content_rect.height;
                content_rect.y = top_start as i64;

                self.render(&n, &content_rect);
            }
            HRightAlign(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let right_most = bounds.x as usize + bounds.width;
                let left_start = right_most - content_rect.width;
                content_rect.x = left_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.render(&n, &content_bounds);
            }
            VTopAlign(n) | HLeftAlign(n) => {
                let content_rect = n.sizing.fit_into(bounds);

                self.render(&n, &content_rect);
            }
            TopPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.height = bounds.height.checked_sub(n).unwrap_or(0);
                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x;
                frame.y = bounds.y + n as i64;

                self.render(&node, &frame);
            }
            RightPadding(n, node) => {
                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                let free_width = bounds.width.checked_sub(n).unwrap_or(0);
                let adjustment = frame.width.checked_sub(free_width).unwrap_or(0);

                frame.width = frame.width.checked_sub(adjustment).unwrap_or(0);

                self.render(&node, &frame);
            }
            BottomPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.height = bounds.height.checked_sub(n).unwrap_or(0);

                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                self.render(&node, &frame);
            }
            LeftPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.width = bounds.width.checked_sub(n).unwrap_or(0);
                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x + n as i64;
                frame.y = bounds.y;

                self.render(&node, &frame);
            }
            Background(c, node) => {
                let mut frame = node.sizing.fit_into(bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                self.draw_rect(bounds, &c.to_string());

                self.render(&node, &frame);
            }
            Border(n, c, edges, node) => {
                let outer_bounds = bounds;
                let mut inner_bounds = bounds.clone();
                if edges.contains(&Edge::Top) {
                    inner_bounds.height = inner_bounds.height.checked_sub(n).unwrap_or(0);
                    inner_bounds.y = inner_bounds.y.checked_add(n as i64).unwrap_or(0);

                    let line_bounds = Rect::new(outer_bounds.x, outer_bounds.y, outer_bounds.width, n);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&Edge::Right) {
                    inner_bounds.width = inner_bounds.width.checked_sub(n).unwrap_or(0);

                    let line_bounds = Rect::new(outer_bounds.max_x() - n as i64, outer_bounds.y, n, outer_bounds.height);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&Edge::Bottom) {
                    inner_bounds.height = inner_bounds.height.checked_sub(n).unwrap_or(0);

                    let line_bounds = Rect::new(outer_bounds.x, outer_bounds.max_y() - n as i64, outer_bounds.width, n);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&Edge::Left) {
                    inner_bounds.width = inner_bounds.width.checked_sub(n).unwrap_or(0);
                    inner_bounds.x = inner_bounds.x.checked_add(n as i64).unwrap_or(0);

                    let line_bounds = Rect::new(outer_bounds.x, outer_bounds.y, n, outer_bounds.height);
                    self.draw_rect(&line_bounds, &c.to_string())
                }

                let mut frame = node.sizing.fit_into(&inner_bounds);
                frame.x = inner_bounds.x;
                frame.y = inner_bounds.y;

                self.render(&node, &frame);
            }
            VerticalStack(alignment, nodes) => {
                let mut max_width = 0usize;

                let mut last_bounds = Rect::new(0, 0, 0, 0);

                let mut greedy_count = 0;
                let mut static_height = 0usize;

                for node in &nodes {
                    if let Sizing::Static(n) = node.sizing.vertical {
                        static_height += n;
                    } else {
                        greedy_count += 1;
                    }
                }

                let mut greedy_space = bounds.height - static_height;
                let greedy_size = if greedy_count != 0 { greedy_space / greedy_count } else { 0 };

                let mut new_nodes = vec![];

                for node in &nodes {
                    let mut n = node.clone();
                    n.sizing.vertical = match n.sizing.vertical {
                        Sizing::Static(sz) => Sizing::Static(sz),
                        Sizing::Greedy(tight) => {
                            greedy_space -= greedy_size;
                            let mut node_height = greedy_size;
                            if greedy_space < greedy_size {
                                node_height += greedy_space;
                                greedy_space = 0;
                            }

                            Sizing::Static(node_height.max(tight))
                        }
                    };

                    new_nodes.push(n);
                }

                let nodes = new_nodes;

                let mut raw_bounds = vec![];
                for node in &nodes {
                    let size = node.sizing.fit_into(bounds);

                    let node_bounds = Rect::new(0, last_bounds.max_y(), size.width, size.height);
                    last_bounds = node_bounds.clone();

                    if node_bounds.width > max_width {
                        max_width = node_bounds.width;
                    }

                    raw_bounds.push(node_bounds);
                }

                let final_bounds: Vec<_> = raw_bounds.into_iter().map(|mut bound| {
                    match &alignment {
                        HorizontalAlignment::Left => { /* Already aligned to the left */}
                        HorizontalAlignment::Center => {
                            let center = max_width / 2;
                            let start = center - bound.width/2;
                            bound.x = start as i64;
                        }
                        HorizontalAlignment::Right => {
                            let right = max_width;
                            let start = right - bound.width;
                            bound.x = start as i64;
                        }
                    }

                    // move from 0 based bounds to the actual frame of the container
                    bound.x += bounds.x;
                    bound.y += bounds.y;

                    bound
                }).collect();

                for i in 0..nodes.len() {
                    let node = nodes[i].clone();
                    let size = &final_bounds[i];

                    self.render(&node, size);
                }
            }
            HorizontalStack(alignment, nodes) => {
                let mut max_height = 0usize;

                let mut last_bounds = Rect::new(0, 0, 0, 0);

                let mut greedy_count = 0;
                let mut static_width = 0usize;

                for node in &nodes {
                    if let Sizing::Static(n) = node.sizing.horizontal {
                        static_width += n;
                    } else {
                        greedy_count += 1;
                    }
                }

                let mut greedy_space = bounds.width.checked_sub(static_width).unwrap_or(0);
                let greedy_size = if greedy_count != 0 { greedy_space / greedy_count } else { 0 };

                let mut new_nodes = vec![];

                for node in &nodes {
                    let mut n = node.clone();
                    n.sizing.horizontal = match n.sizing.horizontal {
                        Sizing::Static(sz) => Sizing::Static(sz),
                        Sizing::Greedy(tight) => {
                            greedy_space -= greedy_size;
                            let mut node_width = greedy_size;
                            if greedy_space < greedy_size {
                                node_width += greedy_space;
                                greedy_space = 0;
                            }

                            Sizing::Static(node_width.max(tight))
                        }
                    };

                    new_nodes.push(n);
                }

                let nodes = new_nodes;

                let mut raw_bounds = vec![];
                for node in &nodes {
                    let size = node.sizing.fit_into(bounds);

                    let node_bounds = Rect::new(last_bounds.max_x(), 0, size.width, size.height);
                    last_bounds = node_bounds.clone();

                    if node_bounds.height > max_height {
                        max_height = node_bounds.height;
                    }

                    raw_bounds.push(node_bounds);
                }

                let final_bounds: Vec<_> = raw_bounds.into_iter().map(|mut bound| {
                    match &alignment {
                        VerticalAlignment::Top => { /* Already aligned to the top */}
                        VerticalAlignment::Center => {
                            let center = max_height / 2;
                            let start = center - bound.height/2;
                            bound.y = start as i64;
                        }
                        VerticalAlignment::Bottom => {
                            let bottom = max_height;
                            let start = bottom - bound.height;
                            bound.y = start as i64;
                        }
                    }

                    // move from 0 based bounds to the actual frame of the container
                    bound.x += bounds.x;
                    bound.y += bounds.y;

                    bound
                }).collect();

                for i in 0..nodes.len() {
                    let node = nodes[i].clone();
                    let size = &final_bounds[i];

                    self.render(&node, size);
                }
            }
        }
    }

    fn render_layout(&mut self, layout: &Layout) {
        let layout = layout.resolve_size(&self.bounds);
        let bounds = layout.sizing.fit_into(&self.bounds);

        self.render(&layout, &bounds);
    }
}

fn print_canvas(canvas: &Canvas) {
    let chars = canvas.contents.clone();
    let mut stdout = std::io::stdout();
    for n in 0..chars.len() {
        let c = &chars[n];
        let _ = crossterm::queue!(stdout, crossterm::style::Print(format!("{c}")));

        if n < chars.len()-1 && (n + 1) % canvas.bounds.width == 0 {
            let _ = crossterm::queue!(stdout, crossterm::style::Print("\n".to_string()));
        }
    }

    let _ = stdout.flush();
}

fn main() {
    let term_size = crossterm::terminal::window_size().unwrap();
    let mut terminal_columns = term_size.columns as usize;
    let mut terminal_rows = term_size.rows as usize;
    let layout = Layout::HorizontalStack(VerticalAlignment::Top, vec![
        Layout::text("Main content")
            .center_horizontally()
            .align_top()
            .padding_vertical(2)
            .border(2, '.', hash_set!(Edge::Right)),
        Layout::VerticalStack(HorizontalAlignment::Center, vec![
            Layout::text("Side content"),
            Layout::VerticalStack(HorizontalAlignment::Left, vec![
                Layout::text("List of content:")
                .padding(1),
                Layout::text("- Item 1"),
                Layout::text("- Item 2"),
                Layout::text("- Item 3"),
            ])
            .border(1, '-', hash_set![Edge::Top])
        ])
        .center_horizontally()
        .width(24)
        .padding_vertical(2)
    ]);

    let bounds = &Rect::sized(terminal_columns, terminal_rows);
    // let bounds = &Rect::sized(20, 5);
    let mut canvas = Canvas::create_in_bounds(bounds);

    // canvas.clear_with(".");

    canvas.render_layout(&layout);

    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).unwrap();
    loop {
        print_canvas(&canvas);
        
        let _waiting = match crossterm::event::read() {
            Ok(event) => {
                if let crossterm::event::Event::Key(k) = event {
                    if k.code == crossterm::event::KeyCode::Esc {
                        break;
                    }
                } else if let crossterm::event::Event::Resize(columns, rows) = event {
                    terminal_columns = columns as usize;
                    terminal_rows = rows as usize;

                    let bounds = &Rect::sized(terminal_columns, terminal_rows);
                    canvas = Canvas::create_in_bounds(bounds);

                    canvas.render_layout(&layout);
                }
            }
            Err(err) => {
                println!("{err:?}");
                break;
            }
        };
        
        crossterm::execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge)).unwrap();
    }

    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
}
