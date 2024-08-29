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
}

pub struct Mutation<T> {
    finished: bool,
    mutator: Box<dyn Mutator<T>>,
}

impl<T> Mutation<T> {
    pub fn new(mutator: impl Mutator<T> + 'static) -> Self {
        Self {
            finished: false,
            mutator: Box::new(mutator),
        }
    }

    pub fn mutate(&mut self, mutant: &mut T) {
        if !self.finished {
            self.finished = self.mutator.mutate(mutant) == MutationState::Finished;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
