use std::ops::Deref;
use std::sync::Arc;

use crate::load::Load;

/// Calculate the load based on the number of inflight requests
#[derive(Clone)]
pub struct Inflight<T> {
    inner: T,
    count: RefCount,
}

/// A reference count. It just uses the number of references to the arc as
/// the count. This is valid for our case since we clone the object every
/// time we handle a new request. As it comes in and out of scope, the
/// count is updated, hence tracking the number of inflight requests.
///
/// The refcount is an _estimation_ in that the values might change at the
/// time of reading the value by other threads. This is fine, we don't need
/// 100% accuracy for load balancing, over the fullness of time we'll still
/// have fairly good load balancing
#[derive(Clone)]
pub struct RefCount(Arc<()>);

// ===== Inflight =====

impl<T> Inflight<T> {
    pub fn new(inner: T) -> Inflight<T> {
        Inflight {
            inner,
            count: RefCount::new(),
        }
    }
}

impl<T> Load for Inflight<T> {
    type Metric = usize;

    fn load(&self) -> Self::Metric {
        Arc::strong_count(&self.count)
    }
}

impl<T> Drop for Inflight<T> {
    fn drop(&mut self) {
        println!("prev count: {}", Arc::strong_count(&self.count));
    }
}

impl<T> Deref for Inflight<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ===== RefCount =====

impl RefCount {
    fn new() -> RefCount {
        RefCount(Arc::new(()))
    }
}

impl Deref for RefCount {
    type Target = Arc<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
