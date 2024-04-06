use crate::ast::Index;
use crate::Compiler;

use serde::Serialize;

use crate::builtins::Value;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Tuple(pub Vec<Value>);

impl Tuple {
    pub fn new(values: Vec<Value>) -> Tuple {
        Tuple(values)
    }
}

impl<'a> std::ops::Index<&'a Index> for Tuple {
    type Output = Value;

    fn index(&self, index: &'a Index) -> &Value {
        &self.0[index.data]
    }
}

impl Compiler {
    pub(super) fn declare_tuple(&mut self) {}
}
