use crate::Compiler;

use std::rc::Rc;

use crate::ast::Name;
use crate::builtins::value::Value;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Variant {
    pub x: Name,
    pub v: Rc<Value>,
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}({})", self.x, self.v)
    }
}

impl Variant {
    pub fn new(x: Name, v: Value) -> Variant {
        Variant { x, v: Rc::new(v) }
    }
}

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_variant(&mut self) {}
}
