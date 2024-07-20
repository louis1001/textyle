use anyhow::Result;
use textyle::{canvas::TextCanvas, layout::{alignment::Edge, Layout}};

fn main() -> Result<()> {
    let layout: Layout<()> = Layout::
        text("testing commands")
        .center()
        .padding(2)
        .background('`')
        .padding(2)
        .border(1, '.', Edge::all());

    let mut canvas = TextCanvas::create(30, 20);

    canvas.render_layout(&layout, &mut ());

    canvas.print();

    Ok(())
}
