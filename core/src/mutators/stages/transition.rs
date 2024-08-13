use crate::{map::Map, mutators::{Mutation, MutationState}, walker::Walker};

pub struct TransitionMutation {
    pub value_from: usize,
    pub value_to: usize,
    pub overall_steps: usize
}

impl Mutation for TransitionMutation {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState {
        MutationState::Finished
    }
}