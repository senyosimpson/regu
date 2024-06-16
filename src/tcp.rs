use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower::Service;

use crate::error::ProxyError;
use crate::regu::Origin;
use crate::request::Request;

pub struct TcpService;

impl Service<Request> for TcpService {
    // TODO: In future, use a hyper Bytes? or use another generic that
    // implements Body
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let fut = async move {
            let origin = request.state.remove::<Origin>().unwrap();
            let origin = TcpStream::connect(origin.addr).await.unwrap();

            // We've got a request, we've connected to the upstream. Now we need to perform
            // the proxying logic.
            // Read from client, write to origin
            // Read from origin, write to client
            let client = match request.client {
                Some(client) => client,
                None => return Err(ProxyError::MissingRequest), // TODO: Fix
            };
            let (mut client_read, mut client_write) = client.into_split();
            let (mut origin_read, mut origin_write) = origin.into_split();

            let fut1 = tokio::spawn(async move {
                loop {
                    let mut buf = [0; 1024];
                    let n = match client_read.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => return,
                    };
                    origin_write.write(&buf[0..n]).await;
                }
            });
            let fut2 = tokio::spawn(async move {
                let mut buf = [0; 1024];
                let n = match origin_read.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => return,
                };
                client_write.write(&buf[0..n]).await;
            });

            tokio::join!(fut1, fut2);

            Ok(())
        };

        Box::pin(fut)
    }
}
