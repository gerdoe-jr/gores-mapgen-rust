use crate::{map::Map, mutators::{Mutation, MutationState}, walker::Walker};

pub struct TransitionMutation {
    pub value_from: usize,
    pub value_to: usize,
    pub overall_steps: usize
}

impl Mutation for TransitionMutation {
    fn mutate_step(&mut self, walker: &mut Walker, map: &mut Map) -> MutationState {

        let slope = (min_size as f32 - max_size as f32) / fade_steps as f32;
        let kernel_size_f = (step as f32) * slope + max_size as f32;
        let kernel_size = kernel_size_f.floor() as usize;
        self.inner_kernel = Kernel::new(kernel_size, 0.0);
        self.outer_kernel = Kernel::new(kernel_size + 2, 0.0);
        
        MutationState::Finished
    }
}