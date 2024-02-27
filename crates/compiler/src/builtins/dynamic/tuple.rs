use serde::Serialize;

use crate::builtins::Value;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Tuple(pub Vec<Value>);

impl Tuple {
    pub fn new(values: Vec<Value>) -> Tuple {
        Tuple(values)
    }
}
