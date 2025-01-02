use crate::{
    brush::Brush,
    mutations::{MutationState, Mutator},
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PulseBrushMutation {
    pub value_border: usize, // from, to
    pub value_climax: usize,
    pub normal_peak: f32, // 0 to 1
    pub overall_steps: usize,
    steps: usize,
}

impl PulseBrushMutation {
    pub fn new(value_min: usize, value_max: usize, overall_steps: usize, normal_peak: f32) -> Self {
        println!("pulse: steps: {}", overall_steps);
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
        if self.steps == 0 {
            return MutationState::Finished;
        }

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

        println!("[pulse]\tslope\t{}", slope);
        mutant.apply_scale(slope);

        self.steps -= 1;

        MutationState::Processing
    }

    fn reset(&mut self) {
        self.steps = self.overall_steps;
    }
}
