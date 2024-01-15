use textyle::{layout::{Layout, alignment::{Edge, VerticalAlignment, HorizontalAlignment}}, hash_set};
use anyhow::Result;

use textyle::animation::{AnimatedTextCanvas, AnimationContext};

fn main() -> Result<()> {
    let animated_canvas = AnimatedTextCanvas::new(app);

    animated_canvas.run()?;

    Ok(())
}

fn app(_ctx: &AnimationContext) -> Layout<AnimationContext> {
    use Layout::*;
    type Node = Layout<AnimationContext>;
    let hspacer = Node::text("").center_horizontally();

    let status_bar = HorizontalStack(VerticalAlignment::Center, vec![
        Layout::text("Net").width(6),
        hspacer.clone(),
        Layout::text("01:30"),
        hspacer,
        Layout::text("80%").align_right().width(6)
    ])
    .padding_horizontal(1)
    .center_horizontally()
    .border(1, '=', hash_set![Edge::Bottom]);

    let note_row = |title, content| {
        Layout::VerticalStack(HorizontalAlignment::Left, vec![
            Node::text(title).padding_bottom(1),
            Node::text(content).align_left()
            .padding_left(4).padding_right(2),
        ]).padding_horizontal(1).border(1, '~', hash_set![Edge::Bottom])
    };

    let note_list = VerticalStack(HorizontalAlignment::Center, vec![
        note_row("New note", " "),
        note_row("New note", " "),
        note_row("First steps", "This is what I need :)")
    ]);

    let title = Text("Notes".to_string())
        .center_horizontally()
        .padding(1)
        .border(1, '-', hash_set![Edge::Bottom]);

    let ui = VerticalStack(HorizontalAlignment::Center, vec![
            title,
            note_list,
            Node::text("3 notes")
        ])
        .align_top();
    
    let screen_components = || {
        VerticalStack(HorizontalAlignment::Center, vec![
            status_bar,
            ui
        ])
    };

    screen_components()
        .align_top()
        .width(35)
        .center_vertically()
        .border(1, '•', hash_set![Edge::Top, Edge::Bottom])
        .border(2, '•', hash_set![Edge::Left, Edge::Right])
        .center()
}