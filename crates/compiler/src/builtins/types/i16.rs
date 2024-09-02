use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_i16(&mut self) {
        self.declare_type("type i16;", BuiltinType { rust: "i16" });
    }
}
