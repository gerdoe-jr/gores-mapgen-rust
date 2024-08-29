use crate::{
    mutations::{MutationState, Mutator}, position::Direction, random::Random, walker::Walker
};

pub struct RandomWalkerMutation {
    prng: Random,
}

impl RandomWalkerMutation {
    pub fn new(prng: Random) -> Self {
        Self { prng }
    }
}

impl Mutator<Walker> for RandomWalkerMutation {
    fn mutate(&mut self, mutant: &mut Walker) -> MutationState {
        let random_direction = self.prng.gen_u64() as usize % 4;
        let random_waypoint = self.prng.gen_u64() as usize % mutant.get_waypoints().len();

        let random_direction = Direction::from(random_direction);

        mutant.set_next_direction(random_direction);
        mutant.set_next_waypoint(random_waypoint);

        MutationState::Finished
    }
}
