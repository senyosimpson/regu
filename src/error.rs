use std::error::Error;
use std::fmt;

use http;

#[derive(Debug)]
pub enum ProxyError {
    Timeout,
    Connection,
    RetryExhausted,
    MissingRequest,
    Http(http::Error),
    Hyper(hyper::Error),
}

impl Error for ProxyError {}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::Timeout => write!(f, "timed out"),
            ProxyError::Connection => write!(f, "connection error"),
            ProxyError::Http(_) => write!(f, "http error"),
            ProxyError::RetryExhausted => write!(f, "exhausted retries"),
            ProxyError::MissingRequest => write!(f, "hyper request missing"),
            ProxyError::Hyper(_) => write!(f, "hyper error"),
        }
    }
}
