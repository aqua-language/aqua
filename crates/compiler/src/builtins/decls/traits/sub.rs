use crate::Compiler;

impl Compiler {
    pub(super) fn declare_sub(&mut self) {
        self.declare_trait(
            "trait Sub[A,B] {
                 type Output;
                 def sub(a:A, b:B): Sub[A,B]::Output;
             }",
        );
    }
}
