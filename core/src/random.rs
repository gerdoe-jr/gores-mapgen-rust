use crate::position::ShiftDirection;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand_distr::WeightedAliasIndex;
use seahash::hash;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomDistConfig<T> {
    pub values: Option<Vec<T>>,
    pub probs: Vec<f32>,
}

impl<T> RandomDistConfig<T> {
    pub fn new(values: Option<Vec<T>>, probs: Vec<f32>) -> RandomDistConfig<T> {
        RandomDistConfig { values, probs }
    }

    pub fn normalize_probs(&mut self) {
        let probs_sum: f32 = self.probs.iter().sum();

        if probs_sum == 1.0 {
            return; // skip if already normalized
        }

        // if all values are zero, set all to 1/n
        if probs_sum == 0.0 {
            let len = self.probs.len();
            for val in self.probs.iter_mut() {
                *val = 1.0 / len as f32;
            }
        // otherwise normalize, if required
        } else if probs_sum != 1.0 {
            for val in self.probs.iter_mut() {
                *val /= probs_sum; // Normalize the vector
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RandomDist<T> {
    rnd_cfg: RandomDistConfig<T>,
    rnd_dist: WeightedAliasIndex<f32>,
}

pub enum RandomDistType {
    InnerSize,
    OuterMargin,
    Circularity,
    ShiftDirection,
}

impl<T: Clone> RandomDist<T> {
    pub fn new(config: RandomDistConfig<T>) -> RandomDist<T> {
        RandomDist {
            rnd_dist: WeightedAliasIndex::new(config.probs.clone()).unwrap(),
            rnd_cfg: config,
        }
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
    gen: SmallRng,
    shift: RandomDist<ShiftDirection>,
    kernel_size: RandomDist<usize>,
    kernel_margin: RandomDist<usize>,
    circularity: RandomDist<f32>,
}

impl Random {
    pub fn new(
        seed: Seed,
        shift: RandomDist<ShiftDirection>,
        kernel_margin: RandomDist<usize>,
        kernel_size: RandomDist<usize>,
        circularity: RandomDist<f32>,
    ) -> Self {
        Random {
            gen: SmallRng::seed_from_u64(seed),
            shift,
            kernel_margin,
            kernel_size,
            circularity,
        }
    }

    pub fn sample_inner_kernel_size(&mut self) -> usize {
        let dist = &self.kernel_size;
        let index = dist.rnd_dist.sample(&mut self.gen);
        dist.rnd_cfg
            .values
            .as_ref()
            .unwrap()
            .get(index)
            .unwrap()
            .clone()
    }

    pub fn sample_outer_kernel_margin(&mut self) -> usize {
        let dist = &self.kernel_margin;
        let index = dist.rnd_dist.sample(&mut self.gen);
        dist.rnd_cfg
            .values
            .as_ref()
            .unwrap()
            .get(index)
            .unwrap()
            .clone()
    }

    pub fn sample_circularity(&mut self) -> f32 {
        let dist = &self.circularity;
        let index = dist.rnd_dist.sample(&mut self.gen);
        dist.rnd_cfg
            .values
            .as_ref()
            .unwrap()
            .get(index)
            .unwrap()
            .clone()
    }

    pub fn sample_shift(&mut self, ordered_shifts: &[ShiftDirection; 4]) -> ShiftDirection {
        let dist = &self.shift;
        let index = dist.rnd_dist.sample(&mut self.gen);
        ordered_shifts.get(index).unwrap().clone()
    }

    pub fn in_range_inclusive(&mut self, low: usize, high: usize) -> usize {
        assert!(high >= low, "no valid range");
        let n = (high - low) + 1;
        let rnd_value = self.gen.next_u64() as usize;

        low + (rnd_value % n)
    }

    pub fn in_range_exclusive(&mut self, low: usize, high: usize) -> usize {
        assert!(high > low, "no valid range");
        let n = high - low;
        let rnd_value = self.gen.next_u64() as usize;

        low + (rnd_value % n)
    }

    pub fn random_u64(&mut self) -> u64 {
        self.gen.next_u64()
    }

    pub fn with_probability(&mut self, probability: f32) -> bool {
        if probability == 1.0 {
            self.skip();
            true
        } else if probability == 0.0 {
            self.skip();
            false
        } else {
            (self.gen.next_u64() as f32) < (u64::max_value() as f32 * probability)
        }
    }

    /// skip one gen step to ensure that a value is consumed in any case
    pub fn skip(&mut self) {
        self.gen.next_u64();
    }

    /// skip n gen steps to ensure that n values are consumed in any case
    pub fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.gen.next_u64();
        }
    }

    pub fn pick_element<'a, T>(&'a mut self, values: &'a [T]) -> &T {
        &values[self.in_range_exclusive(0, values.len())]
    }

    pub fn random_circularity(&mut self) -> f32 {
        self.gen.next_u64() as f32 / u64::max_value() as f32
    }
}
