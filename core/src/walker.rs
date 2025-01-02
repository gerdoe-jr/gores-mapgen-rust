use crate::position::{euclidian, from_raw, straight_neighbors, Direction, Vector2, VectorView2};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalWaypoints {
    pub waypoints: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct WalkerState {
    /// direction of movement
    pub direction: Direction,
    /// current waypoint's index
    pub waypoint: usize,
}

#[derive(Debug)]
pub struct Walker {
    states: Vec<WalkerState>,
    preferred_state: WalkerState,
    next_state: Option<WalkerState>,

    current_step: usize,
    scale_factor: f32,

    raw_waypoints: Vec<(f32, f32)>,
}

impl Walker {
    pub fn new(scale_factor: f32) -> Self {
        Self {
            states: Vec::with_capacity(3),
            preferred_state: WalkerState::default(),
            next_state: None,
            current_step: 0,
            scale_factor,
            raw_waypoints: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.states.clear();
        self.preferred_state = WalkerState::default();
        self.next_state = None;
    }

    pub fn set_waypoints(&mut self, raw_waypoints: Vec<(f32, f32)>) -> &mut Self {
        self.raw_waypoints = raw_waypoints;

        self
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) -> &mut Self {
        self.scale_factor = scale_factor;

        self
    }

    pub fn get_waypoints(&self) -> &Vec<(f32, f32)> {
        &self.raw_waypoints
    }

    pub fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn get_current_step(&self) -> usize {
        self.current_step
    }

    pub fn set_next_direction(&mut self, direction: Direction) -> &mut Self {
        if let Some(state) = &mut self.next_state {
            state.direction = direction;
        } else {
            self.next_state = Some(WalkerState {
                direction,
                ..Default::default()
            })
        }

        self
    }

    pub fn set_next_waypoint(&mut self, waypoint: usize) -> &mut Self {
        if let Some(state) = &mut self.next_state {
            state.waypoint = waypoint;
        } else {
            self.next_state = Some(WalkerState {
                waypoint,
                ..Default::default()
            })
        }

        self
    }

    pub fn current_state(&self) -> &WalkerState {
        self.states.last().unwrap()
    }

    pub fn preferred_state(&self) -> &WalkerState {
        &self.preferred_state
    }

    pub fn step(&mut self, current_pos: VectorView2) -> usize {
        if self.next_state.is_none() {
            return 0;
        }

        if self.states.len() == self.states.capacity() {
            self.states.remove(0);
        }

        self.states.push(self.next_state.take().unwrap());

        let current_state = self.states.last().unwrap();

        if self.raw_waypoints.len() == current_state.waypoint + 1 {
            // we reached last waypoint, halt
            return 0;
        }

        // check if we reached waypoint
        let waypoint_pos = from_raw(
            self.raw_waypoints[current_state.waypoint],
            self.scale_factor,
        ) + Vector2::from(vec![200.0, 200.0]);

        println!("{}\t->\t{}", current_pos, waypoint_pos);

        let current_distance = euclidian(waypoint_pos.view(), current_pos.view());

        // TODO: make it configurable(?)
        if current_distance < 2.0 {
            // we reached waypoint, choose next

            self.preferred_state.waypoint += 1;
        }

        // calculate directions
        let min_neighbor = straight_neighbors(current_pos)
            .iter()
            .map(|n| euclidian(n.view(), waypoint_pos.view()))
            .enumerate()
            .min_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap())
            .unwrap();

        self.preferred_state.direction = Direction::from(min_neighbor.0);

        self.current_step += 1;

        self.current_step
    }
}
