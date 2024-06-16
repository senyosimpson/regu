use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tower::{Layer, Service};

use crate::request::Request;

#[derive(Clone)]
pub struct HttpRetry<S> {
    /// Inner service
    inner: S,
    /// Number of retries
    retries: u8,
}

pub struct HttpRetryLayer {
    retries: u8,
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    future: F,
    retries: u8,
}

impl<S, B> Service<Request> for HttpRetry<S>
where
    S: Service<Request, Response = http::Response<B>>,
    S::Future: Future<Output = Result<http::Response<B>, S::Error>>,
    B: hyper::body::Body,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let f = self.inner.call(request);
        ResponseFuture {
            future: f,
            retries: self.retries,
        }
    }
}

impl HttpRetryLayer {
    pub fn new(retries: u8) -> HttpRetryLayer {
        HttpRetryLayer { retries }
    }
}

impl<S> Layer<S> for HttpRetryLayer {
    type Service = HttpRetry<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpRetry {
            inner,
            retries: self.retries,
        }
    }
}

/// `F` is the future, `R` and `E` are the response and error from the wrapped
/// future `F`. The future `F` is bound the trait `Future<Output = Result<R, E>>`
/// since it returns a result of that nature. Our `ResponseFuture` has the same
/// output, since we return the output of the inner future.
impl<F, E, B> Future for ResponseFuture<F>
where
    F: Future<Output = Result<hyper::Response<B>, E>>,
    B: hyper::body::Body,
{
    type Output = Result<hyper::Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let result = match this.future.poll(cx) {
            Poll::Ready(result) => result,
            Poll::Pending => return Poll::Pending,
        };

        let resp = match result {
            Ok(resp) => resp,
            Err(e) => return Poll::Ready(Err(e)),
        };

        if !resp.status().is_server_error() || *this.retries == 0 {
            return Poll::Ready(Ok(resp));
        }

        // Retry
        *this.retries = *this.retries - 1;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
