use crate::map::Map;
use std::usize;

// using my own position vector to meet ndarray's indexing standard using usize
//
// while glam has nice performance benefits, the amount of expensive operations
// on the position vector will be very limited, so this should be fine..
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector2 {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    #[default]
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Vector2 {
    pub fn new(x: usize, y: usize) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn as_index(&self) -> [usize; 2] {
        [self.x, self.y]
    }

    /// returns a new position shifted by some x and y value
    pub fn shifted_by(&self, x_shift: i32, y_shift: i32) -> Result<Vector2, &'static str> {
        let new_x = match x_shift >= 0 {
            true => self.x + (x_shift as usize),
            false => self
                .x
                .checked_sub((-x_shift) as usize)
                .ok_or("invalid shift")?,
        };

        let new_y = match y_shift >= 0 {
            true => self.y + y_shift as usize,
            false => self
                .y
                .checked_sub((-y_shift) as usize)
                .ok_or("invalid shift")?,
        };

        Ok(Vector2::new(new_x, new_y))
    }

    pub fn shift_in_direction(
        &mut self,
        shift: Direction,
        map: &Map,
    ) -> bool {
        if !self.is_shift_valid(shift, map) {
            return false;
        }

        match shift {
            Direction::Up => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
        }

        return true;
    }

    pub fn is_shift_valid(&self, shift: Direction, map: &Map) -> bool {
        match shift {
            Direction::Up => self.y > 0,
            Direction::Right => self.x < map.width() - 1,
            Direction::Down => self.y < map.height() - 1,
            Direction::Left => self.x > 0,
        }
    }

    pub fn get_greedy_shift(&self, goal: &Vector2) -> Direction {
        let x_diff = goal.x as isize - self.x as isize;
        let x_abs_diff = x_diff.abs();
        let y_diff = goal.y as isize - self.y as isize;
        let y_abs_diff = y_diff.abs();

        // check whether x or y is dominant
        if x_abs_diff > y_abs_diff {
            if x_diff.is_positive() {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if y_diff.is_positive() {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    /// squared euclidean distance between two Positions
    pub fn distance_squared(&self, rhs: Vector2) -> usize {
        self.x.abs_diff(rhs.x).saturating_pow(2) + self.y.abs_diff(rhs.y).saturating_pow(2)
    }

    /// returns a Vec with all possible shifts, sorted by how close they get
    /// towards the goal position
    pub fn get_rated_shifts(&self, goal: Vector2, map: &Map) -> [Direction; 4] {
        let mut shifts = [
            Direction::Left,
            Direction::Up,
            Direction::Right,
            Direction::Down,
        ];

        shifts.sort_by_cached_key(|&shift| {
            let mut shifted_pos = self.clone();
            if shifted_pos.shift_in_direction(shift, map) {
                shifted_pos.distance_squared(goal)
            } else {
                // assign maximum distance to invalid shifts
                // TODO: i could also return a vec and completly remove invalid moves?
                usize::MAX
            }
        });

        shifts
    }

    pub fn dot(&self) -> f32 {
        ((self.x.pow(2) + self.y.pow(2)) as f32).sqrt()
    }

    pub fn distance(&self, rhs: &Vector2) -> f32 {
        (((self.x - rhs.x).pow(2) + (self.y - rhs.y).pow(2)) as f32).sqrt()
    }
}
