use serde::Serialize;

use crate::builtins::Value;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Array(pub Vec<Value>);
