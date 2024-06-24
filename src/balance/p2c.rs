use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::balance::Balancer;
use crate::load::Load;

/// Choose an origin using the p2c algorithm
#[derive(Clone)]
pub struct P2C {
    rng: SmallRng,
}

impl P2C {
    pub fn new() -> P2C {
        P2C {
            rng: SmallRng::from_entropy(),
        }
    }
}

impl Balancer for P2C {
    fn balance<'a, T: Load>(&mut self, candidates: &'a [T]) -> &'a T {
        let mut origins = Vec::with_capacity(2);
        for (b, slot) in candidates
            .choose_multiple(&mut self.rng, 2)
            .zip(origins.iter_mut())
        {
            *slot = b;
        }

        if origins[0].load() > origins[1].load() {
            origins[1]
        } else {
            origins[0]
        }
    }
}
