use std::time::Duration;

use backrooms::{
    camera::{raycast_camera, CameraParams},
    world::ArrayWorld,
};
use cgmath::{vec2, MetricSpace, Vector2};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ndarray::{array, Array2};
use ratatui::widgets::Block;
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    style::Color,
    widgets::canvas::{Canvas, Line},
    Frame, Terminal,
};

pub fn main() {
    let backend = CrosstermBackend::new(std::io::stderr());
    let mut terminal = Terminal::new(backend).expect("Failed to set up terminal");
    let data = array![
        [1, 1, 3, 1, 1, 1, 1, 1, 1],
        [3, 0, 0, 2, 0, 0, 0, 0, 2],
        [1, 0, 0, 0, 0, 0, 0, 0, 1],
        [2, 0, 0, 0, 0, 0, 3, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 2, 1, 1, 1, 1, 1, 2, 1],
    ];
    let world = ArrayWorld::from(data.map(|x| *x != 0));

    terminal::enable_raw_mode().unwrap();
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen, EnableMouseCapture).unwrap();

    terminal.hide_cursor().unwrap();

    let mut pos = vec2(3.5, 4.5);
    let mut facing = 0.0;
    let mut ppw = 1.0;
    terminal.clear().unwrap();

    'outer: loop {
        macro_rules! redraw {
            () => {
                terminal
                    .draw(|f| render_canvas(f, &world, &data, pos, facing, ppw))
                    .unwrap();
            };
        }

        redraw!();

        while event::poll(Duration::from_secs(60)).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(k) => match k.code {
                    event::KeyCode::Char('q') => {
                        facing -= 0.1;
                        redraw!();
                    }
                    event::KeyCode::Char('e') => {
                        facing += 0.1;
                        redraw!();
                    }
                    event::KeyCode::Char('w') => {
                        pos += 0.1 * cos_sin(facing);
                        redraw!();
                    }
                    event::KeyCode::Char('s') => {
                        pos -= 0.1 * cos_sin(facing);
                        redraw!();
                    }
                    event::KeyCode::Char('a') => {
                        pos += 0.1 * cos_sin_rot(facing);
                        redraw!();
                    }
                    event::KeyCode::Char('d') => {
                        pos -= 0.1 * cos_sin_rot(facing);
                        redraw!();
                    }
                    event::KeyCode::Char('t') => {
                        ppw *= 1.1;
                        redraw!();
                    }
                    event::KeyCode::Char('g') => {
                        ppw /= 1.1;
                        redraw!();
                    }
                    event::KeyCode::Esc => break 'outer,
                    _ => (),
                },
                _ => (),
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
}

fn render_canvas(
    frame: &mut Frame<impl Backend>,
    world: &ArrayWorld,
    data: &Array2<i32>,
    pos: Vector2<f32>,
    facing: f32,
    ppw: f32,
) {
    let width = frame.size().width as usize;
    let casts = raycast_camera(
        world,
        &CameraParams {
            pos,
            facing_unit: cos_sin(facing),
            n_rays: width,
            max_dist: 100.0,
            projection_plane_width: ppw,
        },
    );

    let widget = Canvas::default()
        .block(Block::default().title(format!("{pos:?}")))
        .x_bounds([0.0, width as f64])
        .y_bounds([-90.0, 90.0])
        .marker(ratatui::symbols::Marker::Block)
        .paint(move |ctx| {
            for (i, r) in casts.iter().enumerate() {
                let Some(r) = r else { continue };
                let distance = r.hit_pos.distance(pos);
                let height = 200.0 / distance;

                ctx.draw(&Line {
                    x1: i as f64,
                    y1: height as f64 / 2.0,
                    x2: i as f64,
                    y2: -height as f64 / 2.0,
                    color: match data[(r.wall.y, r.wall.x)] {
                        1 => Color::Blue,
                        2 => Color::Red,
                        3 => Color::Gray,
                        _ => Color::White,
                    },
                });
            }
        });

    frame.render_widget(widget, frame.size())
}

pub fn cos_sin(a: f32) -> Vector2<f32> {
    let (s, c) = a.sin_cos();
    vec2(c, s)
}

pub fn cos_sin_rot(a: f32) -> Vector2<f32> {
    let (s, c) = a.sin_cos();
    vec2(s, -c)
}
