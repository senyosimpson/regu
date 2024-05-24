use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder};

use crate::balance::BalanceLayer;
use crate::tcp::TcpService;

/// The Regu TCP + HTTP proxy
pub struct Regu {
    store: Store,
}

pub struct Store {
    apps: HashMap<IpAddr, Target>,
}

#[derive(Debug)]
pub struct Target {
    protocol: Protocol,
    origins: Vec<Origin>,
}

#[derive(Debug)]
pub struct Origin {
    addr: SocketAddr,
    rtt: Duration,
}

#[derive(Debug)]
pub enum Protocol {
    Tcp,
    Http11,
    Http2,
}

impl Regu {
    pub fn new() -> Regu {
        let mut apps = HashMap::new();
        let ip = "127.0.0.1".parse().unwrap();
        let addr = "137.66.17.117:80".parse().unwrap();
        let origin = Origin {
            addr,
            rtt: Duration::from_millis(2),
        };
        let target = Target {
            protocol: Protocol::Tcp,
            origins: vec![origin],
        };
        apps.insert(ip, target);

        let addr = "127.0.0.2:4096".parse().unwrap();
        let origin = Origin {
            addr,
            rtt: Duration::from_millis(1),
        };
        let target = Target {
            protocol: Protocol::Tcp,
            origins: vec![origin],
        };
        apps.insert(ip, target);

        let store = Store { apps };
        Regu { store }
    }

    pub async fn run(&self) {
        println!("lighting up Regu");
        let listener = TcpListener::bind("0.0.0.0:8192").await.unwrap();
        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            println!("received connection from peer {}", addr);

            let target = match self.store.apps.get(&addr.ip()) {
                None => continue,
                Some(target) => {
                    println!("found target: {:#?}", target);
                    target
                }
            };

            match target.protocol {
                Protocol::Tcp => {
                    tokio::spawn(async move {
                        let mut service = ServiceBuilder::new()
                            .layer(BalanceLayer)
                            .service(TcpService);

                        service.call(stream).await;
                    });
                }
                Protocol::Http11 => unreachable!(),
                Protocol::Http2 => unreachable!(),
            }
        }
    }
}
