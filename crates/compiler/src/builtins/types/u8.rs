use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_u8(&mut self) {
        self.declare_type("type u8;", BuiltinType { rust: "u8" });
    }
}
