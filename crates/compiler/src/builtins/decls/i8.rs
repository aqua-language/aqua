use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_i8(&mut self) {
        self.declare_type("type i8;", BuiltinType { rust: "i8" });
    }
}
