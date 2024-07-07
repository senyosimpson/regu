use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use pin_project::pin_project;
use tokio::net::TcpStream;
use tower::Service;

use crate::error::ProxyError;
use crate::request::Request;

/// Replays a request to a different app or machine. Only useful on Fly.io
pub struct Replay<S> {
    inner: S,
}

pub struct ReplayLayer {}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    future: F,
    request: Request,
}

pub struct ReplayHeader {
    region: Option<String>,
    instance: Option<String>,
    app: Option<String>,
}

impl<S, F, B> Service<Request> for Replay<S>
where
    S: Service<Request, Response = hyper::Response<B>>,
    F: Future<Output = Result<hyper::Response<B>, S::Error>>,
    B: hyper::body::Body,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<F>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let f = self.inner.call(request);
        ResponseFuture { future: f, request }
    }
}

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

        let header = match resp.headers().get("fly_replay") {
            Some(header) => header,
            None => return Poll::Ready(Ok(resp)),
        };

        // We have a header value. For now, let's just assume
        // its for an instance
        let replay = ReplayHeader {
            region: None,
            instance: Some("e2866e30b3e686".into()),
            app: None,
        };

        // Now we need to create a service and make a request to our given origin
        let fut = async move {
            let stream = TcpStream::connect("127.0.0.1:8191").await.unwrap();
            let origin = TokioIo::new(stream);

            let (mut sender, conn) = http1::handshake(origin).await.unwrap();
            let http = http1::Builder::new();

            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });

            match this.request.hyper_request {
                Some(hreq) => {
                    let res = match sender.send_request(hreq).await {
                        Ok(resp) => Ok(resp),
                        Err(e) => Err(ProxyError::Hyper(e)),
                    };
                    return res;
                }
                None => return Err(ProxyError::MissingRequest),
            }
        };

        Pin::new(&mut fut).poll(cx)
    }
}
