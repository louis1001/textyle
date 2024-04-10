use anyhow::Result;
use textyle::continuous::canvas::Canvas;
use textyle::continuous::color::Rgba;
use textyle::continuous::layout::Layout;

use nannou::prelude::*;

fn main() -> Result<()> {
    let mut canvas = Canvas::create(400, 300);

    nannou::app(model).update(update).event(event).run();

    Ok(())
}

struct Model {
    _window: window::Id,
    _canvas: Canvas,
    pub _cached_size: (f32, f32),
    layout: Layout<()>
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    let mut _canvas = Canvas::create(10, 10);
    Model { _window, _canvas, _cached_size: (10.0, 10.0), layout: layout() }
}

fn layout() -> Layout<()> {
    Layout::vertical_stack(vec![
        Layout::horizontal_stack(vec![]).width(40).height(40).background(Rgba::new(1.0, 0.0, 0.0, 1.0))
        .padding(20)
    ])
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
    match _event {
        Event::WindowEvent { simple: Some(event), .. } => {
            match event {
                WindowEvent::Resized(size) => {
                    _model._cached_size = (size.x, size.y);
                    _model._canvas = Canvas::create(size.x as usize, size.y as usize);

                    _model._canvas.render_layout(&_model.layout, &mut ());
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEELBLUE);
    draw.to_frame(app, &frame).unwrap();
}