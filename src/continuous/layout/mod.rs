use std::collections::HashSet;

pub mod sizing;
pub mod alignment;
pub mod geometry;

use geometry::Rect;
use super::color::Rgba as Pixel;

/// A node (and the tree itself) that recursively represents the desired UI
/// 
/// This enum describes the basic bulding blocks for a Textyle layout tree. It also stores a generic parameter
/// which you can define as the context of the layout.
/// 
/// You can use the provided methods to modify and build the layout:
/// 
/// ```rust
/// let mut layout: Layout<()> = Layout::Text("Hello!".to_string()) // The inner most node
///     .padding_horizontal(3) // modifies the node wrapping it around a series of Padding nodes, and returns a Layout
///     .background('*');
/// ```
#[derive(Clone)]
pub enum Layout<Ctx> {
    // /// A basic string
    // Text(String),

    /// Constraints the inner node to a specific horizontal space. Takes priority over greedy spacing.
    Width(usize, Box<Layout<Ctx>>),

    /// Constraints the inner node to a specific vertical space. Takes priority over greedy spacing.
    Height(usize, Box<Layout<Ctx>>),

    /// Adds empty spacing to the top of the inner node. If free space is not available in the container,
    /// the inner node will have to shrink to fit the padding.
    TopPadding(usize, Box<Layout<Ctx>>),
    
    /// Adds empty spacing to the right of the inner node. If free space is not available in the container,
    /// the inner node will have to shrink to fit the padding.
    RightPadding(usize, Box<Layout<Ctx>>),

    /// Adds empty spacing to the bottom of the inner node. If free space is not available in the container,
    /// the inner node will have to shrink to fit the padding.
    BottomPadding(usize, Box<Layout<Ctx>>),

    /// Adds empty spacing to the left of the inner node. If free space is not available in the container,
    /// the inner node will have to shrink to fit the padding.
    LeftPadding(usize, Box<Layout<Ctx>>),

    /// A container that expands vertically to the full available space,
    /// keeping the inner node centered without modifying it's regular size.
    VCenter(Box<Layout<Ctx>>),

    /// A container that expands horizontally to the full available space,
    /// keeping the inner node centered without modifying it's regular size.
    HCenter(Box<Layout<Ctx>>),

    /// A container that expands vertically to the full available space,
    /// keeping the inner node aligned to the bottom without it's regular size.
    VBottomAlign(Box<Layout<Ctx>>),

    /// A container that expands horizontally to the full available space,
    /// keeping the inner node aligned to the right without it's regular size.
    HRightAlign(Box<Layout<Ctx>>),

    /// A container that expands vertically to the full available space,
    /// keeping the inner node aligned to the top without it's regular size.
    VTopAlign(Box<Layout<Ctx>>),

    /// A container that expands horizontally to the full available space,
    /// keeping the inner node aligned to the left without it's regular size.
    HLeftAlign(Box<Layout<Ctx>>),

    /// Doesn't affect the layout of the inner node, but fills the empty spaces with the provided `char`
    Background(Pixel, Box<Layout<Ctx>>),

    /// Draws a border around the inner node, with a line width and a specific char. You can specify which edge to draw it on.
    /// Its spacing rules work exactly like `Layout::Padding`.
    Border(usize, Pixel, HashSet<alignment::Edge>, Box<Layout<Ctx>>),

    /// A container that composes nodes vertically, top to bottom. You can define the horizontal alignment and the spacing between elements.
    /// It occupies only the amount of space its nodes use.
    VerticalStack(alignment::HorizontalAlignment, usize, Vec<Layout<Ctx>>),

    /// A container that composes nodes horizontally, top to bottom. You can define the vertical alignment and the spacing between elements.
    /// It occupies only the amount of space its nodes use.
    HorizontalStack(alignment::VerticalAlignment, usize, Vec<Layout<Ctx>>),

    /// Provides a way to embed any text canvas into the current layout. It grows greedily.
    DrawCanvas(fn(&Ctx, &Rect)->crate::continuous::canvas::Canvas),

    /// Allows the rendered layout access to the current context. Useful when the application is keeping external state. It doesn't affect layout.
    WithContext(fn(&Ctx)->Layout<Ctx>)
}

