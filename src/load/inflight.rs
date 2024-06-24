use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::load::Load;

/// Calculate the load based on the number of inflight requests
// #[derive(Clone)]
pub struct Inflight<T> {
    inner: T,
    count: Arc<AtomicU32>,
}

impl<T> Inflight<T> {
    pub fn new(inner: T) -> Inflight<T> {
        Inflight {
            inner,
            count: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl<T> Load for Inflight<T> {
    type Metric = u32;

    fn load(&self) -> Self::Metric {
        self.count.load(Ordering::SeqCst)
    }
}

impl<T: Clone> Clone for Inflight<T> {
    fn clone(&self) -> Self {
        // increase the refcount
        let count = self.count.clone();
        // increment the count
        count.fetch_add(1, Ordering::SeqCst);

        Inflight {
            inner: self.inner.clone(),
            count,
        }
    }
}

impl<T> Drop for Inflight<T> {
    fn drop(&mut self) {
        let count = self.count.fetch_sub(1, Ordering::SeqCst);
        println!("prev count: {}", count);
    }
}

impl<T> Deref for Inflight<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
