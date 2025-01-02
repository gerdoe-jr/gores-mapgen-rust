use crate::{
    brush::Brush,
    mutations::{MutationState, Mutator},
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TransitionBrushMutation {
    pub value_from: usize,
    pub value_to: usize,
    pub overall_steps: usize,
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
        if self.steps == 0 {
            return MutationState::Finished;
        }

        let diff = (self.value_from as f32 - self.value_to as f32).abs();
        let current_step = self.overall_steps - self.steps;
        let slope = current_step as f32 / self.overall_steps as f32 * diff + self.value_from as f32;

        println!("[trans]\tslope\t{}", slope);
        mutant.apply_scale(slope);

        self.steps -= 1;

        MutationState::Processing
    }

    fn reset(&mut self) {
        self.steps = self.overall_steps;
    }
}
