use std::path::PathBuf;
use std::task::{Context, Poll};

use rquickjs::{Context as JsContext, FromJs, Runtime};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tower::Service;

use crate::request::Request;

/// JavaScript scripting capabalities for the proxy
pub struct Js<S> {
    inner: S,
    request_hook_path: PathBuf,
    response_hook_path: PathBuf,
}

impl<S> Service<Request> for Js<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    // type Future = S::Future;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // For now, our JavaScript engine has two hook points: Pre-request and Post-Request
        // This means we can define two functions, modifyRequest & modifyResponse.
        // modifyRequest takes a http::Request object and returns a modified version of it.
        // we can then send it on its way. modifyResponse takes a http::Response object and
        // returns a modified version of it.
        let request_hook_path = self.request_hook_path;
        let response_hook_path = self.response_hook_path;
        async move {
            // let mut file = File::open(request_hook_path).await.unwrap();
            // let mut req_hook = vec![];
            // file.read_to_end(&mut req_hook).await.unwrap();

            // let mut file = File::open(response_hook_path).await.unwrap();
            // let mut resp_hook = vec![];
            // file.read_to_end(&mut resp_hook).await.unwrap();

            let rt = Runtime::new().unwrap();
            let ctx = JsContext::full(&rt).unwrap();
            ctx.with(
                |ctx| match ctx.eval_file::<u32, PathBuf>(request_hook_path) {
                    Ok(v) => println!("got value {v}"),
                    Err(e) => println!("failed javascript evaluation {e}"),
                },
            )
        };

        async move {}
    }
}
