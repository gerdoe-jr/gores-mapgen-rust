use crate::{
    mutations::{MutationState, Mutator},
    walker::Walker,
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct BackwardsWalkerMutation {
    pub overall_steps: usize,
    steps: usize,
}

impl BackwardsWalkerMutation {
    pub fn new(overall_steps: usize) -> Self {
        Self {
            overall_steps,
            steps: overall_steps,
        }
    }
}

impl Mutator<Walker> for BackwardsWalkerMutation {
    fn mutate(&mut self, mutant: &mut Walker) -> MutationState {
        if self.steps == 0 {
            return MutationState::Finished;
        }

        let needed_state = *mutant.preferred_state();

        mutant.set_next_direction(needed_state.direction.backwards());
        mutant.set_next_waypoint(needed_state.waypoint);

        self.steps -= 1;

        MutationState::Finished
    }

    fn reset(&mut self) {
        self.steps = self.overall_steps;
    }
}
