use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_u128(&mut self) {
        self.declare_type("type u128;", BuiltinType { rust: "u128" });
    }
}
