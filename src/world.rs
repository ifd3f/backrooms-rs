use std::{ops::Index, process::Output};

use cgmath::{vec2, One, Vector2, VectorSpace, Zero};
use ndarray::Array2;

#[derive(Debug, Clone)]
pub struct World {
    map: Array2<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East,
    North,
    West,
    South,
}

#[derive(Debug, Clone)]
pub struct RaycastParams {
    pub max_dist: usize,
    /// The projection plane is 1 unit away from the camera. Adjusting this value
    /// allows you to adjust the FOV.
    pub projection_plane_width: f32,
}

#[derive(Debug, Clone)]
pub struct Raycast {
    pub hit_pos: Vector2<f32>,
    pub wall: Vector2<usize>,
    pub wall_side: Direction,
}

impl World {
    pub fn get(&self, pos: (usize, usize)) -> Option<bool> {
        self.map.get((pos.1, pos.0)).copied()
    }

    pub fn get_signed(&self, pos: (isize, isize)) -> Option<bool> {
        if pos.0 < 0 || pos.1 < 0 {
            None
        } else {
            self.map.get((pos.1 as usize, pos.0 as usize)).copied()
        }
    }

    /// Perform a single raycast from the given position along the given ray.
    pub fn raycast(
        &self,
        pos: Vector2<f32>,
        ray: Vector2<f32>,
        max_dist: usize,
    ) -> Option<Raycast> {
        let mut march_pos = pos;
        let mut this_grid = march_pos.map(|x| x.floor()).cast::<isize>().unwrap();

        for _ in 0..=max_dist {
            let box_offset = this_grid.cast().unwrap();
            let box_pos = march_pos - box_offset;
            let (box_hit_pos, outgoing_dir) = raycast_in_box(box_pos, ray);
            let hit_pos = box_hit_pos + box_offset;

            let probe_cell = this_grid + Vector2::<isize>::from(outgoing_dir);

            match self.get_signed((probe_cell.x, probe_cell.y)) {
                Some(true) => {
                    return Some(Raycast {
                        hit_pos,
                        wall: probe_cell.cast().unwrap(),
                        wall_side: -outgoing_dir,
                    });
                }
                Some(false) | None => {
                    march_pos = hit_pos;
                    this_grid = probe_cell;
                }
            }
        }
        None
    }

    /// Raycast along a plane.
    ///
    /// Facing must be a unit vector.
    pub fn raycast_plane(
        &self,
        pos: Vector2<f32>,
        facing_unit: Vector2<f32>,
        n_rays: usize,
        params: &RaycastParams,
    ) -> Vec<Option<Raycast>> {
        let rays = gen_rays(facing_unit, params.projection_plane_width, n_rays);

        rays.map(|ray| self.raycast(pos, ray, params.max_dist))
            .collect()
    }
}

/// Generates a number of rays, for projection plane distance of 1.
///
/// Facing must be a unit vector.
fn gen_rays(
    facing_unit: Vector2<f32>,
    projection_plane_width: f32,
    n_rays: usize,
) -> impl Iterator<Item = Vector2<f32>> {
    // Calculate the unit vector pointed in the facing direction, and its perpendicular
    let facing_left = vec2(facing_unit.y, -facing_unit.x);

    // Calculate the projection plane
    let pp_leftmost_point = facing_unit + (projection_plane_width / 2.0) * facing_left;

    (0..n_rays).map(move |i| pp_leftmost_point - (i as f32 / n_rays as f32) * facing_left)
}

impl From<Array2<bool>> for World {
    fn from(value: Array2<bool>) -> Self {
        Self { map: value }
    }
}

impl From<Vector2<f32>> for Direction {
    fn from(v: Vector2<f32>) -> Self {
        match (v.x >= v.y, v.x >= -v.y) {
            (true, true) => Direction::North,
            (true, false) => Direction::West,
            (false, false) => Direction::South,
            (false, true) => Direction::East,
        }
    }
}

impl std::ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Self::Output {
        match self {
            Direction::East => Direction::West,
            Direction::North => Direction::South,
            Direction::West => Direction::East,
            Direction::South => Direction::North,
        }
    }
}

impl<S> From<Direction> for Vector2<S>
where
    S: One + Zero + std::ops::Neg<Output = S>,
{
    fn from(value: Direction) -> Self {
        match value {
            Direction::East => vec2(S::one(), S::zero()),
            Direction::North => vec2(S::zero(), S::one()),
            Direction::West => vec2(-S::one(), S::zero()),
            Direction::South => vec2(S::zero(), -S::one()),
        }
    }
}

