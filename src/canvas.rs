use crate::layout;

pub struct TextCanvas {
    bounds: layout::Rect,
    contents: Vec<String>,
}

impl TextCanvas {
    pub fn new() -> Self {
        TextCanvas {
            bounds: layout::Rect::zero(),
            contents: Vec::new(),
        }
    }

    pub fn create_in_bounds(bounds: &layout::Rect) -> Self {
        TextCanvas {
            bounds: bounds.clone(),
            contents: vec![" ".to_string(); bounds.width * bounds.height],
        }
    }

    pub fn create(width: usize, height: usize) -> Self {
        TextCanvas {
            bounds: layout::Rect::sized(width, height),
            contents: vec![" ".to_string(); width * height],
        }
    }

    fn write(&mut self, grapheme: &str, x: usize, y: usize) {
        if x >= self.bounds.width || y >= self.bounds.height { return; }

        let index = y * self.bounds.width + x;

        self.contents[index] = grapheme.to_string();
    }

    fn draw_rect(&mut self, bounds: &layout::Rect, grapheme: &str) {
        for x in bounds.x..(bounds.x + bounds.width as i64) {
            for y in bounds.y..(bounds.y + bounds.height as i64) {
                if x < 0 || x >= self.bounds.width as i64 { continue; }
                if y < 0 || y >= self.bounds.height as i64 { continue; }

                self.write(grapheme, x as usize, y as usize);
            }
        }
    }

    pub fn clear_with(&mut self, grapheme: &str) {
        self.draw_rect(&self.bounds.clone(), grapheme);
    }
}

impl TextCanvas {
    fn render(&mut self, layout: &layout::SizedLayout, bounds: &layout::Rect) {
        use layout::SizedNode::*;
        let layout = layout.clone();
        use unicode_segmentation::UnicodeSegmentation;

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
                if edges.contains(&layout::alignment::Edge::Top) {
                    inner_bounds.height = inner_bounds.height.checked_sub(n).unwrap_or(0);
                    inner_bounds.y = inner_bounds.y.checked_add(n as i64).unwrap_or(0);

                    let line_bounds = layout::Rect::new(outer_bounds.x, outer_bounds.y, outer_bounds.width, n);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&layout::alignment::Edge::Right) {
                    inner_bounds.width = inner_bounds.width.checked_sub(n).unwrap_or(0);

                    let line_bounds = layout::Rect::new(outer_bounds.max_x() - n as i64, outer_bounds.y, n, outer_bounds.height);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&layout::alignment::Edge::Bottom) {
                    inner_bounds.height = inner_bounds.height.checked_sub(n).unwrap_or(0);

                    let line_bounds = layout::Rect::new(outer_bounds.x, outer_bounds.max_y() - n as i64, outer_bounds.width, n);
                    self.draw_rect(&line_bounds, &c.to_string())
                }
                if edges.contains(&layout::alignment::Edge::Left) {
                    inner_bounds.width = inner_bounds.width.checked_sub(n).unwrap_or(0);
                    inner_bounds.x = inner_bounds.x.checked_add(n as i64).unwrap_or(0);

                    let line_bounds = layout::Rect::new(outer_bounds.x, outer_bounds.y, n, outer_bounds.height);
                    self.draw_rect(&line_bounds, &c.to_string())
                }

                let mut frame = node.sizing.fit_into(&inner_bounds);
                frame.x = inner_bounds.x;
                frame.y = inner_bounds.y;

                self.render(&node, &frame);
            }
            VerticalStack(alignment, nodes) => {
                let mut max_width = 0usize;

                let mut last_bounds = layout::Rect::new(0, 0, 0, 0);

                let mut greedy_count = 0;
                let mut static_height = 0usize;

                for node in &nodes {
                    if let layout::sizing::Sizing::Static(n) = node.sizing.vertical {
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
                        layout::sizing::Sizing::Static(sz) => layout::sizing::Sizing::Static(sz),
                        layout::sizing::Sizing::Greedy(tight) => {
                            greedy_space -= greedy_size;
                            let mut node_height = greedy_size;
                            if greedy_space < greedy_size {
                                node_height += greedy_space;
                                greedy_space = 0;
                            }

                            layout::sizing::Sizing::Static(node_height.max(tight))
                        }
                    };

                    new_nodes.push(n);
                }

                let nodes = new_nodes;

                let mut raw_bounds = vec![];
                for node in &nodes {
                    let size = node.sizing.fit_into(bounds);

                    let node_bounds = layout::Rect::new(0, last_bounds.max_y(), size.width, size.height);
                    last_bounds = node_bounds.clone();

                    if node_bounds.width > max_width {
                        max_width = node_bounds.width;
                    }

                    raw_bounds.push(node_bounds);
                }

                let final_bounds: Vec<_> = raw_bounds.into_iter().map(|mut bound| {
                    match &alignment {
                        layout::alignment::HorizontalAlignment::Left => { /* Already aligned to the left */}
                        layout::alignment::HorizontalAlignment::Center => {
                            let center = max_width / 2;
                            let start = center - bound.width/2;
                            bound.x = start as i64;
                        }
                        layout::alignment::HorizontalAlignment::Right => {
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

                let mut last_bounds = layout::Rect::new(0, 0, 0, 0);

                let mut greedy_count = 0;
                let mut static_width = 0usize;

                for node in &nodes {
                    if let layout::sizing::Sizing::Static(n) = node.sizing.horizontal {
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
                        layout::sizing::Sizing::Static(sz) => layout::sizing::Sizing::Static(sz),
                        layout::sizing::Sizing::Greedy(tight) => {
                            greedy_space -= greedy_size;
                            let mut node_width = greedy_size;
                            if greedy_space < greedy_size {
                                node_width += greedy_space;
                                greedy_space = 0;
                            }

                            layout::sizing::Sizing::Static(node_width.max(tight))
                        }
                    };

                    new_nodes.push(n);
                }

                let nodes = new_nodes;

                let mut raw_bounds = vec![];
                for node in &nodes {
                    let size = node.sizing.fit_into(bounds);

                    let node_bounds = layout::Rect::new(last_bounds.max_x(), 0, size.width, size.height);
                    last_bounds = node_bounds.clone();

                    if node_bounds.height > max_height {
                        max_height = node_bounds.height;
                    }

                    raw_bounds.push(node_bounds);
                }

                let final_bounds: Vec<_> = raw_bounds.into_iter().map(|mut bound| {
                    match &alignment {
                        layout::alignment::VerticalAlignment::Top => { /* Already aligned to the top */}
                        layout::alignment::VerticalAlignment::Center => {
                            let center = max_height / 2;
                            let start = center - bound.height/2;
                            bound.y = start as i64;
                        }
                        layout::alignment::VerticalAlignment::Bottom => {
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
    
    pub fn render_layout(&mut self, layout: &layout::Layout) {
        let layout = layout.resolve_size(&self.bounds);
        let bounds = layout.sizing.fit_into(&self.bounds);

        self.render(&layout, &bounds);
    }

    pub fn print_canvas(self: &TextCanvas) {
        use std::io::Write;
        let chars = self.contents.clone();
        let mut stdout = std::io::stdout();
        for n in 0..chars.len() {
            let c = &chars[n];
            let _ = crossterm::queue!(stdout, crossterm::style::Print(format!("{c}")));
    
            if n < chars.len()-1 && (n + 1) % self.bounds.width == 0 {
                let _ = crossterm::queue!(stdout, crossterm::style::Print("\n".to_string()));
            }
        }
    
        let _ = stdout.flush();
    }
}