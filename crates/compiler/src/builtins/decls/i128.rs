use crate::Compiler;
use crate::ast::BuiltinType;

impl Compiler {
    pub(in super) fn declare_i128(&mut self) {
        self.declare_type("type i128;", BuiltinType { rust: "i128" });
    }
}
