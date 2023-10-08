use std::ops::{Add, Neg, Sub};

use cgmath::{vec2, One, Vector2, Zero};
use rand::{distributions::Standard, prelude::Distribution, seq::SliceRandom, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East = 0,
    North = 1,
    West = 2,
    South = 3,
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

impl Distribution<Direction> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        use Direction::*;
        *[East, North, West, South].choose(rng).unwrap()
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

#[derive(Debug, Clone)]
pub struct RelativeBounds<T> {
    pub forward: T,
    pub back: T,
    pub left: T,
    pub right: T,
}

#[derive(Debug, Clone, Copy)]
pub enum TurnDir {
    Left,
    Right,
}

pub trait Turnable
where
    Self: Sized,
{
    fn rotate(self, dir: TurnDir) -> Self;
    fn rotate_180(self) -> Self {
        self.rotate(TurnDir::Left).rotate(TurnDir::Left)
    }
}

impl Distribution<TurnDir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TurnDir {
        use TurnDir::*;
        *[Left, Right].choose(rng).unwrap()
    }
}

impl Turnable for Direction {
    fn rotate(self, dir: TurnDir) -> Self {
        match dir {
            TurnDir::Right => -self.rotate(TurnDir::Left),
            TurnDir::Left => match self {
                Direction::East => Direction::North,
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
            },
        }
    }
}

impl<T> Turnable for RelativeBounds<T> {
    fn rotate(self, dir: TurnDir) -> Self {
        match dir {
            TurnDir::Right => Self {
                forward: self.left,
                back: self.right,
                left: self.back,
                right: self.forward,
            },
            TurnDir::Left => Self {
                forward: self.right,
                back: self.left,
                left: self.forward,
                right: self.back,
            },
        }
    }
}

impl<T> RelativeBounds<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    pub fn translate(self, rhs: Vector2<T>) -> RelativeBounds<T> {
        RelativeBounds {
            forward: self.forward + rhs.y,
            back: self.back - rhs.y,
            left: self.left - rhs.x,
            right: self.right + rhs.x,
        }
    }
}

impl<T> RelativeBounds<T> {
    pub fn map<B>(self, f: impl Fn(T) -> B) -> RelativeBounds<B> {
        RelativeBounds {
            forward: f(self.forward),
            back: f(self.back),
            left: f(self.left),
            right: f(self.right),
        }
    }
}
