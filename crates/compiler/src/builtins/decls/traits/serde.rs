use crate::Compiler;

impl Compiler {
    pub(super) fn declare_serde(&mut self) {
        self.declare_trait("trait Serde[T] { }");
    }
}
