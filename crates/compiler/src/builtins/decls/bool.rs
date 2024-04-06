use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_bool(&mut self) {
        self.declare_type("type bool;", BuiltinType { rust: "bool" });
    }
}
