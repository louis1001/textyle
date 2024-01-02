use textyle::{layout::Layout, hash_set};
use textyle::layout::alignment::{VerticalAlignment, HorizontalAlignment, Edge};
use textyle::layout::Rect;
use textyle::canvas::TextCanvas;

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
    let mut canvas = TextCanvas::create_in_bounds(bounds);

    // canvas.clear_with(".");

    canvas.render_layout(&layout);

    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).unwrap();
    loop {
        canvas.print_canvas();
        
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
                    canvas = TextCanvas::create_in_bounds(bounds);

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
