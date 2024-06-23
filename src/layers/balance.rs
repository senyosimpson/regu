use std::sync::Arc;
use std::task::{Context, Poll};

use tower::{Layer, Service};

use crate::balance::Balancer;
use crate::regu::Store;
use crate::request::Request;

#[derive(Clone)]
pub struct Balance<S, B> {
    inner: S,
    balancer: B,
    store: Arc<Store>,
}

pub struct BalanceLayer<B> {
    store: Arc<Store>,
    balancer: B,
}

impl<S, B> Service<Request> for Balance<S, B>
where
    S: Service<Request>,
    B: Balancer,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let target = self.store.apps.get(&request.peer.ip()).unwrap();
        let origin = self.balancer.balance(&target.origins);

        request.state.insert(origin.clone());
        self.inner.call(request)
    }
}

impl<B> BalanceLayer<B> {
    pub fn new(store: Arc<Store>, balancer: B) -> BalanceLayer<B> {
        BalanceLayer { store, balancer }
    }
}

impl<S, B: Balancer> Layer<S> for BalanceLayer<B> {
    type Service = Balance<S, B>;

    fn layer(&self, inner: S) -> Self::Service {
        Balance {
            inner,
            store: self.store.clone(),
            balancer: self.balancer.clone(),
        }
    }
}
