pub mod brush;
pub mod map;
pub mod walker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MutationState {
    Processing,
    Finished,
}

pub trait Mutator<T> {
    fn mutate(&mut self, mutant: &mut T) -> MutationState;
    fn reset(&mut self);
}
