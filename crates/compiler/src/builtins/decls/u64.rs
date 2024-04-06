use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    pub(super) fn declare_u64(&mut self) {
        self.declare_type("type u64;", BuiltinType { rust: "u64" });
    }
}
