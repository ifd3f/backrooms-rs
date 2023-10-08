use backrooms::{util::{RelativeBounds, Direction}, worldgen::{hallways::{random_hallway_tree, draw_hallway}, render_to_img}};
use cgmath::{vec2, conv::array2};
use ndarray::Array2;
use rand::{rngs::SmallRng, SeedableRng};

pub fn main() {
    // let mut rng = SmallRng::seed_from_u64(10);
    let mut rng = SmallRng::from_entropy();
    let w = random_hallway_tree(
        &mut rng,
        RelativeBounds {
            forward: 512,
            back: 0,
            left: 256,
            right: 256,
        },
        3,
    ).unwrap();
    println!("{w:#?}");

    let hws = w.get_hallways(vec2(256, 0), Direction::North).collect::<Vec<_>>();
    let mut a = Array2::zeros((512, 512)).map(|_: &i32| true);
    for h in hws {
        draw_hallway(&mut a, h)
    }

    let img = render_to_img(&a);
    img.save("test.png").unwrap();
    
}
