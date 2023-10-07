use auto_impl::auto_impl;
use cgmath::{vec2, MetricSpace, Vector2};

use crate::util::Direction;

#[derive(Debug, Clone)]
pub struct CameraParams {
    pub pos: Vector2<f32>,

    pub facing_unit: Vector2<f32>,

    pub n_rays: usize,

    pub max_dist: f32,

    /// The projection plane is 1 unit away from the camera. Adjusting this value
    /// allows you to adjust the FOV.
    pub projection_plane_width: f32,
}

#[auto_impl(&, Box, Arc)]
pub trait RaycastableWorld {
    /// Given a grid coordinate, return if there is an object there or not.
    fn exists(&self, pos: (isize, isize)) -> bool;
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub hit_pos: Vector2<f32>,
    pub wall: Vector2<usize>,
    pub wall_side: Direction,
}

/// Raycast along a plane.
///
/// Facing must be a unit vector.
pub fn raycast_camera(
    world: impl RaycastableWorld,
    params: &CameraParams,
) -> Vec<Option<RaycastHit>> {
    let rays = gen_rays(
        params.facing_unit,
        params.projection_plane_width,
        params.n_rays,
    );

    rays.map(|ray| raycast(&world, params.pos, ray, params.max_dist))
        .collect()
}

/// Perform a single raycast from the given position along the given ray.
pub fn raycast(
    world: impl RaycastableWorld,
    pos: Vector2<f32>,
    ray: Vector2<f32>,
    max_dist: f32,
) -> Option<RaycastHit> {
    let max_dist_2 = max_dist * max_dist;

    let mut march_pos = pos;
    let mut this_grid = march_pos.map(|x| x.floor()).cast::<isize>().unwrap();

    loop {
        if march_pos.distance2(pos) > max_dist_2 {
            return None;
        }

        let box_offset = this_grid.cast().unwrap();
        let box_pos = march_pos - box_offset;
        let (box_hit_pos, outgoing_dir) = raycast_in_box(box_pos, ray);
        let hit_pos = box_hit_pos + box_offset;

        let probe_cell = this_grid + Vector2::<isize>::from(outgoing_dir);

        if world.exists(probe_cell.into()) {
            return Some(RaycastHit {
                hit_pos,
                wall: probe_cell.cast().unwrap(),
                wall_side: -outgoing_dir,
            });
        } else {
            march_pos = hit_pos;
            this_grid = probe_cell;
        }
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
    // Calculate the perpendicular of the unit vector, to the left.
    let facing_left_unit = vec2(facing_unit.y, -facing_unit.x);

    // Calculate the projection plane's leftmost point.
    let pp_leftmost_point = facing_unit + (projection_plane_width / 2.0) * facing_left_unit;

    (0..n_rays).map(move |i| {
        pp_leftmost_point - (i as f32 * projection_plane_width / n_rays as f32) * facing_left_unit
    })
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
    use crate::world::ArrayWorld;

    use super::*;
    use cgmath::{assert_ulps_eq, vec2, Vector2};
    use ndarray::array;
    use rstest::rstest;

    fn example_world() -> ArrayWorld {
        let data = array![
            [1, 1, 3, 1, 1, 1, 1, 1, 1],
            [3, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 0, 0, 0, 0, 0, 0, 1],
            [2, 0, 0, 0, 0, 0, 3, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 2, 1, 1, 1, 1, 1, 2, 1],
        ];
        ArrayWorld::from(data.map(|x| *x != 0))
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
        RaycastHit {
            hit_pos: vec2(1.0, 2.5),
            wall: vec2(0, 2),
            wall_side: Direction::East
        }
    )]
    #[case(
        (vec2(1.05, 1.05), vec2(-0.5, -1.0)),
        RaycastHit {
            hit_pos: vec2(1.025, 1.0),
            wall: vec2(1, 0),
            wall_side: Direction::North
        }
    )]
    #[case(
        (vec2(3.5, 3.5), vec2(-1.0, -1.0)),
        RaycastHit {
            hit_pos: vec2(1.0, 1.0),
            wall: vec2(1, 0),
            wall_side: Direction::North
        }
    )]
    fn raycast_edge(#[case] ray: (Vector2<f32>, Vector2<f32>), #[case] expected: RaycastHit) {
        let (pos, ray) = ray;
        let world = example_world();

        let result = raycast(world, pos, ray, 100.0).unwrap();

        assert_eq!(result.wall_side, expected.wall_side);
        assert_eq!(result.wall, expected.wall);
        assert_ulps_eq!(result.hit_pos, expected.hit_pos)
    }
}
