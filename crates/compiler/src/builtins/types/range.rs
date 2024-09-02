use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_range(&mut self) {
        self.declare_type("type Range[T];", BuiltinType { rust: "Range" });
    }
}
