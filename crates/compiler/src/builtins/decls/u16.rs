use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    pub(super) fn declare_u16(&mut self) {
        self.declare_type("type u16;", BuiltinType { rust: "u16" });
    }
}
