use crate::{
    walker::Walker,
    mutations::{MutationState, Mutator},
};

pub struct RightWalkerMutation;

impl RightWalkerMutation {
    pub fn new() -> Self { Self }
}

impl Mutator<Walker> for RightWalkerMutation {
    fn mutate(&mut self, mutant: &mut Walker) -> MutationState {
        let needed_state = *mutant.preferred_state();

        mutant.set_next_direction(needed_state.direction.next());
        mutant.set_next_waypoint(needed_state.waypoint);
        
        MutationState::Finished
    }
}
