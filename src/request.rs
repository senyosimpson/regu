use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;

use crate::regu::Origin;

/// A request to a service. It is parametrized by a context C which maps
/// to each layer and the final service. The request changes from context
/// to context as it passes through the middleware and finally through the
/// service.
pub struct Request<C> {
    pub client: TcpStream,
    pub peer: SocketAddr,
    pub context: C,
}

// ===== Contexts =====

pub struct EntryContext;

pub struct BalanceContext;

pub struct TcpContext {
    /// The chosen origin server we're proxying to
    pub origin: Origin,
}

impl Request<EntryContext> {
    pub fn new(client: TcpStream, peer: SocketAddr) -> Request<BalanceContext> {
        Request {
            client,
            peer,
            context: BalanceContext,
        }
    }
}

impl Request<BalanceContext> {
    pub fn next(self, origin: SocketAddr) -> Request<TcpContext> {
        let origin = Origin {
            addr: origin,
            rtt: Duration::from_millis(1),
        };
        let ctx = TcpContext { origin };

        Request {
            client: self.client,
            peer: self.peer,
            context: ctx,
        }
    }
}

// unsafe impl Send for Request {}
// unsafe impl Sync for Request {}
