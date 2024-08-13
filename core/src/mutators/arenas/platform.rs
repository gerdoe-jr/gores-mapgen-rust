use crate::{map::Map, mutators::{Mutation, MutationState}, position::Vector2, walker::Walker};

pub struct PlatformArena {
    pub shift: Vector2,
    pub size: usize,
    pub connected: bool,
    pub steps: usize
}

impl Mutation for PlatformArena {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState {
        MutationState::Finished
    }
}