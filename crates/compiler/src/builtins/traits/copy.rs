use crate::Compiler;

impl Compiler {
    pub(super) fn declare_copy(&mut self) {
        self.declare_trait("trait Copy[T] {}");
    }
}
