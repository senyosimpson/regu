use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http_body_util::combinators::BoxBody;
// use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Bytes, Incoming};
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

use tokio::net::TcpStream;
use tower::Service;

use crate::{error::ProxyError, request::Request};

pub struct HttpService;
pub struct HttpProxyService;

impl Service<Request> for HttpService {
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let fut = async move {
            let client = TokioIo::new(req.client);
            tokio::task::spawn(async move {
                let service = HttpProxyService;
                let service = TowerToHyperService::new(service);

                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(client, service)
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });

            Ok(())
        };

        Box::pin(fut)
    }
}

impl Service<hyper::Request<Incoming>> for HttpProxyService {
    type Response = Result<http::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<Incoming>) -> Self::Future {
        let fut = async move {
            let ctx = req.context.lock().await;
            let origin_addr = ctx.origin;
            drop(ctx);

            let stream = TcpStream::connect(origin_addr).await.unwrap();
            let origin = TokioIo::new(stream);

            let (mut sender, conn) = hyper::client::conn::http1::handshake(origin).await.unwrap();

            // Polls the connection
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });

            let mut res = sender.send_request(req).await.unwrap();
        };

        Box::pin(fut)
    }
}
