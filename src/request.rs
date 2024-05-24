use std::{net::SocketAddr, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

pub struct Request {
    pub client: TcpStream,
    pub context: Arc<Mutex<Context>>,
}

pub struct Context {
    pub origin: SocketAddr,
}

// unsafe impl Send for Request {}
// unsafe impl Sync for Request {}
