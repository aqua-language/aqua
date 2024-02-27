use serde::ser::SerializeMap;
use serde::Serialize;

use crate::ast::Name;
use crate::builtins::Value;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Record(pub Vec<(Name, Value)>);

impl Record {
    pub fn new(fields: Vec<(Name, Value)>) -> Record {
        Record(fields)
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
