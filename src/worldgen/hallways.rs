use std::collections::VecDeque;

use cgmath::BaseNum;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::util::{Axis, Line, Rectangle};

pub struct RbspParams {
    /// Rooms with a width or height shorter than this size will never be partitioned.
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

        if usize::min(r.w, r.h) <= params.min_room_len {
            // Cannot partition this room any further, so place in "acceptable" set
            safe.push(r);
            continue;
        }

        let avged_size: f32 = (r.w as f32 * r.h as f32).powf(0.5);
        if avged_size <= params.max_room_len as f32 && rng.gen::<f32>() < params.p_keep_rooms {
            safe.push(r);
            continue;
        }

        let axis = pick_axis(rng, &r, params.k_deoblongification);
        let partition_pct = rng.gen::<f32>() * 0.8 + 0.1;
        let (r1, p, r2) = make_partition(&r, partition_pct, axis);

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
    let h_weight = rect.h.to_f32().unwrap().powf(k_deoblongification);
    let w_weight = rect.w.to_f32().unwrap().powf(k_deoblongification);

    let p_horiz = w_weight / (w_weight + h_weight);

    if rng.gen::<f32>() < p_horiz {
        Axis::Horizontal
    } else {
        Axis::Vertical
    }
}

pub fn make_partition(
    r: &Rectangle<isize, usize>,
    ratio: f32,
    axis: Axis,
) -> (Rectangle<isize, usize>, Line, Rectangle<isize, usize>) {
    match axis {
        Axis::Horizontal => {
            let w1 = (r.w as f32 * ratio) as usize;
            let r1 = Rectangle {
                x: r.x,
                y: r.y,
                w: w1,
                h: r.h,
            };
            let r2 = Rectangle {
                x: r.x + w1 as isize,
                y: r.y,
                w: r.w - w1,
                h: r.h,
            };
            let p = Line {
                x: r.x + w1 as isize,
                y: r.y,
                length: r.h,
                axis: Axis::Vertical,
            };
            (r1, p, r2)
        }
        Axis::Vertical => {
            let h1 = (r.h as f32 * ratio) as usize;
            let r1 = Rectangle {
                x: r.x,
                y: r.y,
                w: r.w,
                h: h1,
            };
            let r2 = Rectangle {
                x: r.x,
                y: r.y + h1 as isize,
                w: r.w,
                h: r.h - h1,
            };
            let p = Line {
                x: r.x,
                y: r.y + h1 as isize,
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
    use crate::util::{Line, Rectangle};

    use super::*;

    #[test]
    fn do_make_partition() {
        let r = make_partition(
            &Rectangle {
                x: 2,
                y: 5,
                w: 10,
                h: 8,
            },
            0.5,
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
            0.5,
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
