use crate::layout::{self, geometry::{Rect, Vector}, SizedLayout};

pub enum DrawCommand {
    Text(Rect, String),
    Rect(Rect, String),
    Line(Vector, Vector, String),
}

impl<Ctx: Clone> SizedLayout<Ctx> {
    fn resolve_draw_commands(&self, bounds: &Rect, context: &mut Ctx) -> Vec<DrawCommand> {
        use layout::SizedNode::*;
        let layout = self.clone();
        use unicode_segmentation::UnicodeSegmentation;

        match *layout.node {
            Text(content) => {
                let graphemes = content.graphemes(true).collect::<Vec<_>>();
                let mut x = bounds.x as usize;
                let mut y = bounds.y as usize;
                
                vec![DrawCommand::Text(bounds.clone(), content)]
            }
            Width(_, node) | Height(_, node) => {
                let frame = node.sizing.fit_into(bounds);

                node.resolve_draw_commands(&frame, context)
            }
            VCenter(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let center_pos = bounds.y as usize + bounds.height / 2;
                let center_start = center_pos - content_rect.height / 2;
                content_rect.y = center_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.resolve_draw_commands(&content_bounds, context)
            }
            HCenter(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let center_pos = bounds.x as usize + bounds.width / 2;
                let center_start = center_pos - content_rect.width / 2;
                content_rect.x = center_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.resolve_draw_commands(&content_bounds, context)
            }
            VBottomAlign(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let bottom_most = bounds.y as usize + bounds.height;
                let top_start = bottom_most - content_rect.height;
                content_rect.y = top_start as i64;

                self.resolve_draw_commands(&content_rect, context)
            }
            HRightAlign(n) => {
                let mut content_rect = n.sizing.fit_into(bounds);
                let right_most = bounds.x as usize + bounds.width;
                let left_start = right_most - content_rect.width;
                content_rect.x = left_start as i64;

                let content_bounds = n.sizing.fit_into(&content_rect);

                self.resolve_draw_commands(&content_bounds, context)
            }
            VTopAlign(n) | HLeftAlign(n) => {
                let content_rect = n.sizing.fit_into(bounds);

                self.resolve_draw_commands(&content_rect, context)
            }
            TopPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.height = bounds.height.saturating_sub(n);
                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x;
                frame.y = bounds.y + n as i64;

                self.resolve_draw_commands(&frame, context)
            }
            BottomPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.height = bounds.height.saturating_sub(n);

                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                self.resolve_draw_commands(&frame, context)
            }
            RightPadding(n, node) => {
                let mut frame = node.sizing.fit_into(bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                let free_width = bounds.width.saturating_sub(n);
                let adjustment = frame.width.saturating_sub(free_width);

                frame.width = frame.width.saturating_sub(adjustment);

                self.resolve_draw_commands(&frame, context)
            }
            LeftPadding(n, node) => {
                let mut bounds = bounds.clone();
                bounds.width = bounds.width.saturating_sub(n);
                let mut frame = node.sizing.fit_into(&bounds);
                frame.x = bounds.x + n as i64;
                frame.y = bounds.y;

                self.resolve_draw_commands(&frame, context)
            }
            Background(c, node) => {
                let mut frame = node.sizing.fit_into(bounds);
                frame.x = bounds.x;
                frame.y = bounds.y;

                // self.draw_rect(bounds, &c.to_string());
                let mut commands = vec![DrawCommand::Rect(bounds.clone(), c.to_string())];

                let text_command = self.resolve_draw_commands(&frame, context);

                commands.extend(text_command);

                commands
            }
            Border(n, c, edges, node) => {
                let outer_bounds = bounds;
                let mut inner_bounds = bounds.clone();
                for edge in &edges {
                    match edge {
                        layout::alignment::Edge::Top => {
                            inner_bounds.height = inner_bounds.height.saturating_sub(n);
                            inner_bounds.y = inner_bounds.y.checked_add(n as i64).unwrap_or(0);
                        }
                        layout::alignment::Edge::Right => {
                            inner_bounds.width = inner_bounds.width.saturating_sub(n);
                        }
                        layout::alignment::Edge::Bottom => {
                            inner_bounds.height = inner_bounds.height.saturating_sub(n);
                        }
                        layout::alignment::Edge::Left => {
                            inner_bounds.width = inner_bounds.width.saturating_sub(n);
                            inner_bounds.x = inner_bounds.x.checked_add(n as i64).unwrap_or(0);
                        }
                    }
                }

                let mut frame = node.sizing.fit_into(&inner_bounds);
                frame.x = inner_bounds.x;
                frame.y = inner_bounds.y;

                let mut commands = self.resolve_draw_commands(&frame, context);

                for edge in &edges {
                    let command = match edge {
                        layout::alignment::Edge::Top => {
                            let line_bounds = Rect::new(outer_bounds.x, outer_bounds.y, outer_bounds.width, n);
                            let line = DrawCommand::Rect(line_bounds, c.to_string());
                            line
                        }
                        layout::alignment::Edge::Right => {
                            let line_bounds = Rect::new(outer_bounds.max_x() - n as i64, outer_bounds.y, n, outer_bounds.height);
                            let line = DrawCommand::Rect(line_bounds, c.to_string());
                            line
                        }
                        layout::alignment::Edge::Bottom => {
                            let line_bounds = Rect::new(outer_bounds.x, outer_bounds.max_y() - n as i64, outer_bounds.width, n);
                            let line = DrawCommand::Rect(line_bounds, c.to_string());
                            line
                        }
                        layout::alignment::Edge::Left => {
                            let line_bounds = Rect::new(outer_bounds.x, outer_bounds.y, n, outer_bounds.height);
                            let line = DrawCommand::Rect(line_bounds, c.to_string());
                            line
                        }
                    };

                    commands.push(command);
                }

                commands
            }
            VerticalStack(alignment, spacing, nodes) => {
                let mut max_width = 0usize;
                
                let spacing_sizing = spacing * (nodes.len().saturating_sub(1));

                let mut last_bounds = Rect::zero();

                let mut greedy_count = 0;
                let mut static_height = spacing_sizing;

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
                    let mut n = (*node).clone();
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

                    let spacing_offset = if raw_bounds.is_empty() {
                        0
                    } else {
                        spacing as i64
                    };

                    let node_bounds = Rect::new(0, last_bounds.max_y() + spacing_offset, size.width, size.height);
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

                let mut commands = nodes.into_iter().enumerate().flat_map(|(i, node)| {
                    let size = &final_bounds[i];

                    node.resolve_draw_commands(size, context)
                }).collect::<Vec<_>>();

                commands
            }
            HorizontalStack(alignment, spacing, nodes) => {
                let mut max_height = 0usize;

                let spacing_sizing = spacing * (nodes.len().saturating_sub(1));

                let mut last_bounds = Rect::zero();

                let mut greedy_count = 0;
                let mut static_width = spacing_sizing;

                for node in &nodes {
                    if let layout::sizing::Sizing::Static(n) = node.sizing.horizontal {
                        static_width += n;
                    } else {
                        greedy_count += 1;
                    }
                }

                let mut greedy_space = bounds.width.saturating_sub(static_width);
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

                    let spacing_offset = if raw_bounds.is_empty() {
                        0
                    } else {
                        spacing as i64
                    };

                    let node_bounds = Rect::new(last_bounds.max_x() + spacing_offset, 0, size.width, size.height);
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

                let commands = nodes.into_iter().enumerate().flat_map(|(i, node)| {
                    let size = &final_bounds[i];

                    node.resolve_draw_commands(size, context)
                }).collect::<Vec<_>>();

                commands
            }
            DrawCanvas(action) => {
                let result = action(context, bounds);

                vec![DrawCommand::Text(bounds.clone(), result.to_string())]
            }
        }
    }
}