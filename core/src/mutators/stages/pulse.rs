use crate::{map::Map, mutators::{Mutation, MutationState}, walker::Walker};

pub struct PulseMutation {
    pub value_min: usize, // from, to
    pub value_max: usize, // climax
    pub steps: usize
}

impl Mutation for PulseMutation {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState {
        MutationState::Finished
    }
}