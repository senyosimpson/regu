use std::sync::Arc;
use std::task::{Context, Poll};

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use tower::{Layer, Service};

use crate::regu::Store;
use crate::request::Request;

#[derive(Clone)]
pub struct Balance<S> {
    inner: S,
    store: Arc<Store>,
}

pub struct BalanceLayer {
    store: Arc<Store>,
}

impl<S> Service<Request> for Balance<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let target = self.store.apps.get(&request.peer.ip()).unwrap();
        let mut rng = SmallRng::from_entropy();
        let origin = target.origins.choose(&mut rng).unwrap();

        request.state.insert(origin.clone());
        self.inner.call(request)
    }
}

impl BalanceLayer {
    pub fn new(store: Arc<Store>) -> BalanceLayer {
        BalanceLayer { store }
    }
}

impl<S> Layer<S> for BalanceLayer {
    type Service = Balance<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Balance {
            inner,
            store: self.store.clone(),
        }
    }
}
