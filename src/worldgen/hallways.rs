use cgmath::{vec2, Vector2};
use image::{ImageBuffer, Rgb};
use ndarray::{s, Array2};
use rand::{
    distributions::Standard, prelude::Distribution, rngs::SmallRng, seq::SliceRandom, Rng,
    SeedableRng,
};

use crate::util::{Direction, RelativeBounds, TurnDir, Turnable};

#[derive(Debug, Clone)]
pub struct Hallway {
    pub position: Vector2<isize>,
    pub length: usize,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub struct HallwayTree {
    length: usize,
    children: Vec<BranchChild>,
}

#[derive(Debug, Clone)]
pub struct BranchChild {
    position: usize,
    direction: TurnDir,
    child: HallwayTree,
}

pub fn random_hallway_tree(
    rng: &mut impl Rng,
    bounds: RelativeBounds<usize>,
    tree_depth: usize,
) -> Option<HallwayTree> {
    if tree_depth == 0 || bounds.forward < 10 {
        return None;
    }

    let length = rng.gen_range(bounds.forward / 2..bounds.forward);
    let new_bounds = RelativeBounds {
        forward: length,
        ..bounds
    };

    let children = if tree_depth > 0 && length > 20 {
        (0..rng.gen_range(0..10))
            .flat_map(|_| {
                let position: usize = rng.gen_range(1..=length);
                let direction = rng.gen::<TurnDir>();
                let bounds = new_bounds
                    .clone()
                    .map(|a| a as isize)
                    .translate(vec2(0, -(position as isize)));
                let child = random_hallway_tree(rng, bounds.map(|a| a as usize), tree_depth - 1)?;

                Some(BranchChild {
                    position,
                    direction,
                    child,
                })
            })
            .collect()
    } else {
        vec![]
    };

    Some(HallwayTree { length, children })
}

fn full_length_hallway(direction: Direction, position: usize, length: usize) -> Hallway {
    let position = match direction {
        Direction::East => vec2(0, position),
        Direction::North => vec2(position, 0),
        Direction::West => vec2(length - 1, position),
        Direction::South => vec2(position, length - 1),
    };

    Hallway {
        position: position.cast().unwrap(),
        length,
        direction,
    }
}

pub fn draw_hallway(a: &mut Array2<bool>, h: Hallway) {
    let pos = h.position;
    let step: Vector2<isize> = h.direction.into();
    for i in 0..=h.length as isize {
        let Vector2 { x, y } = pos.cast().unwrap() + i * step;
        if let Some(c) = a.get_mut((x as usize, y as usize)) {
            *c = false
        }
    }
}

impl HallwayTree {
    pub fn get_hallways(
        self,
        root_pos: Vector2<isize>,
        root_dir: Direction,
    ) -> Box<dyn Iterator<Item = Hallway>> {
        let this = Hallway {
            position: root_pos,
            length: self.length,
            direction: root_dir,
        };
        let axis = Vector2::<isize>::from(root_dir);

        Box::new(
            [this]
                .into_iter()
                .chain(self.children.into_iter().flat_map(move |c| {
                    let abs_pos = root_pos + axis * c.position as isize;
                    let abs_dir = root_dir.rotate(c.direction);
                    c.child.get_hallways(abs_pos, abs_dir)
                })),
        )
    }
}
