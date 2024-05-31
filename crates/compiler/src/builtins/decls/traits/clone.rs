use crate::Compiler;

impl Compiler {
    pub(super) fn declare_clone(&mut self) {
        self.declare_trait(
            "trait Clone[T] {
                 def clone(v:T): T;
             }",
        );
    }
}
