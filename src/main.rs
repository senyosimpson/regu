mod core;
mod error;
mod request;
mod tcp;

mod balance;
mod layers;
mod load;

// mod machines;
// mod replay;

mod regu;
use std::collections::HashMap;

use core::{Origin, Protocol, Store, Target};
use load::inflight::Inflight;
use regu::Regu;

#[tokio::main]
async fn main() {
    let mut apps = HashMap::new();
    let ip = "127.0.0.1".parse().unwrap();
    let addr = "137.66.17.117:80".parse().unwrap();
    let origin = Origin { addr };
    // Wrap the origin so we can keep track of the load
    let origin = Inflight::new(origin);

    let target = Target {
        protocol: Protocol::Http11,
        origins: vec![origin],
    };
    apps.insert(ip, target);

    let store = Store { apps };

    let regu = Regu::new(store);
    regu.run().await;
}
