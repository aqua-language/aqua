use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_u16(&mut self) {
        self.declare_type("type u16;", BuiltinType { rust: "u16" });
    }
}
