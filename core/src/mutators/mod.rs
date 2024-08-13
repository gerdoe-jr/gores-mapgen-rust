pub mod arenas;
pub mod stages;
pub mod any;

use crate::{map::Map, walker::Walker};

pub enum MutationState {
    Processing,
    Finished,
}

pub trait Mutation {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState;
}