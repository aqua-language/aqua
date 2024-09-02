use crate::Compiler;

impl Compiler {
    pub(super) fn declare_default(&mut self) {
        self.declare_trait(
            "trait Default[T] {
                 def default(): T;
             }",
        );
    }
}
