use std::io::Write;

use anyhow::Result;

use crate::{canvas::TextCanvas, layout::{geometry::Size, Layout}};
use defer_lite::defer;

#[derive(Clone)]
pub struct AnimationContext {
    pub frame_count: usize,
    pub delta_milis: f64
}

pub struct AnimatedTextCanvas {
    layout: Layout<AnimationContext>
}

impl AnimatedTextCanvas {
    pub fn new(layout: fn(&AnimationContext)->Layout<AnimationContext>) -> Self {
        AnimatedTextCanvas {
            layout: Layout::WithContext(layout)
        }
    }

    fn clear_buffer(&self) {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge),
            crossterm::cursor::MoveTo(0, 0),
        ).unwrap();
    }

    pub fn run(&self) -> Result<()> {
        let mut stdout = std::io::stdout();

        let (terminal_columns, terminal_rows ) = crossterm::terminal::size().unwrap();

        let mut terminal_columns = terminal_columns as usize;
        let mut terminal_rows = terminal_rows as usize;

        let bounds = &Size::new(terminal_columns as usize, terminal_rows as usize);
        // let bounds = &Rect::sized(20, 5);
        let mut canvas = TextCanvas::create_in_bounds(bounds);

        let mut context = AnimationContext {
            frame_count: 0,
            delta_milis: 0.0
        };

        canvas.render_layout(&self.layout, &context);

        crossterm::terminal::enable_raw_mode().unwrap_or_else(|_| {
            crossterm::terminal::disable_raw_mode().unwrap();
        });
        defer! { let _ = crossterm::terminal::disable_raw_mode(); }

        let mut last_time = std::time::Instant::now();

        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::cursor::Hide)?;
        defer!{
            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen, crossterm::cursor::Show)
                .unwrap_or_else(|err| {
                    println!("Error exiting alternate screen buffer:\n{err}");
                });
        }
        loop {
            context.delta_milis = last_time.elapsed().as_secs_f64().clamp(0.000001, f64::MAX) * 1000.0;
            last_time = std::time::Instant::now();
            canvas.draw_on_buffer();
            
            if crossterm::event::poll(std::time::Duration::from_millis(1))? {
                match crossterm::event::read() {
                    Ok(event) => {
                        if let crossterm::event::Event::Key(k) = event {
                            if k.code == crossterm::event::KeyCode::Esc {
                                break;
                            }
                        } else if let crossterm::event::Event::Resize(columns, rows) = event {
                            terminal_columns = columns as usize;
                            terminal_rows = rows as usize;
        
                            let bounds = &Size::new(terminal_columns, terminal_rows);
                            canvas = TextCanvas::create_in_bounds(bounds);
                        }
                    }
                    Err(err) => {
                        let _ = write!(stdout, "{err}");
                        break;
                    }
                };
            }

            canvas.clear_with(" ");
            canvas.render_layout(&self.layout, &context);
            
            self.clear_buffer();
            context.frame_count += 1;
        }

        Ok(())
    }
}