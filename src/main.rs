use textyle::{layout::Layout, hash_set};
use textyle::layout::alignment::{VerticalAlignment, HorizontalAlignment, Edge};
use textyle::canvas::TextCanvas;
use anyhow::Result;

use textyle::animation::{AnimatedTextCanvas, AnimationContext};

fn main() -> Result<()> {
    let animated_canvas = AnimatedTextCanvas::new(app);

    animated_canvas.run()?;

    Ok(())
}

fn app(_ctx: &AnimationContext) -> Layout<AnimationContext> {
    let graph = Layout::DrawCanvas(|ctx: &AnimationContext, bounds| {
        let mut canvas = TextCanvas::create_in_bounds(&bounds.size());

        for x in 0..bounds.width {
            let xf = (x as f64) * 0.1 + (ctx.frame_count as f64 / 60.0);
            let height = xf.cos() / 2.4;
            let border = (height * bounds.height as f64) as i64 + (bounds.height as i64/2);
            for y in 0..bounds.height {
                if y > border as usize {
                    canvas.write("#", x, y);
                } else {
                    canvas.write("-", x, y);
                }
            }
        }

        canvas
    });
    
    Layout::HorizontalStack(VerticalAlignment::Top, vec![
        Layout::vertical_stack(vec![
            Layout::text("Main content").center_horizontally()
            .padding_vertical(2),
            graph.padding_horizontal(2)
        ])
        .border(2, '.', hash_set!(Edge::Right)),
        Layout::vertical_stack(vec![
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
    ])
}