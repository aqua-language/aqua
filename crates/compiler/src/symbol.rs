mod standard;
use smol_str::SmolStr;

use serde::Deserialize;
use serde::Serialize;

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
    fn from(name: &'a str) -> Symbol {
        intern(SmolStr::from(name))
    }
}

impl From<SmolStr> for Symbol {
    fn from(name: SmolStr) -> Symbol {
        intern(name)
    }
}

impl From<String> for Symbol {
    fn from(name: String) -> Symbol {
        intern(SmolStr::from(name))
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
