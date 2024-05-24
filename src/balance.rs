use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tower::{Layer, Service};

use crate::request::{self, Request};

pub struct Balance<S> {
    inner: S,
}

pub struct BalanceLayer;

impl<S> Service<TcpStream> for Balance<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, client: TcpStream) -> Self::Future {
        let addr = "127.0.0.2:4096".parse().unwrap();
        let request = Request {
            client,
            context: Arc::new(Mutex::new(request::Context { origin: addr })),
        };

        self.inner.call(request)
    }
}

impl<S> Layer<S> for BalanceLayer {
    type Service = Balance<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Balance { inner }
    }
}
