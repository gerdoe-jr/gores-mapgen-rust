use crate::{
    generator,
    kernel::Kernel,
    map::{BlockType, Map, Overwrite},
    position::{Position, ShiftDirection},
    random::{Random, RandomDistConfig},
};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WalkerParams {
    /// probability for mutating inner radius
    pub inner_rad_mut_prob: f32,

    /// probability for mutating inner size
    pub inner_size_mut_prob: f32,

    /// probability for mutating outer radius
    pub outer_rad_mut_prob: f32,

    /// probability for mutating outer size
    pub outer_size_mut_prob: f32,

    /// probability weighting for random selection from best to worst towards next goal
    pub shift_weights: RandomDistConfig<ShiftDirection>,

    /// probability for doing the last shift direction again
    pub momentum_prob: f32,

    /// probabilities for (inner_kernel_size, probability)
    pub inner_size_probs: RandomDistConfig<usize>,

    /// probabilities for (outer_kernel_margin, probability)
    pub outer_margin_probs: RandomDistConfig<usize>,

    /// probabilities for (kernel circularity, probability)
    pub circ_probs: RandomDistConfig<f32>,

    /// enable pulse
    pub pulse: Option<Pulse>,

    /// number of initial walker steps to perform fading. Will fade from max to min kernel size.
    pub fade_steps: usize,

    /// initial max kernel size for fading
    pub fade_max_size: usize,

    /// goal min kernel size for fading
    pub fade_min_size: usize,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pulse {
    /// TODO:
    pub straight_delay: usize,
    pub corner_delay: usize,
    pub max_kernel_size: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Waypoints {
    pub waypoints: Vec<(f32, f32)>,
}

impl Waypoints {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
        }
    }

    pub fn with_map_bounds(&self, width: usize, height: usize) -> Vec<Position> {
        self.waypoints
            .iter()
            .map(|&(x, y)| Position {
                x: (x * width as f32) as usize,
                y: (y * height as f32) as usize,
            })
            .collect::<Vec<Position>>()
    }
}

#[derive(Debug)]
pub struct Walker {
    pub pos: Position,
    pub steps: usize,
    pub inner_kernel: Kernel,
    pub outer_kernel: Kernel,
    pub goal: Option<Position>,
    pub goal_index: usize,
    pub raw_waypoints: Waypoints,
    pub waypoints: Vec<Position>,

    /// indicates whether walker has reached the last waypoint
    pub finished: bool,

    pub steps_since_platform: usize,

    pub last_shift: Option<ShiftDirection>,

    /// counts how many steps the pulse constraints have been fulfilled
    pub pulse_counter: usize,

    pub prng: Random,
    pub params: WalkerParams,
}

impl Walker {
    /// walker accepts waypoints only after performing map bounds
    pub fn new(
        inner_kernel: Kernel,
        outer_kernel: Kernel,
        prng: Random,
        params: WalkerParams,
    ) -> Walker {
        Walker {
            pos: Position::new(0, 0),
            steps: 0,
            inner_kernel,
            outer_kernel,
            goal: None,
            goal_index: 0,
            raw_waypoints: Waypoints::new(),
            waypoints: Vec::new(),
            finished: false,
            steps_since_platform: 0,
            last_shift: None,
            pulse_counter: 0,
            prng,
            params,
        }
    }

    pub fn set_waypoints(&mut self, raw_waypoints: Waypoints) -> &mut Self {
        self.raw_waypoints = raw_waypoints;

        self
    }

    pub fn set_bounds(&mut self, width: usize, height: usize) -> &mut Self {
        self.waypoints = self.raw_waypoints.with_map_bounds(width, height);
        self.pos = self.waypoints[0];
        self.goal = Some(self.waypoints[0]);
        
        self
    }

    pub fn initial_pos(&self) -> Position {
        self.waypoints[0]
    }

    pub fn is_goal_reached(&self, waypoint_reached_dist: usize) -> Option<bool> {
        self.goal
            .as_ref()
            .map(|goal| goal.distance_squared(&self.pos) <= waypoint_reached_dist)
    }

    pub fn next_waypoint(&mut self) {
        if let Some(next_goal) = self.waypoints.get(self.goal_index + 1) {
            self.goal_index += 1;
            self.goal = Some(next_goal.clone());
        } else {
            self.finished = true;
            self.goal = None;
        }
    }

    /// will try to place a platform at the walkers position.
    /// If force is true it will enforce a platform.
    pub fn check_platform(
        &mut self,
        map: &mut Map,
        min_distance: usize,
        max_distance: usize,
    ) -> Result<(), &'static str> {
        self.steps_since_platform += 1;

        // Case 1: min distance is not reached -> skip
        if self.steps_since_platform < min_distance {
            return Ok(());
        }

        let walker_pos = self.pos.clone();

        // Case 2: max distance has been exceeded -> force platform using a room
        if self.steps_since_platform > max_distance {
            generator::generate_room(map, walker_pos.shifted_by(0, 6)?, 5, 3, None)?;
            self.steps_since_platform = 0;
            return Ok(());
        }

