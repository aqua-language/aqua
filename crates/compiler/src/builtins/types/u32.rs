use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_u32(&mut self) {
        self.declare_type("type u32;", BuiltinType { rust: "u32" });
    }
}
