use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tower::Service;

use crate::error::ProxyError;
use crate::regu::Origin;
use crate::request::Request;

#[derive(Clone)]
pub struct HttpService;

impl Service<Request> for HttpService {
    type Response = hyper::Response<Incoming>;
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let origin = request.state.remove::<Origin>().unwrap();
        let fut = async move {
            let stream = TcpStream::connect(origin.addr).await.unwrap();
            let origin = TokioIo::new(stream);

            let (mut sender, conn) = hyper::client::conn::http1::handshake(origin).await.unwrap();

            // Polls the connection
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });

            match request.hyper_request {
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

        Box::pin(fut)
    }
}
