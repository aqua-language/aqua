pub mod rust;

use crate::builtins::types::dataflow::Dataflow;
use crate::declare;

pub struct Codegen<'a> {
    decls: &'a declare::Context,
    dataflow: &'a Dataflow,
}

impl<'a> Codegen<'a> {
    pub fn new(decls: &'a declare::Context, dataflow: &'a Dataflow) -> Self {
        Self { decls, dataflow }
    }
}
