use crate::Compiler;

impl Compiler {
    pub(super) fn declare_partial_eq(&mut self) {
        self.declare_trait(
            "trait PartialEq[L,R] {
                 def eq(a:L, b:R): bool;
                 def ne(a:L, b:R): bool;
             }",
        );
    }
}
