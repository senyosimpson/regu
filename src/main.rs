mod balance;
mod error;
mod http;
mod request;
mod tcp;

mod regu;
use regu::Regu;

#[tokio::main]
async fn main() {
    let regu = Regu::new();
    regu.run().await;
}