/// A variant of the layout tree that contains a rough description of the UI's sizing information.
/// It calculates the minimum space that a node can take up, and if it will expand in any way to fill it's content.
#[derive(Clone)]
pub enum SizedNode<Ctx: Clone> {
    // Text(String),
    Width(usize, SizedLayout<Ctx>),
    Height(usize, SizedLayout<Ctx>),
    TopPadding(usize, SizedLayout<Ctx>),
    RightPadding(usize, SizedLayout<Ctx>),
    BottomPadding(usize, SizedLayout<Ctx>),
    LeftPadding(usize, SizedLayout<Ctx>),
    VCenter(SizedLayout<Ctx>),
    HCenter(SizedLayout<Ctx>),
    VBottomAlign(SizedLayout<Ctx>),
    HRightAlign(SizedLayout<Ctx>),
    VTopAlign(SizedLayout<Ctx>),
    HLeftAlign(SizedLayout<Ctx>),
    Background(Pixel, SizedLayout<Ctx>),
    Border(usize, Pixel, HashSet<alignment::Edge>, SizedLayout<Ctx>),

    VerticalStack(alignment::HorizontalAlignment, usize, Vec<SizedLayout<Ctx>>),
    HorizontalStack(alignment::VerticalAlignment, usize, Vec<SizedLayout<Ctx>>),

    DrawCanvas(fn(&Ctx, &Rect)->crate::continuous::canvas::Canvas)
}

#[derive(Clone)]
pub struct SizedLayout<Ctx: Clone> {
    pub node: Box<SizedNode<Ctx>>,
    pub sizing: sizing::ItemSizing
}

impl<Ctx: Clone> SizedLayout<Ctx> {
    fn new(node: SizedNode<Ctx>, sizing: sizing::ItemSizing) -> Self {
        SizedLayout { node: Box::new(node), sizing }
    }
}

