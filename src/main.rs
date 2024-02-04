use crossterm::event::KeyCode;
use textyle::{animation::AnimationContext, canvas::TextCanvas, layout::{alignment::{Edge, HorizontalAlignment, VerticalAlignment}, geometry::{Matrix, Vector}, Layout}};
use anyhow::Result;
use rand::prelude::*;

use textyle::animation::AnimatedTextCanvas;

#[derive(Clone)]
struct GameState {
    data: Matrix<u64>,
    game_over: bool
}

enum GameDirection {
    Up, Down,
    Left, Right
}

impl GameState {
    fn free_spaces(&mut self) -> Vec<Vector> {
        let mut result = vec![];
        for j in 0..4 {
            for i in 0..4 {
                if *self.data.get_mut(i,  j) == 0 {
                    result.push(Vector::new(i as i64, j as i64))
                }
            }
        }

        result
    }

    fn max_score(&self) -> u64 {
        let mut max_score = 0;
        for j in 0..4 {
            for i in 0..4 {
                if *self.data.get(i, j) > max_score {
                    max_score =  *self.data.get(i, j);
                }
            }
        }

        max_score
    }

    fn generate_new(&mut self) {
        let new_value = if rand::random() {
            2
        } else {
            4
        };

        let mut rng = rand::thread_rng();

        let spaces = self.free_spaces();
        if !spaces.is_empty() {
            let i = rng.gen_range(0..spaces.len());

            let point = &spaces[i];
            *self.data.get_mut(point.x() as usize,  point.y() as usize) = new_value;
        }
    }

    fn are_moves_available(&mut self) -> bool {
        let mut cloned = self.clone();
        cloned.collapse(GameDirection::Right);
        if cloned.data != self.data { return true }

        cloned.collapse(GameDirection::Down);
        if cloned.data != self.data { return true }

        cloned.collapse(GameDirection::Left);
        if cloned.data != self.data { return true }

        cloned.collapse(GameDirection::Up);
        if cloned.data != self.data { return true }

        false
    }

    fn collapse(&mut self, direction: GameDirection) {
        let mut new_data = self.data.clone();
        let prev_data = new_data.clone();

        match direction {
            GameDirection::Up => {
                let mut prev_numbers = [0, 0, 0, 0];
                let mut prev_positions = [5, 5, 5, 5];
                for j in 0..4 {
                    for i in 0..4 {
                        let value = *new_data.get(i, j);
                        let prev_position = prev_positions[i];
                        let prev_match = prev_numbers[i];
                        if prev_position == 5 {
                            prev_positions[i] = j;
                            prev_numbers[i] = value;
                        } else if prev_match == value {
                            let prev_match = prev_numbers[i];
                            *new_data.get_mut(i,  prev_position as usize) = 0;
                            *new_data.get_mut(i,  j) = prev_match * 2;

                            prev_positions[i] = 5;
                        } else if value != 0 {
                            prev_positions[i] = j;
                            prev_numbers[i] = value;
                        }
                    }
                }

                for i in 0..4 {
                    let mut accumulator = 0;
                    for j in 0..4 {
                        let value = *new_data.get(i, j);
                        if value != 0 {
                            *new_data.get_mut(i,  j) = 0;
                            *new_data.get_mut(i,  accumulator) = value;
                            if accumulator < 3 {
                                accumulator += 1;
                            }
                        }
                    }
                }
            },
            GameDirection::Down => {
                let mut prev_numbers = [0, 0, 0, 0];
                let mut prev_positions = [5, 5, 5, 5];
                for j in 0..4 {
                    let j = 3 - j;
                    for i in 0..4 {
                        let value = *new_data.get(i, j);
                        let prev_position = prev_positions[i];
                        let prev_match = prev_numbers[i];
                        if prev_position == 5 {
                            prev_positions[i] = j;
                            prev_numbers[i] = value;
                        } else if prev_match == value {
                            let prev_match = prev_numbers[i];
                            *new_data.get_mut(i,  prev_position as usize) = 0;
                            *new_data.get_mut(i,  j) = prev_match * 2;

                            prev_positions[i] = 5;
                        } else if value != 0 {
                            prev_positions[i] = j;
                            prev_numbers[i] = value;
                        }
                    }
                }

                for i in 0..4 {
                    let mut accumulator = 3;
                    for j in 0..4 {
                        let j = 3 - j;

                        let value = *new_data.get(i, j);
                        if value != 0 {
                            *new_data.get_mut(i,  j) = 0;
                            *new_data.get_mut(i,  accumulator) = value;
                            if accumulator > 0 {
                                accumulator -= 1;
                            }
                        }
                    }
                }
            },
            GameDirection::Left => {
                let mut prev_numbers = [0, 0, 0, 0];
                let mut prev_positions = [5, 5, 5, 5];
                for i in 0..4 {
                    for j in 0..4 {
                        let value = *new_data.get(i, j);
                        let prev_position = prev_positions[j];
                        let prev_match = prev_numbers[j];
                        if prev_position == 5 {
                            prev_positions[j] = i;
                            prev_numbers[j] = value;
                        } else if prev_match == value {
                            let prev_match = prev_numbers[j];
                            *new_data.get_mut(prev_position as usize,  j) = 0;
                            *new_data.get_mut(i,  j) = prev_match * 2;

                            prev_positions[j] = 5;
                        } else if value != 0{
                            prev_positions[j] = i;
                            prev_numbers[j] = value;
                        }
                    }
                }

                for j in 0..4 {
                    let mut accumulator = 0;
                    for i in 0..4 {
                        let value = *new_data.get(i, j);
                        if value != 0 {
                            *new_data.get_mut(i,  j) = 0;
                            *new_data.get_mut(accumulator,  j) = value;
                            if accumulator < 3 {
                                accumulator += 1;
                            }
                        }
                    }
                }
            },
            GameDirection::Right => {
                let mut prev_numbers = [0, 0, 0, 0];
                let mut prev_positions = [5, 5, 5, 5];
                for i in 0..4 {
                    let i = 3 - i;
                    for j in 0..4 {
                        let value = *new_data.get(i, j);
                        let prev_position = prev_positions[j];
                        let prev_match = prev_numbers[j];
                        if prev_position == 5 {
                            prev_positions[j] = i;
                            prev_numbers[j] = value;
                        } else if prev_match == value {
                            let prev_match = prev_numbers[j];
                            *new_data.get_mut(prev_position as usize,  j) = 0;
                            *new_data.get_mut(i,  j) = prev_match * 2;

                            prev_positions[j] = 5;
                        } else if value != 0 {
                            prev_positions[j] = i;
                            prev_numbers[j] = value;
                        }
                    }
                }

                for j in 0..4 {
                    let mut accumulator = 3;
                    for i in 0..4 {
                        let i = 3 - i;

                        let value = *new_data.get(i, j);
                        if value != 0 {
                            *new_data.get_mut(i,  j) = 0;
                            *new_data.get_mut(accumulator,  j) = value;
                            if accumulator > 0 {
                                accumulator -= 1;
                            }
                        }
                    }
                }
            },
        }
        
        self.data = new_data;

        if self.data != prev_data {
            self.generate_new();
        }
    }
}

