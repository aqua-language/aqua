use crate::Compiler;

impl Compiler {
    pub(super) fn declare_display(&mut self) {
        self.declare_trait(
            "trait Display[T] {
                 def to_string(v:T): String;
             }",
        );
    }
}
