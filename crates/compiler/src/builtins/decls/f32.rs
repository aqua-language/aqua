use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_f32(&mut self) {
        self.declare_type("type f32;", BuiltinType { rust: "f32" });
    }
}
