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
    fn get_at(&self, x: usize, y: usize) -> Option<&str> {
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

    fn draw_rect(&mut self, bounds: &Rect, grapheme: &str) {
        for x in bounds.x..(bounds.x + bounds.width as i64) {
            for y in bounds.y..(bounds.y + bounds.height as i64) {
                if x < 0 || x >= self.size.width as i64 { continue; }
                if y < 0 || y >= self.size.height as i64 { continue; }

                self.write(grapheme, x as usize, y as usize);
            }
        }
    }
    
    fn paste_canvas(&mut self, other: &TextCanvas, bounds: &Rect) {
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
                    let mut graphemes = text.as_str().graphemes(true);
                    for x in bounds.x..(bounds.x + bounds.width as i64) {
                        for y in bounds.y..(bounds.y + bounds.height as i64) {
                            if x < 0 || x >= self.size.width as i64 { continue; }
                            if y < 0 || y >= self.size.height as i64 { continue; }

                            // Fixme, ignore new lines because the bounds already account for them.
                            let c = match graphemes.next() {
                                None | Some(" ") => continue,
                                Some(c) => c
                            };

                            self.write(c, x as usize, y as usize);
                        }
                    }
                }
                DrawCommand::Rect(bounds, grapheme) => {
                    self.draw_rect(bounds, grapheme);
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

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for n in 0..self.contents.len() {
            let c = &self.contents[n];
            result.push_str(c);
    
            if n < self.contents.len()-1 && (n + 1) % self.size.width == 0 {
                result.push('\n');
            }
        }
    
        result
    }
}