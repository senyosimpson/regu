use std::fmt;
use std::task::{Context, Poll};

use tower::{Service, ServiceBuilder};

use crate::proxy::{ProxyError, Request};

/// Layer that automatically starts a machine if:
///   1. It's valid to start
///   2. We've received a request to the machine and its stopped

/// Replays a request to a different app or machine
pub struct Replay<S> {
    inner: S,
}

impl<S> Service<http::Request<Vec<u8>>> for Replay<S>
where
    S: Service<http::Request<Vec<u8>>, Response = http::Response<Vec<u8>>>,
    S::Error: fmt::Debug,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<Vec<u8>>) -> Self::Future {
        async move {
            let resp = self.inner.call(req).await.unwrap();
            match resp.headers().get("fly-replay") {
                None => return,
                Some(replay) => {
                    // We have a replay value, we need to replay this to another
                    // machine. To do that the tower way, we need to create another
                    // service and send this request through that service.
                    // let balance = Balance::new();
                    let proxy = ProxyHttp {
                        backends: vec!["oom.senyosimpson.com".into()],
                    };
                    // Exclude the replay layer here, we don't need it
                    let service = ServiceBuilder::new()
                        // .layer(balance)
                        // .layer(machine)
                        .service(proxy);

                    // service.call(stream);
                }
            }
        }
    }
}
