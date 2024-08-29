use crate::{
    brush::Brush,
    mutations::{MutationState, Mutator},
};

pub struct TransitionBrushMutation {
    value_from: usize,
    value_to: usize,
    overall_steps: usize,
    steps: usize,
}

impl TransitionBrushMutation {
    pub fn new(value_from: usize, value_to: usize, overall_steps: usize) -> Self {
        Self {
            value_from,
            value_to,
            overall_steps,
            steps: overall_steps,
        }
    }
}

impl Mutator<Brush> for TransitionBrushMutation {
    fn mutate(&mut self, mutant: &mut Brush) -> MutationState {
        let diff = (self.value_from as f32 - self.value_to as f32).abs();
        let current_step = self.overall_steps - self.steps;
        let slope = current_step as f32 / self.overall_steps as f32 * diff + self.value_from as f32;

        println!("transition slope: {}", slope);
        mutant.apply_scale(slope);

        self.steps -= 1;

        if self.steps == 0 {
            return MutationState::Finished;
        }

        MutationState::Processing
    }
}
