use http;

#[derive(Debug)]
pub enum ProxyError {
    Timeout,
    Connection,
    Http(http::Error),
}
