use std::sync::Arc;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder, ServiceExt};

use crate::balance::RandomBalancer;
use crate::core::{Protocol, Store};
use crate::layers::{BalanceLayer, HttpRetryLayer, HttpService, HyperToReguRequestLayer};
use crate::load::Load;
use crate::request::Request;
use crate::tcp::TcpService;

/// The Regu TCP + HTTP proxy
pub struct Regu<T> {
    store: Arc<Store<T>>,
}

impl<T> Regu<T>
where
    T: Load + Clone + Sync + Send + 'static,
{
    pub fn new(store: Store<T>) -> Regu<T> {
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
                    // println!("found target: {:#?}", target);
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
                        let _ = service.ready().await.unwrap().call(request).await;
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
                    });
                }
                Protocol::Http2 => unreachable!(),
            }
        }
    }
}
