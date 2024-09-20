use runtime::prelude::DeepClone;
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::ast::Map;
use crate::ast::Name;
use crate::builtins::value::Value;

use crate::builtins::Context;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Record(pub Map<Name, Value>);

impl DeepClone for Record {
    fn deep_clone(&self) -> Self {
        let map = self
            .0
            .iter()
            .map(|(k, v)| (k.clone(), v.deep_clone()))
            .collect();
        Record(map)
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.0.iter();
        if let Some((k, v)) = iter.next() {
            write!(f, "{}: {}", k, v)?;
            for (k, v) in iter {
                write!(f, ", {}: {}", k, v)?;
            }
        }
        write!(f, "}}")
    }
}

impl Record {
    pub fn new(fields: Map<Name, Value>) -> Record {
        Record(fields)
    }
}

impl std::ops::Index<&Name> for Record {
    type Output = Value;

    fn index(&self, name: &Name) -> &Value {
        self.0
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .unwrap()
    }
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, value) in &self.0 {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}
