use crate::builtins::Context;

use std::rc::Rc;

use crate::ast::Name;
use crate::builtins::value::Value;
use linkme::distributed_slice;
use runtime::traits::DeepClone;
use serde::Serialize;

use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Variant {
    pub x: Name,
    pub v: Rc<Value>,
}

impl DeepClone for Variant {
    fn deep_clone(&self) -> Self {
        Variant {
            x: self.x.clone(),
            v: self.v.deep_clone(),
        }
    }
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
