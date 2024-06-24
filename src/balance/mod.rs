pub mod p2c;
pub use p2c::P2C;

pub mod random;
pub use random::RandomBalancer;

use crate::load::Load;

pub trait Balancer: Clone {
    fn balance<'a, T: Load>(&mut self, candidates: &'a [T]) -> &'a T;
}
