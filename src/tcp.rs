use std::{
    error::Error,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower::Service;

use crate::{
    error::ProxyError,
    request::{Request, TcpContext},
};

pub struct TcpService;

impl Service<Request<TcpContext>> for TcpService {
    // TODO: In future, use a hyper Bytes? or use another generic that
    // implements Body
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // If we know we're handling HTTP connections, maybe we can use hyper::serve_conn?
    // That way, we can get HTTP information we actually care about and then make some
    // moves from there
    fn call(&mut self, req: Request<TcpContext>) -> Self::Future {
        let fut = async move {
            // let origin_addr = {
            //     let ctx = req.context.lock().await;
            //     ctx.origin
            // };
            let origin = TcpStream::connect(req.state.origin.addr).await.unwrap();

            // We've got a request, we've connected to the upstream. Now we need to perform
            // the proxying logic.
            // Read from client, write to origin
            // Read from origin, write to client
            let (mut client_read, mut client_write) = req.client.into_split();
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

impl Error for ProxyError {}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::Timeout => write!(f, "timed out"),
            ProxyError::Connection => write!(f, "connection error"),
            ProxyError::Http(_) => write!(f, "http error"),
        }
    }
}
