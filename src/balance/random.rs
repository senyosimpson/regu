use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::balance::Balancer;

/// Choose an origin at random based on a uniform distribution
#[derive(Clone)]
pub struct RandomBalancer {
    rng: SmallRng,
}

impl RandomBalancer {
    pub fn new() -> RandomBalancer {
        RandomBalancer {
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Balancer for RandomBalancer {
    fn balance<'a, T>(&mut self, candidates: &'a [T]) -> &'a T {
        candidates.choose(&mut self.rng).unwrap()
    }
}
