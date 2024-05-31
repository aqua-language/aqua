use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_i16(&mut self) {
        self.declare_type("type i16;", BuiltinType { rust: "i16" });
    }
}
