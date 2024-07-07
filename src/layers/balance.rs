use std::sync::Arc;
use std::task::{Context, Poll};

use tower::{Layer, Service};

use crate::balance::Balancer;
use crate::core::Store;
use crate::load::Load;
use crate::request::Request;

#[derive(Clone)]
pub struct Balance<S, B, T> {
    inner: S,
    balancer: B,
    store: Arc<Store<T>>,
}

pub struct BalanceLayer<B, T> {
    store: Arc<Store<T>>,
    balancer: B,
}

impl<S, B, T> Service<Request> for Balance<S, B, T>
where
    S: Service<Request>,
    B: Balancer,
    T: Send + Sync + Load + Clone + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let target = self.store.apps.get(&request.peer.ip()).unwrap();
        let origin = self.balancer.balance(&target.origins());

        request.state.insert(origin.clone());
        self.inner.call(request)
    }
}

impl<B, T> BalanceLayer<B, T> {
    pub fn new(store: Arc<Store<T>>, balancer: B) -> BalanceLayer<B, T> {
        BalanceLayer { store, balancer }
    }
}

impl<S, B: Balancer, T> Layer<S> for BalanceLayer<B, T> {
    type Service = Balance<S, B, T>;

    fn layer(&self, inner: S) -> Self::Service {
        Balance {
            inner,
            store: self.store.clone(),
            balancer: self.balancer.clone(),
        }
    }
}
