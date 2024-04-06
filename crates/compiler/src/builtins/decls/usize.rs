use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_usize(&mut self) {
        self.declare_type("type usize;", BuiltinType { rust: "usize" });
    }
}
