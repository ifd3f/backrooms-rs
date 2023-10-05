use std::ops::Index;

use cgmath::{vec2, Vector2, VectorSpace};
use ndarray::Array2;

#[derive(Debug, Clone)]
pub struct World {
    map: Array2<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    East,
    North,
    West,
    South,
}

#[derive(Debug, Clone)]
pub struct RaycastParams {
    pub max_dist: usize,
    pub n_columns: usize,
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
    /// Perform a single raycast from the given position along the given ray.
    ///
    /// Ray must be a unit vector
    pub fn raycast(
        &self,
        pos: Vector2<f32>,
        ray_unit: Vector2<f32>,
        max_dist: usize,
    ) -> Option<Raycast> {
        // Probe for the first filled grid
        for i in 1..=max_dist {
            let march_pos = pos + ray_unit * i as f32;
            let Some(grid_cell) = march_pos.cast::<usize>() else {
                return None;
            };
            match self.map.get((grid_cell.y, grid_cell.x)) {
                Some(true) => {
                    // Found the grid position, perform raycast operation
                    let last_march_pos = pos + ray_unit * (i - 1) as f32;
                    let box_offset = last_march_pos.map(|x| x.floor());
                    let box_pos = last_march_pos - box_offset;
                    let box_hit_pos = raycast_in_box(box_pos, ray_unit);
                    let hit_pos = box_hit_pos + box_offset;

                    return Some(Raycast {
                        hit_pos,
                        wall: grid_cell,
                        wall_side: box_hit_pos_to_box_side(box_hit_pos),
                    });
                }
                Some(false) => continue,
                _ => return None,
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
        params: RaycastParams,
    ) -> Vec<Option<Raycast>> {
        let rays = gen_rays(facing_unit, params.projection_plane_width, params.n_columns);

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

    (0..n_rays).map(move |i| pp_leftmost_point - (i as f32 / n_rays as f32) * facing_unit)
}

pub fn cos_sin(a: f32) -> Vector2<f32> {
    let (s, c) = a.sin_cos();
    vec2(c, s)
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

impl From<Direction> for Vector2<f32> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::East => vec2(1.0, 0.0),
            Direction::North => vec2(0.0, 1.0),
            Direction::West => vec2(-1.0, 0.0),
            Direction::South => vec2(0.0, -1.0),
        }
    }
}

/// Raycast to the edge of the box bounded by points (0, 0) and (1, 1).
fn raycast_in_box(pos: Vector2<f32>, ray_unit: Vector2<f32>) -> Vector2<f32> {
    /// This is restricted to the case where both components of ray_unit
    /// are less than or equal to zero.
    #[inline(always)]
    fn towards_origin(pos: Vector2<f32>, ray_unit: Vector2<f32>) -> Vector2<f32> {
        match (ray_unit.x == 0.0, ray_unit.y == 0.0) {
            (true, true) => panic!("Cannot raycast with zero-valued ray"),
            (true, false) => return vec2(pos.x, 0.0),
            (false, true) => return vec2(0.0, pos.y),
            (false, false) => (),
        }

        let x_int = pos.x - (ray_unit.x / ray_unit.y) * pos.y;
        let y_int = pos.y - (ray_unit.y / ray_unit.x) * pos.x;

        if x_int < 0.0 {
            vec2(0.0, y_int)
        } else {
            vec2(x_int, 0.0)
        }
    }

    if ray_unit.x > 0.0 {
        let o = raycast_in_box(vec2(1.0 - pos.x, pos.y), vec2(-ray_unit.x, ray_unit.y));
        return vec2(1.0 - o.x, o.y);
    }
    if ray_unit.y > 0.0 {
        let o = towards_origin(vec2(pos.x, 1.0 - pos.y), vec2(ray_unit.x, -ray_unit.y));
        return vec2(o.x, 1.0 - o.y);
    }

    towards_origin(pos, ray_unit)
}

fn box_hit_pos_to_box_side(hit: Vector2<f32>) -> Direction {
    if hit.x == 1.0 {
        Direction::West
    } else if hit.x == 0.0 {
        Direction::East
    } else if hit.y == 1.0 {
        Direction::South
    } else if hit.y == 0.0 {
        Direction::North
    } else {
        panic!("Not a box side!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{assert_ulps_eq, vec2, InnerSpace, Vector2};
    use rstest::rstest;

    #[rstest]
    #[case(vec2(0.75, 0.5), vec2(-1.0, 0.0), vec2(0.0, 0.5))]
    #[case(vec2(0.75, 0.5), vec2(0.0, -1.0), vec2(0.75, 0.0))]
    #[case(vec2(0.5, 0.5), vec2(-1.0, 0.5), vec2(0.0, 0.75))]
    #[case(vec2(0.25, 0.5), vec2(1.0, -0.25), vec2(1.0, 0.3125))]
    #[case(vec2(0.5, 0.25), vec2(1.0, 1.0), vec2(1.0, 0.75))]
    #[case(vec2(0.5, 0.5), vec2(1.0, 1.0), vec2(1.0, 1.0))]
    fn test_raycast_in_box(
        #[case] pos: Vector2<f32>,
        #[case] ray: Vector2<f32>,
        #[case] expected: Vector2<f32>,
    ) {
        let ray_unit = ray.normalize();

        let actual = raycast_in_box(pos, ray_unit);

        assert_ulps_eq!(actual, expected)
    }
}
