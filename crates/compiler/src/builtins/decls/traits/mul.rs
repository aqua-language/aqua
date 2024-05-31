use crate::Compiler;

impl Compiler {
    pub(super) fn declare_mul(&mut self) {
        self.declare_trait(
            "trait Mul[A,B] {
                 type Output;
                 def mul(a:A, b:B): Mul[A,B]::Output;
             }",
        );
    }
}
