pub mod inflight;

pub trait Load {
    type Metric: PartialOrd;

    fn load(&self) -> Self::Metric;
}
