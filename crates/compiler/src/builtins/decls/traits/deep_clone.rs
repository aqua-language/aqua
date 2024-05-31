use crate::Compiler;

impl Compiler {
    pub(super) fn declare_deep_clone(&mut self) {
        self.declare_trait(
            "trait DeepClone[T] {
                 def deep_clone(v:T): T;
             }",
        );
    }
}
