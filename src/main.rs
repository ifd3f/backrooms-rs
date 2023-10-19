use backrooms::{
    util::{Axis, Rectangle, Line},
    worldgen::{
        hallways::{rbsp, RbspParams},
        render_to_img,
    },
};
use ndarray::Array2;
use rand::{rngs::SmallRng, SeedableRng};

pub fn main() {
    // let mut rng = SmallRng::seed_from_u64(10);
    let mut rng = SmallRng::from_entropy();
    let (rooms, lines) = rbsp(
        &mut rng,
        Rectangle {
            x: 0,
            y: 0,
            w: 512,
            h: 512,
        },
        RbspParams {
            min_room_len: 20,
            max_room_len: 50,
            p_keep_rooms: 0.3,
            k_deoblongification: 5.0,
        },
    );

    let mut a = Array2::zeros((512, 512)).map(|_: &i32| true);
    for h in lines {
        draw_hallway(&mut a, h)
    }

    let img = render_to_img(&a);
    img.save("test.png").unwrap();
}

pub fn draw_hallway(a: &mut Array2<bool>, l: Line) {
    for pos in l.points() {
        if let Some(c) = a.get_mut((pos.0 as usize, pos.1 as usize)) {
            *c = false
        }
    }
}
