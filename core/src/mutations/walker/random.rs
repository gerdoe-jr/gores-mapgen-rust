use crate::{
    mutations::{MutationState, Mutator},
    position::Direction,
    random::{Random, Seed},
    walker::Walker,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RandomWalkerMutation {
    pub seed: Seed,
    pub overall_steps: usize,

    prng: Random,
    steps: usize
}

impl RandomWalkerMutation {
    pub fn new(overall_steps: usize, seed: Seed) -> Self {
        Self {
            seed,
            overall_steps,
            prng: Random::new(seed),
            steps: overall_steps
        }
    }
}

impl Mutator<Walker> for RandomWalkerMutation {
    fn mutate(&mut self, mutant: &mut Walker) -> MutationState {
        if self.steps == 0 {
            return MutationState::Finished;
        }

        let random_direction = self.prng.gen_u64() as usize % 4;
        let random_waypoint = self.prng.gen_u64() as usize % mutant.get_waypoints().len();

        let random_direction = Direction::from(random_direction);

        mutant.set_next_direction(random_direction);
        mutant.set_next_waypoint(random_waypoint);

        self.steps -= 1;

        MutationState::Processing
    }
}
