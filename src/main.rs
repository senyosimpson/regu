mod error;
// mod replay;
mod request;
mod tcp;

mod layers;
mod machines;

mod regu;
use regu::Regu;

#[tokio::main]
async fn main() {
    let regu = Regu::new();
    regu.run().await;
}
