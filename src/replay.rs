use std::task::{Context, Poll};

use tower::{Service, ServiceBuilder};

use crate::{balance::BalanceLayer, request::Request, tcp::TcpService};

/// Replays a request to a different app or machine
pub struct Replay<S> {
    inner: S,
}

pub struct ReplayHeader {
    region: Option<String>,
    instance: Option<String>,
    app: Option<String>,
}

impl<S> Service<Request> for Replay<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let fut = async move {
            // We don't know what its going to return?
            let resp = self.inner.call(request).await.unwrap();

            match resp.headers().get("fly-replay") {
                None => return,
                Some(replay) => {
                    // We have a replay value, we need to replay this to another
                    // machine. To do that the tower way, we need to create another
                    // service and send this request through that service.
                    // Exclude the replay layer here, we don't need it

                    let replay = ReplayHeader {
                        instance: Some(String::from("e2866e30b3e686")),
                        region: None,
                        app: None,
                    };

                    let request = request.next(replay);
                    let service = ServiceBuilder::new()
                        .layer(BalanceLayer::new(store))
                        // .layer(machine)
                        .service(TcpService);

                    service.call(request);
                }
            }
        };

        Box::pin(fut)
    }
}
