use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tower::{Layer, Service};

use crate::regu::Store;
use crate::request::{self, Request};

pub struct Balance<S> {
    inner: S,
    store: Arc<Store>,
}

pub struct BalanceLayer {
    store: Arc<Store>,
}

impl<S> Service<(TcpStream, SocketAddr)> for Balance<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: (TcpStream, SocketAddr)) -> Self::Future {
        let (client, addr) = request;
        let target = self.store.apps.get(&addr.ip()).unwrap();
        let mut rng = SmallRng::from_entropy();
        let origin = target.origins.choose(&mut rng).unwrap();

        let request = Request {
            client,
            context: Arc::new(Mutex::new(request::Context {
                origin: origin.addr,
            })),
        };

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
