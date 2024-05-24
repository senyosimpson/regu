mod proxy;
use proxy::ProxyHttp;

use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder};

#[tokio::main]
async fn main() {
    let proxy = ProxyHttp {
        backends: vec!["oom.senyosimpson.com".into()],
    };

    let listener = TcpListener::bind("0.0.0.0:8192").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let service = ServiceBuilder::new().service(proxy);
            service.call(stream);
        });
    }
}