impl Direction {
    /// Reflect left to right, and right to left.
    pub fn reflect_lr(self) -> Self {
        match self {
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            ns => ns,
        }
    }

    /// Reflect up to down, and down to up.
    pub fn reflect_ud(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            ew => ew,
        }
    }
}

/// Raycast to the edge of the box bounded by points (0, 0) and (1, 1).
fn raycast_in_box(pos: Vector2<f32>, ray: Vector2<f32>) -> (Vector2<f32>, Direction) {
    use Direction::*;

    /// This is restricted to the case where both components of ray_unit
    /// are less than or equal to zero.
    #[inline(always)]
    fn towards_origin(pos: Vector2<f32>, ray: Vector2<f32>) -> (Vector2<f32>, Direction) {
        let xdir = if ray.x > 0.0 { East } else { West };
        let ydir = if ray.y > 0.0 { North } else { South };

        match (ray.x == 0.0, ray.y == 0.0) {
            (true, true) => panic!("Cannot raycast with zero-valued ray"),
            (true, false) => return (vec2(pos.x, 0.0), ydir),
            (false, true) => return (vec2(0.0, pos.y), xdir),
            (false, false) => (),
        }

        let x_int = pos.x - (ray.x / ray.y) * pos.y;
        let y_int = pos.y - (ray.y / ray.x) * pos.x;

        if x_int < 0.0 {
            (vec2(0.0, y_int), xdir)
        } else {
            (vec2(x_int, 0.0), ydir)
        }
    }

    if ray.x > 0.0 {
        let (o, d) = raycast_in_box(vec2(1.0 - pos.x, pos.y), vec2(-ray.x, ray.y));
        return (vec2(1.0 - o.x, o.y), d.reflect_lr());
    }
    if ray.y > 0.0 {
        let (o, d) = towards_origin(vec2(pos.x, 1.0 - pos.y), vec2(ray.x, -ray.y));
        return (vec2(o.x, 1.0 - o.y), d.reflect_ud());
    }

    towards_origin(pos, ray)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{assert_ulps_eq, vec2, InnerSpace, Vector2};
    use ndarray::array;
    use rstest::rstest;

    fn example_world() -> World {
        let data = array![
            [1, 1, 3, 1, 1, 1, 1, 1, 1],
            [3, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 0, 0, 1],
            [2, 0, 0, 0, 0, 0, 3, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 2, 1, 1, 1, 1, 1, 2, 1],
        ];
        World::from(data.map(|x| *x != 0))
    }

    #[rstest]
    #[case(vec2(0.75, 0.5), vec2(-1.0, 0.0),  (vec2(0.0, 0.5),    Direction::West))]
    #[case(vec2(0.75, 0.5), vec2(0.0, -1.0),  (vec2(0.75, 0.0),   Direction::South))]
    #[case(vec2(0.5, 0.5),  vec2(-1.0, 0.5),  (vec2(0.0, 0.75),   Direction::West))]
    #[case(vec2(0.25, 0.5), vec2(1.0, -0.25), (vec2(1.0, 0.3125), Direction::East))]
    #[case(vec2(0.5, 0.25), vec2(1.0, 1.0),   (vec2(1.0, 0.75),   Direction::East))]
    #[case(vec2(0.5, 0.5),  vec2(1.0, 1.0),   (vec2(1.0, 1.0),    Direction::North))]
    fn test_raycast_in_box(
        #[case] pos: Vector2<f32>,
        #[case] ray: Vector2<f32>,
        #[case] expected: (Vector2<f32>, Direction),
    ) {
        let (hit, dir) = raycast_in_box(pos, ray);

        assert_eq!(dir, expected.1);
        assert_ulps_eq!(hit, expected.0);
    }

    #[rstest]
    #[case(
        (vec2(2.5, 2.5), vec2(-1.0, 0.0)),
        Raycast {
            hit_pos: vec2(1.0, 2.5),
            wall: vec2(0, 2),
            wall_side: Direction::East
        }
    )]
    #[case(
        (vec2(1.05, 1.05), vec2(-0.5, -1.0)),
        Raycast {
            hit_pos: vec2(1.025, 1.0),
            wall: vec2(1, 0),
            wall_side: Direction::North
        }
    )]
    fn raycast_edge(#[case] ray: (Vector2<f32>, Vector2<f32>), #[case] expected: Raycast) {
        let (pos, ray) = ray;
        let world = example_world();

        let result = world.raycast(pos, ray, 100).unwrap();

        assert_eq!(result.wall_side, expected.wall_side);
        assert_eq!(result.wall, expected.wall);
        assert_ulps_eq!(result.hit_pos, expected.hit_pos)
    }
}
