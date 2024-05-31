use crate::Compiler;

impl Compiler {
    pub(super) fn declare_eq(&mut self) {
        self.declare_trait(
            "trait Eq[T] {
                 def eq(a:T, b:T): bool;
                 def ne(a:T, b:T): bool;
             }",
        );
    }
}
