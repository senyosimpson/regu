use std::net::SocketAddr;

use http::Extensions;
use hyper::body::Incoming;
use tokio::net::TcpStream;

/// A request to a service. It contains an Extension type which can be used
/// to store underlying state throughout the lifetime of a request.
pub struct Request {
    pub peer: SocketAddr,
    pub client: Option<TcpStream>,
    pub hyper_request: Option<hyper::Request<Incoming>>,
    pub state: Extensions,
}

impl Request {
    pub fn new(peer: SocketAddr, client: Option<TcpStream>) -> Request {
        Request {
            client,
            peer,
            hyper_request: None,
            state: Extensions::new(),
        }
    }
}
