use crate::builtins::Context;

use crate::builtins::DECLS;
use linkme::distributed_slice;
use runtime::prelude::DeepClone;
use serde::Serialize;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Unit;

impl DeepClone for Unit {
    fn deep_clone(&self) -> Self {
        Unit
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "()")
    }
}
