use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use crate::load::Load;

pub struct Store<T> {
    pub apps: HashMap<IpAddr, Target<T>>,
}

#[derive(Debug)]
pub struct Target<T> {
    /// The protocol the target speaks. One of TCP, HTTP/1.1 or HTTP/2
    pub protocol: Protocol,
    /// A collection of origins that services this target
    pub origins: Vec<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct Origin {
    /// The address of the origin server
    pub addr: SocketAddr,
}

#[derive(Debug)]
pub enum Protocol {
    Tcp,
    Http11,
    Http2,
}

impl<T> Target<T>
where
    T: Load + Clone,
{
    pub fn origins(&self) -> &[T] {
        &self.origins
    }
}
