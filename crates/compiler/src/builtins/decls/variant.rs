use crate::Compiler;

use std::rc::Rc;

use crate::ast::Name;
use crate::builtins::Value;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Variant {
    pub name: Name,
    pub value: Rc<Value>,
}

impl Variant {
    pub fn new(name: Name, value: Value) -> Variant {
        Variant {
            name,
            value: Rc::new(value),
        }
    }
}

impl Compiler {
    pub(super) fn declare_variant(&mut self) {}
}
