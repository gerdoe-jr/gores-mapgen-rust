use crate::{
    brush::Brush,
    mutations::{MutationState, Mutator},
};

pub struct PulseBrushMutation {
    value_border: usize, // from, to
    value_climax: usize,
    normal_peak: f32, // 0 to 1
    overall_steps: usize,
    steps: usize,
}

impl PulseBrushMutation {
    pub fn new(value_min: usize, value_max: usize, overall_steps: usize, normal_peak: f32) -> Self {
        Self {
            value_border: value_min,
            value_climax: value_max,
            overall_steps,
            normal_peak,
            steps: overall_steps,
        }
    }
}

impl Mutator<Brush> for PulseBrushMutation {
    fn mutate(&mut self, mutant: &mut Brush) -> MutationState {
        let diff = (self.value_border as f32 - self.value_climax as f32).abs();
        let current_step = self.overall_steps - self.steps;
        let overall_steps_until_peak = (self.overall_steps as f32 * self.normal_peak) as usize;

        let slope = if current_step < overall_steps_until_peak {
            current_step as f32 / overall_steps_until_peak as f32 * diff + self.value_border as f32
        } else if current_step == overall_steps_until_peak {
            self.value_climax as f32
        } else {
            self.steps as f32 / (self.overall_steps - overall_steps_until_peak) as f32 * diff
                + self.value_border as f32
        };

        println!("pulse slope: {}", slope);
        mutant.apply_scale(slope);

        self.steps -= 1;

        if self.steps == 0 {
            return MutationState::Finished;
        }

        MutationState::Processing
    }
}
