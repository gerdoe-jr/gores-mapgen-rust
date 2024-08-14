use crate::{
    map::Map, mutators::Mutation, position::{Direction, Vector2}, random::Random
};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalWaypoints {
    pub waypoints: Vec<(f32, f32)>,
}

impl NormalWaypoints {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WalkerState {
    Idling,
    Processing {
        /// current position of brush
        position: Vector2,
        /// current walker's step
        step: usize, 
        /// direction of movement
        direction: Direction,
        /// current waypoint's index
        waypoint: usize,
    },
    Finished
}

#[derive(Debug)]
pub struct Walker {
    prev_state: WalkerState,
    curr_state: WalkerState,
    next_state: WalkerState,
    raw_waypoints: NormalWaypoints,

    prng: Random,
    map_bounds: Vector2,
}

impl Walker {
    pub fn new(prng: Random) -> Self {
        Self {
            prev_state: WalkerState::Idling,
            curr_state: WalkerState::Idling,
            next_state: WalkerState::Idling,
            raw_waypoints: NormalWaypoints::new(),
            prng,
            map_bounds: Vector2::new(0, 0),
        }
    }

    /// not used
    /// use for "multi-seed" generation
    /// do. never. call. seed. cum.
    /// it's not a "multi-cum"
    pub fn from_state(state: WalkerState) -> Self {
        Self {
            prev_state: state,
            curr_state: state,
            next_state: state,
            raw_waypoints: NormalWaypoints::new(),
            prng: Random::new(0),
            map_bounds: Vector2::new(0, 0)
        }
    } 

    pub fn previous_state(&self) -> &WalkerState {
        &self.prev_state
    }

    pub fn current_state(&self) -> &WalkerState {
        &self.curr_state
    }

    pub fn next_state(&self) -> &WalkerState {
        &self.next_state
    }

    pub fn next_state_mut(&mut self) -> &mut WalkerState {
        &mut self.next_state
    }

    pub fn reset(&mut self) {
        self.prev_state = WalkerState::Idling;
        self.curr_state = WalkerState::Idling;
        self.next_state = WalkerState::Idling;

        self.map_bounds = Vector2::new(0, 0);

        self.prng.reset();
    }

    pub fn set_random(&mut self, prng: Random) -> &mut Self {
        self.prng = prng;

        self
    }

    pub fn set_waypoints(&mut self, raw_waypoints: NormalWaypoints) -> &mut Self {
        self.raw_waypoints = raw_waypoints;

        self
    }

    pub fn set_bounds(&mut self, width: usize, height: usize) -> &mut Self {
        self.map_bounds.x = width;
        self.map_bounds.y = height;

        self
    }

    pub fn get_bounds(&self) -> Vector2 {
        self.map_bounds
    }

    pub fn step(&mut self, mutator: &mut impl Mutation, map: &mut Map) -> bool {
        mutator.mutate_step(self, map);

        self.prev_state = self.curr_state; // take previous state
        self.curr_state = self.next_state; // take next state
        self.next_state = self.curr_state; // repeat next state

        self.curr_state == WalkerState::Finished
    }
}