impl<Ctx: Clone> Layout<Ctx> {
    pub fn resolve_size(&self, bounds: &Rect, context: &mut Ctx) -> SizedLayout<Ctx> {
        use Layout::*;
        use sizing::Sizing::*;

        match self {
            // Text(t) => {
            //     let lines = t.lines();

            //     let mut width = 0usize;
            //     let mut height = 0usize;
            //     for line in lines {
            //         let sz = self.calculate_line_size(line, bounds);
            //         if sz.width > width {
            //             width = sz.width;
            //         }

            //         height += sz.height;
            //     }

            //     let sizing = sizing::ItemSizing::new(Static(width), Static(height));

            //     SizedLayout::new(SizedNode::Text(t.clone()), sizing)
            // }
            VCenter(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing::new(content_size.horizontal, Greedy(min_height));

                SizedLayout::new(SizedNode::VCenter(resolved), sizing)
            }
            VBottomAlign(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VBottomAlign(resolved), sizing)
            }
            HCenter(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HCenter(resolved), sizing)
            }
            HRightAlign(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HRightAlign(resolved), sizing)
            }
            VTopAlign(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_height = content_size.vertical.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: content_size.horizontal, vertical: Greedy(min_height) };

                SizedLayout::new(SizedNode::VTopAlign(resolved), sizing)
            }
            HLeftAlign(node) => {
                let resolved = node.resolve_size(bounds, context);
                let content_size = resolved.sizing.clone();

                let min_width = content_size.horizontal.min_content_size();

                let sizing = sizing::ItemSizing { horizontal: Greedy(min_width), vertical: content_size.vertical };

                SizedLayout::new(SizedNode::HRightAlign(resolved), sizing)
            }
            Width(size, node) => {
                let mut bounds = bounds.clone();
                bounds.width = *size;

                let resolved_content = node.resolve_size(&bounds, context);
                let mut frame = resolved_content.sizing.clone();
                frame.horizontal = Static(*size);

                SizedLayout::new(SizedNode::Width(*size, resolved_content), frame)
            }
            Height(size, node) => {
                let mut bounds = bounds.clone();
                bounds.height = *size;

                let resolved_content = node.resolve_size(&bounds, context);
                let mut frame = resolved_content.sizing.clone();
                frame.vertical = Static(*size);

                SizedLayout::new(SizedNode::Height(*size, resolved_content), frame)
            }
            TopPadding(n, node) | BottomPadding(n, node) => {
                let resolved = node.resolve_size(bounds, context);
                let mut frame = resolved.sizing.clone();
                
                frame.vertical.clamped_add(*n);

                let make_node = |n: usize, node: SizedLayout<Ctx>|{
                    match self {
                        TopPadding(_, _) => SizedNode::TopPadding(n, node),
                        BottomPadding(_, _) => SizedNode::BottomPadding(n, node),
                        _ => unreachable!()
                    }
                };

                if frame.vertical.min_content_size() > bounds.height {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.height = bounds.height.saturating_sub(*n);

                    let resolved_content = node.resolve_size(&bounds, context);
                    let mut frame = resolved_content.sizing.clone();

                    frame.vertical.clamped_add(*n);

                    SizedLayout::new(make_node(*n, resolved_content), frame)
                } else {
                    SizedLayout::new(make_node(*n, resolved), frame)
                }
            }
            LeftPadding(n, node) | RightPadding(n, node) => {
                let resolved = node.resolve_size(bounds, context);
                let mut frame = resolved.sizing.clone();

                let make_node = |n: usize, node: SizedLayout<Ctx>|{
                    match self {
                        LeftPadding(_, _) => SizedNode::LeftPadding(n, node),
                        RightPadding(_, _) => SizedNode::RightPadding(n, node),
                        _ => unreachable!()
                    }
                };
                
                frame.horizontal.clamped_add(*n);
                if frame.horizontal.min_content_size() > bounds.width {
                    // recalculate with less space
                    let mut bounds = bounds.clone();
                    bounds.width = bounds.width.saturating_sub(*n);

                    let resolved_content = node.resolve_size(&bounds, context);
                    frame = resolved_content.sizing.clone();
                    frame.horizontal.clamped_add(*n);

                    let node = make_node(*n, resolved_content);

                    SizedLayout::new(node, frame)
                } else {
                    SizedLayout::new(make_node(*n, resolved), frame)
                }
            }
            Background(c, node) => {
                let resolved_content = node.resolve_size(bounds, context);
                let frame = resolved_content.sizing.clone();

                SizedLayout::new(SizedNode::Background(*c, resolved_content), frame)
            }
            Border(n, c, edges, node) => {
                let outer_bounds = bounds;
                let mut resolved_content = node.resolve_size(outer_bounds, context);
                let mut frame = resolved_content.sizing.clone();
                
                if edges.contains(&alignment::Edge::Top) {
                    frame.vertical.clamped_add(*n);
                }
                if edges.contains(&alignment::Edge::Bottom) {
                    frame.vertical.clamped_add(*n);
                }

                if frame.vertical.min_content_size() > outer_bounds.height {
                    // recalculate with less space
                    let mut bounds = outer_bounds.clone();
                    bounds.height = bounds.height.saturating_sub(*n);

                    resolved_content = node.resolve_size(&bounds, context);
                    frame = resolved_content.sizing.clone();

                    frame.vertical.clamped_add(*n);
                }

                if edges.contains(&alignment::Edge::Left) {
                    frame.horizontal.clamped_add(*n);
                }
                if edges.contains(&alignment::Edge::Right) {
                    frame.horizontal.clamped_add(*n);
                }

                if frame.horizontal.min_content_size() > outer_bounds.width {
                    // recalculate with less space
                    let mut bounds = outer_bounds.clone();
                    bounds.width = bounds.width.saturating_sub(*n);

                    resolved_content = node.resolve_size(&bounds, context);
                    frame = resolved_content.sizing.clone();

                    frame.horizontal.clamped_add(*n);
                }

                SizedLayout::new(SizedNode::Border(*n, *c, edges.clone(), resolved_content), frame)
            }

            VerticalStack(alignment, spacing,  nodes) => {
                let spacing_sizing = spacing * nodes.len().saturating_sub(1);
                let mut result = sizing::ItemSizing { horizontal: Static(0), vertical: Static(spacing_sizing) };
                let mut bounds = bounds.clone();
                bounds.height -= spacing_sizing;
                let mut resolved_children: Vec<SizedLayout<_>> = vec![];

                for node in nodes {
                    let resolved_node = node.resolve_size(&bounds, context);
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

                SizedLayout::new(SizedNode::VerticalStack(alignment.clone(), *spacing, resolved_children), result)
            }
            HorizontalStack(alignment, spacing, nodes) => {
                let spacing_sizing = spacing * nodes.len().saturating_sub(1);
                let mut result = sizing::ItemSizing { horizontal: Static(spacing_sizing), vertical: Static(0) };let mut bounds = bounds.clone();
                bounds.width -= spacing_sizing;

                let mut resolved_children = vec![];

                for node in nodes {
                    let resolved_node = node.resolve_size(&bounds, context);
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

                SizedLayout::new(SizedNode::HorizontalStack(alignment.clone(), *spacing, resolved_children), result)
            }
            DrawCanvas(action) => {
                SizedLayout::new(
                    SizedNode::DrawCanvas(*action),
                    sizing::ItemSizing::new(
                        sizing::Sizing::Greedy(1),
                        sizing::Sizing::Greedy(1)
                    )
                )
            },
            WithContext(node) => {
                let node = node(context);

                node.resolve_size(bounds, context)
            }
        }
    }
}

