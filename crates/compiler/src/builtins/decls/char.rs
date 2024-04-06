use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_char(&mut self) {
        self.declare_type("type char;", BuiltinType { rust: "Aggregator" });
    }
}
