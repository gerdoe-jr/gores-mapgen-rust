use std::f32::consts::PI;

use ndarray::{Array1, ArrayView1};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    #[default]
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Direction {
    pub fn prev(&self) -> Self {
        match &self {
            Self::Up => Self::Left,
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
        }
    }

    pub fn next(&self) -> Self {
        match &self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn backwards(&self) -> Self {
        self.next().next()
    }
}

impl From<usize> for Direction {
    fn from(id: usize) -> Direction {
        match id {
            0 => Self::Up,
            1 => Self::Right,
            2 => Self::Down,
            3 => Self::Left,
            _ => Default::default(),
        }
    }
}

pub type Vector2 = Array1<f32>;
pub type VectorView2<'a> = ArrayView1<'a, f32>;

#[inline]
pub fn get_x(value: VectorView2) -> f32 {
    value[[0]]
}

#[inline]
pub fn get_y(value: VectorView2) -> f32 {
    value[[1]]
}

#[inline]
pub fn as_index(value: VectorView2) -> [usize; 2] {
    [value[[0]] as usize, value[[1]] as usize]
}

pub fn from_raw(value: (f32, f32), scale_factor: f32) -> Vector2 {
    Vector2::from(vec![(value.0 * scale_factor), (value.1 * scale_factor)])
}

pub fn euclidian(lhs: VectorView2, rhs: VectorView2) -> f32 {
    let x = lhs[[0]] - rhs[[0]];
    let y = lhs[[1]] - rhs[[1]];

    length(Vector2::from(vec![x, y]).view())
}

#[inline]
pub fn manhattan(x: VectorView2) -> f32 {
    x.fold(0., |acc, elem| acc + elem)
}

#[inline]
pub fn length(value: VectorView2) -> f32 {
    (value[[0]].powi(2) + value[[1]].powi(2)).sqrt()
}

#[inline]
pub fn normalize(mut value: Vector2) -> Vector2 {
    let len = length(value.view());
    value.mapv_inplace(|e| e / len);

    value
}

#[inline]
pub fn angle(value: VectorView2) -> f32 {
    f32::atan2(value[[0]], value[[1]])
}

pub fn angle_direction(angle: f32) -> Direction {
    let angle = (angle + 2.0 * PI + PI / 2.0) % (PI * 2.0);

    if (0.0..=PI / 2.0).contains(&angle) {
        Direction::Right
    } else if (PI / 2.0..=PI).contains(&angle) {
        Direction::Up
    } else if (PI..=3.0 * PI / 2.0).contains(&angle) {
        Direction::Left
    } else if (3.0 * PI / 2.0..=2.0 * PI).contains(&angle) {
        Direction::Down
    } else {
        panic!()
    }
}

#[inline]
pub fn direction(value: VectorView2) -> Direction {
    angle_direction(angle(value))
}

pub fn shift_by_direction(value: &mut Vector2, shift: f32, direction: Direction) {
    match direction {
        Direction::Up => value[[1]] -= shift,
        Direction::Right => value[[0]] += shift,
        Direction::Down => value[[1]] += shift,
        Direction::Left => value[[0]] -= shift,
    }
}

pub fn straight_neighbors(pos: VectorView2) -> Vec<Vector2> {
    let cur = pos.to_vec();
    let mut neighbors: Vec<Vector2> = vec![
        cur.clone().into(),
        cur.clone().into(),
        cur.clone().into(),
        cur.into(),
    ];
    for i in 0..4 {
        shift_by_direction(&mut neighbors[i], 1.0, Direction::from(i));
    }

    neighbors
}

pub fn all_neighbors(pos: VectorView2) -> Vec<Vector2> {
    let mut neighbors = straight_neighbors(pos);

    neighbors.extend(straight_neighbors(pos));

    for i in 0..4 {
        shift_by_direction(&mut neighbors[i + 4], 1.0, Direction::from(i).next());
    }

    neighbors
}
