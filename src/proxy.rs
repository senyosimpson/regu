use std::{
    future::Future,
    // io::{Read, Write},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower::Service;

pub struct ProxyHttp {
    // This should obviously contain some set of things that "dns lookupable"
    pub backends: Vec<String>,
}

pub enum ProxyError {
    Timeout,
    Connection,
    Http(http::Error),
}

impl Service<TcpStream> for ProxyHttp {
    // TODO: In future, use a hyper Bytes? or use another generic that
    // implements Body
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // If we know we're handling HTTP connections, maybe we can use hyper::serve_conn?
    // That way, we can get HTTP information we actually care about and then make some
    // moves from there
    fn call(&mut self, client: TcpStream) -> Self::Future {
        let backend = self.backends[0].clone();

        let fut = async move {
            // Okay, so we actually want to make a request to an upstream server, which we'll
            // get from the request

            // Make a tcp connection. For now, we're acting like the backends are all for the
            // same application
            // only connect on 443 for now
            let ip = dns_lookup::lookup_host(&backend).unwrap()[0];
            let addr = SocketAddr::new(ip, 443);

            let origin = TcpStream::connect(addr).await.unwrap();

            // We've got a request, we've connected to the upstream. Now we need to perform
            // the proxying logic.
            // Read from client, write to origin
            // Read from origin, write to client
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
                    origin_write.write(&buf[0..n]);
                }
            });
            let fut2 = tokio::spawn(async move {
                let mut buf = [0; 1024];
                let n = match origin_read.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => return,
                };
                client_write.write(&buf[0..n]);
            });

            tokio::join!(fut1, fut2);

            Ok(())
        };

        Box::pin(fut)
    }
}
