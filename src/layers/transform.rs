use std::net::SocketAddr;
use std::task::{Context, Poll};

use http::Extensions;
use hyper::body::Incoming;
use tower::{Layer, Service};

use crate::request::Request;

#[derive(Clone)]
pub struct HyperToReguRequest<S> {
    inner: S,
    peer: SocketAddr,
}

pub struct HyperToReguRequestLayer {
    peer: SocketAddr,
}

impl<S> Service<hyper::Request<Incoming>> for HyperToReguRequest<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: hyper::Request<Incoming>) -> Self::Future {
        let request = Request {
            peer: self.peer,
            client: None,
            state: Extensions::new(),
            hyper_request: Some(request),
        };
        self.inner.call(request)
    }
}

impl HyperToReguRequestLayer {
    pub fn new(peer: SocketAddr) -> HyperToReguRequestLayer {
        HyperToReguRequestLayer { peer }
    }
}

impl<S> Layer<S> for HyperToReguRequestLayer {
    type Service = HyperToReguRequest<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HyperToReguRequest {
            inner,
            peer: self.peer,
        }
    }
}
