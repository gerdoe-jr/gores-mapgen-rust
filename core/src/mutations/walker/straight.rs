use crate::{
    walker::Walker,
    mutations::{MutationState, Mutator},
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct StraightWalkerMutation{
    pub overall_steps: usize,
    steps: usize,
}

impl StraightWalkerMutation {
    pub fn new(overall_steps: usize) -> Self {
        Self {
            overall_steps,
            steps: overall_steps,
        }
    }
}

impl Mutator<Walker> for StraightWalkerMutation {
    fn mutate(&mut self, mutant: &mut Walker) -> MutationState {
        if self.steps == 0 {
            return MutationState::Finished;
        }

        let needed_state = *mutant.preferred_state();

        mutant.set_next_direction(needed_state.direction);
        mutant.set_next_waypoint(needed_state.waypoint);
        
        MutationState::Processing
    }
}
