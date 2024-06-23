use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder};

use crate::balance::RandomBalancer;
use crate::layers::{BalanceLayer, HttpRetryLayer, HttpService, HyperToReguRequestLayer};
use crate::request::Request;
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
    /// The protocol the target speaks. One of TCP, HTTP/1.1 or HTTP/2
    pub protocol: Protocol,
    /// A collection of origins that services this target
    pub origins: Vec<Origin>,
}

#[derive(Debug, Clone, Copy)]
pub struct Origin {
    /// The address of the origin server
    pub addr: SocketAddr,
    /// The last seen round-trip time to the origin server. Used as a deciding factor
    /// during load balancing. Defaults to 1 second.
    pub rtt: Duration,
    /// Current count of how many inflight requests are going to this origin
    pub inflight: u32,
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
            rtt: Duration::from_secs(1),
            inflight: 0,
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
                            // .layer(ReplayLayer::new(store))
                            .layer(BalanceLayer::new(store, RandomBalancer::new()))
                            .service(TcpService);

                        let request = Request::new(addr, Some(stream));
                        service.call(request).await;
                        println!("serviced connection");
                    });
                }
                Protocol::Http11 => {
                    tokio::spawn(async move {
                        let service = ServiceBuilder::new()
                            .layer(HyperToReguRequestLayer::new(addr))
                            .layer(BalanceLayer::new(store, RandomBalancer::new()))
                            .layer(HttpRetryLayer::new(5))
                            .service(HttpService);

                        let service = TowerToHyperService::new(service);
                        let io = TokioIo::new(stream);
                        let http = http1::Builder::new();
                        let conn = http.serve_connection(io, service);
                        if let Err(err) = conn.await {
                            eprintln!("server error: {}", err);
                        }
                        println!("serviced connection");
                    });
                }
                Protocol::Http2 => unreachable!(),
            }
        }
    }
}
