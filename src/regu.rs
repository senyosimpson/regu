use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder};

use crate::balance::BalanceLayer;
use crate::http::HttpService;
use crate::tcp::TcpService;

/// The Regu TCP + HTTP proxy
pub struct Regu {
    store: Arc<Store>,
}

pub struct Store {
    pub apps: HashMap<IpAddr, Target>,
}

#[derive(Debug)]
pub struct Target {
    pub protocol: Protocol,
    pub origins: Vec<Origin>,
}

#[derive(Debug)]
pub struct Origin {
    pub addr: SocketAddr,
    pub rtt: Duration,
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
        Regu {
            store: Arc::new(store),
        }
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

            let store = self.store.clone();
            match target.protocol {
                Protocol::Tcp => {
                    tokio::spawn(async move {
                        let mut service = ServiceBuilder::new()
                            .layer(BalanceLayer::new(store))
                            .service(TcpService);

                        service.call((stream, addr)).await;
                        println!("serviced connection");
                    });
                }
                Protocol::Http11 => {
                    tokio::spawn(async move {
                        let mut service = ServiceBuilder::new()
                            .layer(BalanceLayer::new(store))
                            .service(HttpService);

                        service.call((stream, addr)).await;
                        println!("serviced connection");
                    });
                }
                Protocol::Http2 => unreachable!(),
            }
        }
    }
}
