use crate::Compiler;

impl Compiler {
    pub(super) fn declare_debug(&mut self) {
        self.declare_trait(
            "trait Debug[T] {
                 def to_string(v:T): String;
             }",
        );
    }
}
