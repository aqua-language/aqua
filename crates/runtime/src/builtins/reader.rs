#![allow(unused)]
use serde::Deserialize;
use serde::Serialize;

use crate::builtins::path::Path;
use crate::builtins::socket::SocketAddr;
use crate::builtins::stream::Stream;
use crate::builtins::string::String;
// use crate::builtins::url::Url;
use crate::traits::Data;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub enum Reader {
    Stdin,
    File { path: Path, watch: bool },
    // Http { url: Url },
    Tcp { addr: SocketAddr },
    Kafka { addr: SocketAddr, topic: String },
}

impl std::fmt::Display for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reader::Stdin => write!(f, "Stdin"),
            Reader::File { path, watch } => write!(f, "File(path={path}, watch={watch})"),
            // Reader::Http { url } => write!(f, "Http(url={})", url),
            Reader::Tcp { addr } => write!(f, "Tcp(addr={addr})"),
            Reader::Kafka { addr, topic } => write!(f, "Kafka(addr={addr}, topic={topic})"),
        }
    }
}

impl Reader {
    pub fn stdin() -> Self {
        Self::Stdin
    }
    pub fn file(path: Path, watch: bool) -> Self {
        if !path.0.exists() {
            tracing::warn!("{} does not exist", path.0.display());
        }
        Self::File { path, watch }
    }
    // pub fn http(url: Url) -> Self {
    //     Self::Http { url }
    // }
    pub fn tcp(addr: SocketAddr) -> Self {
        Self::Tcp { addr }
    }
    pub fn kafka(addr: SocketAddr, topic: String) -> Self {
        Self::Kafka { addr, topic }
    }
}
