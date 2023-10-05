use std::io::Write;
use std::{f32::consts::PI, fs::File, time::Duration};

use backrooms::{RaycastParams, World};
use cgmath::{vec2, MetricSpace, Vector2};
use crossterm::event::KeyEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ndarray::{array, Array2};
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
        [1, 1, 3, 1, 1, 1],
        [0, 2, 0, 3, 0, 1],
        [0, 1, 0, 0, 0, 1],
        [0, 2, 0, 1, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [1, 2, 1, 0, 2, 1],
    ];
    let world = World::from(data.map(|x| *x != 0));

    let params = RaycastParams {
        max_dist: 10,
        projection_plane_width: 2.0,
    };
    terminal::enable_raw_mode().unwrap();
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen, EnableMouseCapture).unwrap();

    terminal.hide_cursor().unwrap();

    let mut pos = vec2(2.5, 4.5);
    let mut facing = PI;

    loop {
        let mut file = File::create("pose.log").unwrap();
        writeln!(file, "{pos:?} {facing:?}").unwrap();

        terminal.clear().unwrap();
        terminal
            .draw(|f| render_canvas(f, &world, &data, pos, facing, &params))
            .unwrap();

        match event::poll(Duration::from_secs(60)).unwrap() {
            true => match event::read().unwrap() {
                event::Event::Key(k) => match k.code {
                    event::KeyCode::Char('a') => {
                        facing += 0.1;
                    }
                    event::KeyCode::Char('d') => {
                        facing -= 0.1;
                    }
                    event::KeyCode::Char('w') => {
                        pos += 0.1 * cos_sin(facing);
                    }
                    event::KeyCode::Char('s') => {
                        pos -= 0.1 * cos_sin(facing);
                    }
                    event::KeyCode::Esc => break,
                    _ => (),
                },
                _ => (),
            },
            false => break,
        }
    }

    terminal::disable_raw_mode().unwrap();
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
}

fn render_canvas(
    frame: &mut Frame<impl Backend>,
    world: &World,
    data: &Array2<i32>,
    pos: Vector2<f32>,
    facing: f32,
    params: &RaycastParams,
) {
    let width = frame.size().width as usize;
    let casts = world.raycast_plane(pos, cos_sin(facing), width, params);
    let vp_center = frame.size().height as f64 / 2.0;

    let widget = Canvas::default()
        .x_bounds([0.0, width as f64])
        .y_bounds([-90.0, 90.0])
        .marker(ratatui::symbols::Marker::Block)
        .paint(move |ctx| {
            let mut file = File::create("data.log").unwrap();
            for (i, r) in casts.iter().enumerate() {
                let Some(r) = r else { continue };
                let distance = r.hit_pos.distance(pos);
                let height = 100.0 / distance;

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

                writeln!(&mut file, "{:?} {:?} {:?} {:?}", i, r, distance, vp_center).unwrap();
            }
        });

    frame.render_widget(widget, frame.size())
}

pub fn cos_sin(a: f32) -> Vector2<f32> {
    let (s, c) = a.sin_cos();
    vec2(c, s)
}
