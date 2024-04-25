use crate::Compiler;

use std::rc::Rc;

use crate::ast::Name;
use crate::builtins::Value;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Variant {
    pub x: Name,
    pub v: Rc<Value>,
}

impl Variant {
    pub fn new(x: Name, v: Value) -> Variant {
        Variant {
            x,
            v: Rc::new(v),
        }
    }
}

impl Compiler {
    pub(super) fn declare_variant(&mut self) {}
}
