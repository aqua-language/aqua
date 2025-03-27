use smol_str::SmolStr;

use serde::Deserialize;
use serde::Serialize;
use smol_str::ToSmolStr;

use crate::collections::concurrent::ConcurrentMap;
use crate::collections::concurrent::Uid;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(Uid);

impl Symbol {
    pub fn as_str(self) -> &'static str {
        MAP.get(self.0)
    }

    pub fn suffix(self, suffix: impl std::fmt::Display) -> Symbol {
        intern(smol_str::format_smolstr!("{}_{}", self.as_str(), suffix))
    }
}

static MAP: ConcurrentMap<SmolStr, SmolStr> = ConcurrentMap::new();

fn intern(key: SmolStr) -> Symbol {
    Symbol(MAP.insert(key.clone(), key))
}

impl<'a> From<&'a str> for Symbol {
    fn from(data: &'a str) -> Symbol {
        intern(SmolStr::from(data))
    }
}

impl From<SmolStr> for Symbol {
    fn from(data: SmolStr) -> Symbol {
        intern(data)
    }
}

impl From<String> for Symbol {
    fn from(data: String) -> Symbol {
        intern(SmolStr::from(data))
    }
}

impl From<usize> for Symbol {
    fn from(data: usize) -> Symbol {
        intern(data.to_smolstr())
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", MAP.get(self.0))
    }
}

impl Serialize for Symbol {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(MAP.get(self.0))
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(intern(SmolStr::from(String::deserialize(deserializer)?)))
    }
}
