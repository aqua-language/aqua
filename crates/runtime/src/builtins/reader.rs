use serde::Deserialize;
use serde::Serialize;

use crate::builtins::path::Path;
use crate::builtins::socket::SocketAddr;
use crate::traits::DeepClone;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub enum Reader {
    Stdin,
    File { path: Path, watch: bool },
    Http { addr: SocketAddr },
    Tcp { addr: SocketAddr },
    Kafka { addr: SocketAddr, topic: crate::prelude::String },
}

impl DeepClone for Reader {
    fn deep_clone(&self) -> Self {
        self.clone()
    }
}

impl std::fmt::Display for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reader::Stdin => write!(f, "Stdin"),
            Reader::File { path, watch } => write!(f, "File(path={path}, watch={watch})"),
            Reader::Http { addr } => write!(f, "Http(addr={})", addr),
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
        Self::File { path, watch }
    }
    pub fn http(addr: SocketAddr) -> Self {
        Self::Http { addr }
    }
    pub fn tcp(addr: SocketAddr) -> Self {
        Self::Tcp { addr }
    }
    pub fn kafka(addr: SocketAddr, topic: crate::prelude::String) -> Self {
        Self::Kafka { addr, topic }
    }
}
