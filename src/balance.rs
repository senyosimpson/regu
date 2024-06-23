use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

use crate::regu::Origin;

pub trait Balancer: Clone {
    fn balance<'a>(&mut self, candidates: &'a [Origin]) -> &'a Origin;
}

/// Choose a target at random based on a uniform distribution
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
    fn balance<'a>(&mut self, candidates: &'a [Origin]) -> &'a Origin {
        candidates.choose(&mut self.rng).unwrap()
    }
}