        // Case 3: min distance has been exceeded -> Try to place platform, but only if possible
        let area_empty = map.check_area_all(
            walker_pos.shifted_by(-3, -3)?,
            walker_pos.shifted_by(3, 2)?,
            BlockType::Empty,
        )?;
        if area_empty {
            map.set_area(
                walker_pos.shifted_by(-1, 0)?,
                walker_pos.shifted_by(1, 0)?,
                BlockType::Platform,
                Overwrite::ReplaceEmptyOnly,
            );
            self.steps_since_platform = 0;
        }

        Ok(())
    }

    pub fn probabilistic_step(&mut self, map: &mut Map) -> Result<(), &'static str> {
        if self.finished {
            return Err("Walker is finished");
        }

        // sample next shift
        let goal = self.goal.as_ref().ok_or("Error: Goal is None")?;
        let shifts = self.pos.get_rated_shifts(goal, map);

        let mut current_shift = self.prng.sample_shift(&shifts);

        let same_dir = match self.last_shift {
            Some(last_shift) => {
                // Momentum: re-use last shift direction
                if self.prng.with_probability(self.params.momentum_prob) {
                    current_shift = last_shift;
                }

                // check whether walker hasnt changed direction
                current_shift == last_shift
            }
            None => false,
        };

        // apply selected shift
        self.pos.shift_in_direction(current_shift, map)?;
        self.steps += 1;

        let mut pulsate = false;

        // perform pulse if direction changed and self.params constraints allows it
        if let Some(pulse) = &self.params.pulse {
            if (same_dir && self.pulse_counter > pulse.straight_delay)
                || (!same_dir && self.pulse_counter > pulse.corner_delay)
            {
                self.pulse_counter = 0; // reset pulse counter
                map.apply_kernel(
                    self,
                    &Kernel::new(&self.inner_kernel.size + 4, 0.0),
                    BlockType::Freeze,
                )?;
                map.apply_kernel(
                    self,
                    &Kernel::new(&self.inner_kernel.size + 2, 0.0),
                    BlockType::Empty,
                )?;

                pulsate = true;
            }

            if same_dir && self.inner_kernel.size <= pulse.max_kernel_size {
                self.pulse_counter += 1;
            } else {
                self.pulse_counter = 0;
            };
        }

        if !pulsate {
            map.apply_kernel(self, &self.outer_kernel, BlockType::Freeze)?;

            let empty = if self.steps < self.params.fade_steps {
                BlockType::EmptyReserved
            } else {
                BlockType::Empty
            };
            map.apply_kernel(self, &self.inner_kernel, empty)?;
        }

        // apply kernels

        self.last_shift = Some(current_shift.clone());

        Ok(())
    }

    /// fades kernel size from max_size to min_size for fade_steps
    fn set_fade_kernel(&mut self) {
        let slope = (self.params.fade_min_size as f32 - self.params.fade_max_size as f32)
            / self.params.fade_steps as f32;
        let kernel_size_f = (self.steps as f32) * slope + self.params.fade_max_size as f32;
        let kernel_size = kernel_size_f.floor() as usize;
        self.inner_kernel = Kernel::new(kernel_size, 0.0);
        self.outer_kernel = Kernel::new(kernel_size + 2, 0.0);
    }

    pub fn mutate_kernel(&mut self) {
        if self.steps <= self.params.fade_steps {
            self.set_fade_kernel();
            return;
        }

        let mut inner_size = self.inner_kernel.size;
        let mut inner_circ = self.inner_kernel.circularity;
        let mut outer_size = self.outer_kernel.size;
        let mut outer_circ = self.outer_kernel.circularity;
        let mut outer_margin = outer_size - inner_size;
        let mut modified = false;

        if self.prng.with_probability(self.params.inner_size_mut_prob) {
            inner_size = self.prng.sample_inner_kernel_size();
            modified = true;
        } else {
            self.prng.skip_n(2); // for some reason sampling requires two values?
        }

        if self.prng.with_probability(self.params.outer_size_mut_prob) {
            outer_margin = self.prng.sample_outer_kernel_margin();
            modified = true;
        } else {
            self.prng.skip_n(2);
        }

        if self.prng.with_probability(self.params.inner_rad_mut_prob) {
            inner_circ = self.prng.sample_circularity();
            modified = true;
        } else {
            self.prng.skip_n(2);
        }

        if self.prng.with_probability(self.params.outer_rad_mut_prob) {
            outer_circ = self.prng.sample_circularity();
            modified = true;
        } else {
            self.prng.skip_n(2);
        }

        outer_size = inner_size + outer_margin;

        // constraint 1: small circles must be fully rect
        if inner_size <= 3 {
            inner_circ = 0.0;
        }
        if outer_size <= 3 {
            outer_circ = 0.0;
        }

        // constraint 2: outer size cannot be smaller than inner
        assert!(outer_size >= inner_size); // this shoulnt happen -> crash!

        if modified {
            self.inner_kernel = Kernel::new(inner_size, inner_circ);
            self.outer_kernel = Kernel::new(outer_size, outer_circ);
        }
    }
}
