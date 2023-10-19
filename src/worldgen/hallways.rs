use cgmath::BaseNum;
use rand::{seq::IteratorRandom, Rng};

use crate::util::{Axis, Line, Rectangle};

pub struct RbspParams {
    /// Rooms with a width or height shorter than this size will never be created.
    pub min_room_len: usize,

    /// Rooms with an area larger than a square of size will always be partitioned.
    pub max_room_len: usize,

    /// A probability in [0, 1] determining if a room in [min room len, max room len] should
    /// be kept.
    pub p_keep_rooms: f32,

    /// A factor in (0, inf) controlling how much the partitioner prefers making
    /// rooms more square than oblong.
    ///
    /// Given a room that has a long axis and a short axis:
    /// - k > 1 prefers cutting rooms along the long axis, making them less oblong.
    /// - k < 1 prefers cutting rooms along the short axis, making them more oblong.
    /// - k = 1 has no preference.
    ///
    /// Square will not be affected by this parameter.
    pub k_deoblongification: f32,
}

/// random binary space partition
pub fn rbsp(
    rng: &mut impl Rng,
    full_rect: Rectangle<isize, usize>,
    params: RbspParams,
) -> (Vec<Rectangle<isize, usize>>, Vec<Line>) {
    let mut examining = vec![full_rect];
    let mut safe = vec![];
    let mut partitions = vec![];

    loop {
        let Some(i) = (0..examining.len()).choose(rng) else {
            break;
        };
        let r = examining.remove(i);

        if usize::min(r.w, r.h) / 2 <= params.min_room_len {
            // Cannot partition this room any further without going less than min_room_len,
            // so place in "acceptable" set
            safe.push(r);
            continue;
        }

        let avged_size: f32 = (r.w as f32 * r.h as f32).powf(0.5);
        if avged_size <= params.max_room_len as f32 && rng.gen::<f32>() < params.p_keep_rooms {
            safe.push(r);
            continue;
        }

        let axis = pick_axis(rng, &r, params.k_deoblongification);
        println!("{}, {}", r.axis_length(axis), params.min_room_len);
        let distribution_width = r.axis_length(axis) - params.min_room_len + 1;
        let partition_offset = rng.gen_range(0..distribution_width) + params.min_room_len / 2;
        let (r1, p, r2) = make_partition(&r, partition_offset, axis);

        examining.push(r1);
        examining.push(r2);
        partitions.push(p);
    }

    println!("{safe:#?}");

    (safe, partitions)
}

fn pick_axis<O: BaseNum, L: BaseNum>(
    rng: &mut impl Rng,
    rect: &Rectangle<O, L>,
    k_deoblongification: f32,
) -> Axis {
    let w_weight = rect.w.to_f32().unwrap().powf(k_deoblongification);
    let h_weight = rect.h.to_f32().unwrap().powf(k_deoblongification);

    let p_horiz = w_weight / (w_weight + h_weight);

    if rng.gen::<f32>() < p_horiz {
        Axis::Horizontal
    } else {
        Axis::Vertical
    }
}

pub fn make_partition(
    r: &Rectangle<isize, usize>,
    offset: usize,
    axis: Axis,
) -> (Rectangle<isize, usize>, Line, Rectangle<isize, usize>) {
    match axis {
        Axis::Horizontal => {
            let r1 = Rectangle {
                x: r.x,
                y: r.y,
                w: offset,
                h: r.h,
            };
            let r2 = Rectangle {
                x: r.x + offset as isize,
                y: r.y,
                w: r.w - offset,
                h: r.h,
            };
            let p = Line {
                x: r.x + offset as isize,
                y: r.y,
                length: r.h,
                axis: Axis::Vertical,
            };
            (r1, p, r2)
        }
        Axis::Vertical => {
            let r1 = Rectangle {
                x: r.x,
                y: r.y,
                w: r.w,
                h: offset,
            };
            let r2 = Rectangle {
                x: r.x,
                y: r.y + offset as isize,
                w: r.w,
                h: r.h - offset,
            };
            let p = Line {
                x: r.x,
                y: r.y + offset as isize,
                length: r.w,
                axis: Axis::Horizontal,
            };
            (r1, p, r2)
        }
    }
}

pub fn partition<O, L>(
    rect: Rectangle<O, L>,
    divider_percents: impl IntoIterator<Item = L>,
    axis: Axis,
) -> Vec<Rectangle<O, L>>
where
    O: BaseNum,
    L: BaseNum,
{
    let walls_percents = [L::zero()]
        .into_iter()
        .chain(divider_percents.into_iter())
        .chain([L::one()].into_iter());

    let mut wall_offsets = walls_percents
        .map(|p| match axis {
            Axis::Vertical => rect.h,
            Axis::Horizontal => rect.w
        } * p)
        .collect::<Vec<_>>();
    wall_offsets.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let neighbors = wall_offsets.iter().skip(1).zip(wall_offsets.iter());

    neighbors
        .map(|(l, r)| match axis {
            Axis::Vertical => Rectangle {
                x: rect.x,
                y: rect.y + O::from(*l).unwrap(),
                w: rect.w,
                h: *l - *r,
            },
            Axis::Horizontal => Rectangle {
                x: rect.x + O::from(*l).unwrap(),
                y: rect.y,
                w: *l - *r,
                h: rect.h,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rand::{rngs::SmallRng, SeedableRng};

    use crate::util::{Line, Rectangle};

    use super::*;

    #[test]
    fn generation_smoke_test() {
        for i in 0..1000 {
            let mut rng = SmallRng::seed_from_u64(i);
            rbsp(
                &mut rng,
                Rectangle {
                    x: 0,
                    y: 0,
                    w: 512,
                    h: 512,
                },
                RbspParams {
                    min_room_len: 5,
                    max_room_len: 80,
                    p_keep_rooms: 0.3,
                    k_deoblongification: 5.0,
                },
            );
        }
    }

    #[test]
    fn do_make_partition() {
        let r = make_partition(
            &Rectangle {
                x: 2,
                y: 5,
                w: 10,
                h: 8,
            },
            5,
            Axis::Horizontal,
        );

        let expected = (
            Rectangle {
                x: 2,
                y: 5,
                w: 5,
                h: 8,
            },
            Line {
                x: 7,
                y: 5,
                length: 8,
                axis: Axis::Vertical,
            },
            Rectangle {
                x: 7,
                y: 5,
                w: 5,
                h: 8,
            },
        );

        assert_eq!(r, expected);
    }

    #[test]
    fn do_make_partition_vert() {
        let r = make_partition(
            &Rectangle {
                x: 2,
                y: 5,
                w: 10,
                h: 8,
            },
            4,
            Axis::Vertical,
        );

        let expected = (
            Rectangle {
                x: 2,
                y: 5,
                w: 10,
                h: 4,
            },
            Line {
                x: 2,
                y: 9,
                length: 10,
                axis: Axis::Horizontal,
            },
            Rectangle {
                x: 2,
                y: 9,
                w: 10,
                h: 4,
            },
        );

        assert_eq!(r, expected);
    }
}
