use cgmath::{Vector2, vec2, Zero, One};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East,
    North,
    West,
    South,
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
