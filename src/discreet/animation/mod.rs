use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::{discreet::canvas::TextCanvas, discreet::layout::{geometry::Size, Layout}};
use defer_lite::defer;

pub trait AnimationState: Clone {}
impl <T: Clone> AnimationState for T {}

#[derive(Clone)]
pub struct AnimationContext<State: AnimationState> {
    pub frame_count: usize,
    pub delta_milis: f64,
    pub state: State,
    pub pending_events: Vec<crossterm::event::Event>
}

pub type PlainAnimationContext = AnimationContext<()>;

type AnimatedLayoutProvider<State> = fn(&AnimationContext<State>)->Layout<AnimationContext<State>>;
pub struct AnimatedTextCanvas<State: AnimationState> {
    layout: AnimatedLayoutProvider<State>,
    update: fn(&mut AnimationContext<State>)
}

impl<State: AnimationState> AnimatedTextCanvas<State> {
    fn clear_buffer(&self) {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge),
            crossterm::cursor::MoveTo(0, 0),
        ).unwrap();
    }
    
    pub fn set_update(&mut self, update_fn: fn(&mut AnimationContext<State>)) {
        self.update = update_fn;
    }
}

impl<State: AnimationState> AnimatedTextCanvas<State> {
    pub fn new(layout: AnimatedLayoutProvider<State>) -> Self {
        AnimatedTextCanvas { layout, update: |_|{} }
    }

    pub fn run_with_state(&self, state: State) -> Result<()> {
        let mut stdout = std::io::stdout();

        let (terminal_columns, terminal_rows ) = crossterm::terminal::size().unwrap();

        let mut terminal_columns = terminal_columns as usize;
        let mut terminal_rows = terminal_rows as usize;

        let bounds = &Size::new(terminal_columns, terminal_rows);
        // let bounds = &Rect::sized(20, 5);
        let mut canvas = TextCanvas::create_in_bounds(bounds);

        let mut context = AnimationContext {
            frame_count: 0,
            delta_milis: 0.0,
            state,
            pending_events: vec![]
        };

        canvas.render_layout(&Layout::WithContext(self.layout), &mut context);

        crossterm::terminal::enable_raw_mode().unwrap_or_else(|_| {
            crossterm::terminal::disable_raw_mode().unwrap();
        });
        defer! { let _ = crossterm::terminal::disable_raw_mode(); }

        let mut last_time = std::time::Instant::now();

        crossterm::execute!(stdout, /*crossterm::terminal::EnterAlternateScreen, */ crossterm::cursor::Hide)?;
        
        defer!{
            crossterm::execute!(std::io::stdout(), /*crossterm::terminal::LeaveAlternateScreen, */ crossterm::cursor::Show)
                .unwrap_or_else(|err| {
                    println!("Error exiting alternate screen buffer:\n{err}");
                });
        }

        loop {
            (self.update)(&mut context);
            context.delta_milis = last_time.elapsed().as_secs_f64().clamp(0.000001, f64::MAX) * 1000.0;
            last_time = std::time::Instant::now();
            canvas.draw_on_buffer();
            
            if crossterm::event::poll(std::time::Duration::from_millis(1))? {
                match crossterm::event::read() {
                    Ok(event) => {
                        if let crossterm::event::Event::Key(KeyEvent { code: crossterm::event::KeyCode::Esc, .. }) = event {
                            break;
                        } else if let crossterm::event::Event::Resize(columns, rows) = event {
                            terminal_columns = columns as usize;
                            terminal_rows = rows as usize;
        
                            let bounds = &Size::new(terminal_columns, terminal_rows);
                            canvas = TextCanvas::create_in_bounds(bounds);
                        } else {
                            context.pending_events.push(event);
                        }
                    }
                    Err(err) => {
                        crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen, crossterm::style::Print(format!("{err}")), crossterm::terminal::EnterAlternateScreen)?;
                        break;
                    }
                };
            }

            canvas.clear_with(" ");

            canvas.render_layout(&Layout::WithContext(self.layout), &mut context);
            
            self.clear_buffer();
            context.frame_count += 1;
        }

        Ok(())
    }
}

impl AnimatedTextCanvas<()> {
    pub fn run(&self) -> Result<()> {
        self.run_with_state(())
    }
}