impl<Ctx: Clone> Layout<Ctx> {
    // pub fn text(content: &str) -> Layout<Ctx> {
    //     Layout::Text(content.to_string())
    // }

    pub fn center(self) -> Layout<Ctx> {
        Layout::VCenter(Box::new(Layout::HCenter(Box::new(self))))
    }

    pub fn center_vertically(self) -> Layout<Ctx> {
        Layout::VCenter(Box::new(self))
    }

    pub fn center_horizontally(self) -> Layout<Ctx> {
        Layout::HCenter(Box::new(self))
    }

    pub fn width(self, n: usize) -> Layout<Ctx> {
        Layout::Width(n, Box::new(self))
    }
    
    pub fn height(self, n: usize) -> Layout<Ctx> {
        Layout::Height(n, Box::new(self))
    }
    
    pub fn padding_top(self, n: usize) -> Layout<Ctx> {
        Layout::TopPadding(n, Box::new(self))
    }
    
    pub fn padding_bottom(self, n: usize) -> Layout<Ctx> {
        Layout::BottomPadding(n, Box::new(self))
    }

    pub fn padding_left(self, n: usize) -> Layout<Ctx> {
        Layout::LeftPadding(n, Box::new(self))
    }
    
    pub fn padding_right(self, n: usize) -> Layout<Ctx> {
        Layout::RightPadding(n, Box::new(self))
    }

    pub fn padding_horizontal(self, n: usize) -> Layout<Ctx> {
        self.padding_left(n).padding_right(n)
    }

    pub fn padding_vertical(self, n: usize) -> Layout<Ctx> {
        self.padding_top(n).padding_bottom(n)
    }

    pub fn padding(self, n: usize) -> Layout<Ctx> {
        self
            .padding_top(n)
            .padding_right(n)
            .padding_bottom(n)
            .padding_left(n)
    }

    pub fn align_right(self) -> Layout<Ctx> {
        Layout::HRightAlign(Box::new(self))
    }

    pub fn align_left(self) -> Layout<Ctx> {
        Layout::HLeftAlign(Box::new(self))
    }

    pub fn align_top(self) -> Layout<Ctx> {
        Layout::VTopAlign(Box::new(self))
    }

    pub fn align_bottom(self) -> Layout<Ctx> {
        Layout::VBottomAlign(Box::new(self))
    }

    pub fn border(self, n: usize, c: Pixel, edges: HashSet<alignment::Edge>) -> Layout<Ctx> {
        Layout::Border(n, c, edges, Box::new(self))
    }

    pub fn background(self, c: Pixel) -> Layout<Ctx> {
        Layout::Background(c, Box::new(self))
    }

    pub fn vertical_stack(nodes: Vec<Layout<Ctx>>) -> Layout<Ctx> {
        Layout::VerticalStack(alignment::HorizontalAlignment::Center, 0, nodes)
    }
    
    pub fn horizontal_stack(nodes: Vec<Layout<Ctx>>) -> Layout<Ctx> {
        Layout::HorizontalStack(alignment::VerticalAlignment::Center, 0, nodes)
    }

    pub fn grid<State, Item: Clone>(items: &geometry::Matrix<Item>, spacing: usize, view: fn(&Item)->Layout<Ctx>) -> Layout<Ctx> {
        let mut rows = vec![];

        let mut row = vec![];
        let mut col_counter = 0;
        for item in items.data() {
            col_counter += 1;

            let cell = view(item).center();
            row.push(cell);

            if col_counter == items.shape().0 {
                rows.push(Layout::HorizontalStack(alignment::VerticalAlignment::Center, spacing, row));
                row = vec![];
                col_counter = 0;
            }
        }

        Layout::VerticalStack(alignment::HorizontalAlignment::Center, spacing, rows)
    }
}