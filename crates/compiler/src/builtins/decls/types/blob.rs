use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_blob(&mut self) {
        self.declare_type("type Blob;", BuiltinType { rust: "Blob" });
    }
}
