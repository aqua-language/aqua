use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_i8(&mut self) {
        self.declare_type("type i8;", BuiltinType { rust: "i8" });
    }
}
