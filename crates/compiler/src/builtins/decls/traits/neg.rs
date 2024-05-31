use crate::Compiler;

impl Compiler {
    pub(super) fn declare_neg(&mut self) {
        self.declare_trait(
            "trait Neg[T] {
                 type Output;
                 def neg(v:T): Neg[T]::Output;
             }",
        );
    }
}
