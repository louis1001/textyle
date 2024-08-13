use anyhow::Result;
use textyle::{animation::{AnimatedTextCanvas, AnimationBuffer, AnimationCommand, AnimationEvent, AnimationRunConfig, KeyCode, KeyModifiers, PlainAnimationContext}, canvas::TextCanvas, hash_set, layout::{alignment::{Edge, HorizontalAlignment}, Layout}};

fn main() -> Result<()> {
    let mut canvas = AnimatedTextCanvas::new(app);

    canvas.set_update(app_update);

    let config = AnimationRunConfig{
        buffer_type: AnimationBuffer::Alternate
    };

    canvas.run(config)?;

    Ok(())
}

fn app_update(ctx: &mut PlainAnimationContext) {
    while let Some(event) = ctx.pending_events.pop() {
        if let AnimationEvent::KeyEvent(e, modifiers) = event {
            if modifiers.contains(KeyModifiers::CONTROL) {
                if let KeyCode::Char(' ') = e {
                    ctx.add_command(AnimationCommand::Quit);
                }
            }
        }
    }
}

fn pixel_shader(ctx: &PlainAnimationContext, aspect_ratio: f64, u: f64, v: f64) -> &'static str {
    let x = (2.0 * u - 1.0) * aspect_ratio;
    let y = 1.0 - 2.0 * v;

    let r = x * x + y * y;

    let frame_duration = 200;

    // constant progress
    let progress = (ctx.frame_count % frame_duration) as f64 / frame_duration as f64;

    if r < (progress - 0.5).abs() * 2.0 {
        match r {
            r if r < 0.05 => " ",
            r if r < 0.1 => "-",
            r if r < 0.2 => "`",
            r if r < 0.3 => ".",
            r if r < 0.4 => ",",
            r if r < 0.5 => ":",
            r if r < 0.6 => ";",
            r if r < 0.7 => "?",
            r if r < 0.8 => "0",
            r if r < 0.9 => "8",
            _ => "@",
        }
    } else {
        " "
    }
}

fn draw_contents(_: &PlainAnimationContext) -> Layout<PlainAnimationContext> {
    Layout::DrawCanvas(|ctx, bounds| {
        let mut canvas = TextCanvas::create_in_bounds(&bounds.size());

        for x in 0..bounds.width {
            for y in 0..bounds.height {
                let u = x as f64 / bounds.width as f64;
                let v = y as f64 / bounds.height as f64;

                let aspect_ratio = bounds.width as f64 / bounds.height as f64;
                let grapheme = pixel_shader(ctx, aspect_ratio, u, v);

                canvas.write(grapheme, x, y);
            }
        }

        canvas
    })
}

fn text_contents(_ctx: &PlainAnimationContext) -> Layout<PlainAnimationContext> {
    Layout::VerticalStack(HorizontalAlignment::Left, 1, vec![
        Layout::VerticalStack(HorizontalAlignment::Left, 1, vec![
            Layout::text("This is Textyle"),
            Layout::text("A text-based UI library"),
        ])
        .border(1, '-', hash_set![Edge::Bottom]),
        Layout::text("A simple project for myself,\nin order to learn UI and Layout system basics.")
    ])
    .padding_vertical(2)
    .padding_horizontal(4)
    .align_left()
    .align_top()
    .border(1, '|', hash_set![Edge::Left])
    .padding_left(1)
}

fn app(ctx: &PlainAnimationContext) -> Layout<PlainAnimationContext> {
    Layout::horizontal_stack(vec![
        draw_contents(ctx)
            .width(20),
        text_contents(ctx)

    ])
    .center()
    .border(1, '%', Edge::all())
}
