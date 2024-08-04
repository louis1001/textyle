use std::fmt::Display;

use crate::{layout, rendering::DrawCommand};

use layout::geometry::{Rect, Size};
use unicode_segmentation::UnicodeSegmentation;

pub struct TextCanvas {
    size: Size,
    contents: Vec<String>,
}

impl Default for TextCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl TextCanvas {
    pub fn new() -> Self {
        TextCanvas {
            size: Size::zero(),
            contents: Vec::new(),
        }
    }

    pub fn create_in_bounds(size: &Size) -> Self {
        TextCanvas {
            size: size.clone(),
            contents: vec![" ".to_string(); size.width * size.height],
        }
    }

    pub fn create(width: usize, height: usize) -> Self {
        TextCanvas {
            size: Size::new(width, height),
            contents: vec![" ".to_string(); width * height],
        }
    }
}

impl TextCanvas {
    pub fn get_at(&self, x: usize, y: usize) -> Option<&str> {
        if x >= self.size.width || y >= self.size.height {
            return None;
        }

        let index = y * self.size.width + x;

        Some(self.contents[index].as_str())
    }

    pub fn write(&mut self, grapheme: &str, x: usize, y: usize) {
        if x >= self.size.width || y >= self.size.height { return; }

        let index = y * self.size.width + x;

        self.contents[index] = grapheme.to_string();
    }

    pub fn draw_rect(&mut self, bounds: &Rect, grapheme: &str) {
        for x in bounds.x..(bounds.x + bounds.width as i64) {
            for y in bounds.y..(bounds.y + bounds.height as i64) {
                if x < 0 || x >= self.size.width as i64 { continue; }
                if y < 0 || y >= self.size.height as i64 { continue; }

                self.write(grapheme, x as usize, y as usize);
            }
        }
    }
    
    pub fn paste_canvas(&mut self, other: &TextCanvas, bounds: &Rect) {
        assert_eq!(other.size.width, bounds.width);
        assert_eq!(other.size.height, bounds.height);
        
        for x in 0..bounds.width {
            for y in 0..bounds.height {
                let c = match other.get_at(x, y) {
                    Some(c) => c,
                    None => continue
                };

                self.write(c, x + bounds.x as usize, y + bounds.y as usize);
            }
        }
    }

    pub fn clear_with(&mut self, grapheme: &str) {
        self.draw_rect(&Rect::from_size(&self.size), grapheme);
    }
}

impl TextCanvas {
    fn execute_draw_commands(&mut self, commands: &[DrawCommand]) {
        for command in commands {
            match command {
                DrawCommand:: Text(bounds, text) => {
                    let graphemes = text.as_str().graphemes(true)
                    .collect::<Vec<_>>();
                    
                    let mut x = bounds.x as usize;
                    let mut y = bounds.y as usize;

                    let mut iter = graphemes.iter().peekable();

                    while let Some(g) = iter.next() {
                        if *g == "\n" {
                            y += 1;
                            x = bounds.x as usize;
                            continue;
                        } else if *g == " " {
                            // don't write anything
                        } else {
                            self.write(g, x, y);
                        }

                        x += 1;
                        if (x - bounds.x as usize) >= bounds.width {
                            y += 1;
                            x = bounds.x as usize;

                            if let Some(next) = iter.peek() {
                                if **next == "\n" {
                                    iter.next();
                                }
                            }
                        }
                    }
                }
                DrawCommand::FillRect(bounds, grapheme) => {
                    self.draw_rect(bounds, grapheme);
                }
                DrawCommand::StrokeRect(bounds, n, grapheme) => {
                    // Top
                    for x in bounds.x..(bounds.x + bounds.width as i64) {
                        if x < 0 || x >= self.size.width as i64 { continue; }
                        
                        for y in 0..*n {
                            let y_point = bounds.y + y as i64;
                            if y_point < 0 || y_point >= self.size.height as i64 { continue; }
                            self.write(grapheme, x as usize, y_point as usize);
                        }
                    }

                    // Bottom
                    for x in bounds.x..(bounds.x + bounds.width as i64) {
                        if x < 0 || x >= self.size.width as i64 { continue; }
                        
                        for y in 0..*n {
                            let y_point = bounds.y + bounds.height as i64 - y as i64 - 1;
                            if y_point < 0 || y_point >= self.size.height as i64 { continue; }
                            self.write(grapheme, x as usize, y_point as usize);
                        }
                    }

                    // Left
                    for y in bounds.y..(bounds.y + bounds.height as i64) {
                        if y < 0 || y >= self.size.height as i64 { continue; }
                        
                        for x in 0..*n {
                            let x_point = bounds.x + x as i64;
                            if x_point < 0 || x_point >= self.size.width as i64 { continue; }
                            self.write(grapheme, x_point as usize, y as usize);
                        }
                    }

                    // Right
                    for y in bounds.y..(bounds.y + bounds.height as i64) {
                        if y < 0 || y >= self.size.height as i64 { continue; }
                        
                        for x in 0..*n {
                            let x_point = bounds.x + bounds.width as i64 - x as i64 - 1;
                            if x_point < 0 || x_point >= self.size.width as i64 { continue; }
                            self.write(grapheme, x_point as usize, y as usize);
                        }
                    }
                }
            }
        }
    }
    
    pub fn render_layout<Ctx: Clone>(&mut self, layout: &layout::Layout<Ctx>, context: &mut Ctx) {
        let self_bounds = Rect::sized(self.size.width, self.size.height);
        let layout = layout.resolve_size(&self_bounds, context);
        let bounds = layout.sizing.fit_into(&self_bounds);

        let draw_commands = layout.resolve_draw_commands(&bounds, context);

        self.execute_draw_commands(&draw_commands);
    }

    pub fn draw_on_buffer(&self) {
        use std::io::Write;
        let chars = self.contents.clone();
        let mut stdout = std::io::stdout();
        for n in 0..chars.len() {
            let c = &chars[n];
            let _ = crossterm::queue!(stdout, crossterm::style::Print(c.to_string()));
    
            if n < chars.len()-1 && (n + 1) % self.size.width == 0 {
                let _ = crossterm::queue!(stdout, crossterm::cursor::MoveToNextLine(1) );
            }
        }
    
        let _ = stdout.flush();
    }
    
    pub fn print(&self) {
        use std::io::Write;
        let chars = self.contents.clone();
        let mut stdout = std::io::stdout();
        for n in 0..chars.len() {
            let c = &chars[n];
            let _ = crossterm::queue!(stdout, crossterm::style::Print(c.to_string()));
    
            if n < chars.len()-1 && (n + 1) % self.size.width == 0 {
                let _ = crossterm::queue!(stdout, crossterm::style::Print("\n".to_string()));
            }
        }
    
        let _ = stdout.flush();
    }
}

impl Display for TextCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for n in 0..self.contents.len() {
            let c = &self.contents[n];
            write!(f, "{c}")?;
    
            if n < self.contents.len()-1 && (n + 1) % self.size.width == 0 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}