type GameContext = AnimationContext<GameState>;

fn main() -> Result<()> {
    let mut animated_canvas = AnimatedTextCanvas::new(app);
    animated_canvas.set_update(update);

    let mut state = GameState {
        data: Matrix::with_rows(&[0; 16], 4),
        game_over: false
    };

    state.generate_new();
    state.generate_new();

    animated_canvas.run_with_state(state)?;

    Ok(())
}

fn app(ctx: &GameContext) -> Layout<GameContext> {
    let size = 20;
    
    let game_board = Layout::grid::<GameContext, u64>(&ctx.state.data, 1, |i| {
        if *i != 0 {
            Layout::<GameContext>::Text(format!("{i}"))
        } else {
            Layout::text("")
        }
        .center()
        .background(' ')
    })
    .height(size)
    .width(size * 2)
    .background('.')
    .border(1, '.', Edge::all())
    .center();

    let max_score = ctx.state.max_score();

    let info_table = Layout::HorizontalStack(VerticalAlignment::Center, 0, vec![
        Layout::Text(format!("Max score: {max_score}"))
        .padding_horizontal(2)
    ]).height(1);

    Layout::VerticalStack(HorizontalAlignment::Center, 0, vec![
        info_table,
        game_board,
    ])
}

fn update(ctx: &mut AnimationContext<GameState>) {
    let mut events = ctx.pending_events.clone();
    events.reverse();
    ctx.pending_events.clear();

    if ctx.state.game_over { return }

    while let Some(event) = events.pop() {
        match event {
            crossterm::event::Event::Key(k) => {
                if k.kind == crossterm::event::KeyEventKind::Press {
                    match k.code {
                        KeyCode::Left => { ctx.state.collapse(GameDirection::Left) },
                        KeyCode::Right => { ctx.state.collapse(GameDirection::Right) },
                        KeyCode::Up => { ctx.state.collapse(GameDirection::Up) },
                        KeyCode::Down => { ctx.state.collapse(GameDirection::Down) },
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if !ctx.state.are_moves_available() {
        ctx.state.game_over = true;
    }
}
