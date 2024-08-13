use crate::{
    map::Map,
    mutators::{Mutation, MutationState},
    random::RandomDist,
    walker::Walker,
};

pub struct AnyMutation {
    mutations_indeces: Vec<RandomDist<usize>>,
}

impl AnyMutation {
    pub fn from_possible_mutations(mutations_indeces: Vec<RandomDist<usize>>) -> Self {
        Self { mutations_indeces }
    }
}

impl Mutation for AnyMutation {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState {
        todo!()
    }
}
