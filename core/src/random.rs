use rand::prelude::*;
use rand::rngs::SmallRng;
use rand_distr::uniform::{SampleRange, SampleUniform};
use rand_distr::WeightedAliasIndex;
use seahash::hash;

// only trivially copyable
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProbableValue<T: Copy>(pub f32, pub T);

impl<T: Copy> ProbableValue<T> {
    pub fn new(probability: f32, value: T) -> Self {
        Self(probability, value)
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomDistConfig<T: Copy> {
    pub values: Vec<ProbableValue<T>>,
}

impl<T: Copy> RandomDistConfig<T> {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
    pub fn from_values(values: Vec<ProbableValue<T>>) -> Self {
        Self { values }
    }

    pub fn get(&self, index: usize) -> ProbableValue<T> {
        self.values[index]
    }

    pub fn normalize_probs(&mut self) {
        let probs_sum: f32 = self.values.iter().map(|&ProbableValue(p, _)| p).sum();

        // TODO: does it really work? fp maths and comparison is a bit *float*
        if probs_sum == 1.0 {
            return; // skip if already normalized
        }

        let mapped_values = self.values.iter_mut().map(|ProbableValue(p, _)| p);

        // if all values are zero, set all to 1/n
        if probs_sum == 0.0 {
            let len = mapped_values.len();

            mapped_values.for_each(|p| *p = 1.0 / len as f32);
        // otherwise normalize, if required
        } else {
            mapped_values.for_each(|p| *p /= probs_sum);
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomDist<T: Copy> {
    pub config: RandomDistConfig<T>,
}

impl<T: Copy> RandomDist<T> {
    pub fn new() -> Self {
        Self {
            config: RandomDistConfig::new(),
        }
    }

    pub fn from_config(config: RandomDistConfig<T>) -> Self {
        Self { config }
    }

    pub fn weights(&self) -> WeightedAliasIndex<f32> {
        WeightedAliasIndex::new(
            self.config
                .values
                .iter()
                .map(|&ProbableValue(p, _)| p)
                .collect(),
        )
        .unwrap()
    }
}

pub type Seed = u64;

pub fn seed_from_str(seed: &str) -> Seed {
    hash(seed.as_bytes())
}

pub fn random_seed() -> Seed {
    SmallRng::from_entropy().next_u64()
}

#[derive(Debug, Clone)]
pub struct Random {
    prng: SmallRng,
}

impl Random {
    pub fn new(seed: Seed) -> Self {
        Random {
            prng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn sample_value<T: Copy>(&mut self, dist: &RandomDist<T>) -> T {
        dist.config.get(self.sample_index(dist)).1
    }

    pub fn sample_index<T: Copy>(&mut self, dist: &RandomDist<T>) -> usize {
        // TODO: cache weights somehow, config can be changed middleway though
        dist.weights().sample(&mut self.prng)
    }

    pub fn in_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.prng.gen_range(range)
    }

    pub fn gen_u64(&mut self) -> u64 {
        self.prng.next_u64()
    }

    pub fn gen_bool(&mut self, probability: f32) -> bool {
        self.prng.gen_bool(probability.clamp(0.0, 1.0).into())
    }

    pub fn gen_normal(&mut self) -> f32 {
        self.prng.next_u32() as f32 / f32::MAX
    }

    pub fn pick<'a, T>(&'a mut self, values: &'a [T]) -> &T {
        &values[self.in_range(0..values.len())]
    }

    /// skip one gen step to ensure that a value is consumed in any case
    pub fn skip(&mut self) {
        self.prng.next_u64();
    }

    /// skip n gen steps to ensure that n values are consumed in any case
    pub fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.skip();
        }
    }
}
