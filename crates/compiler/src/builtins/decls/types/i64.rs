use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    pub(super) fn declare_i64(&mut self) {
        self.declare_type("type i64;", BuiltinType { rust: "i64" });
    }
}
