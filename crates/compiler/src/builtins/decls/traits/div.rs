use crate::Compiler;

impl Compiler {
    pub(super) fn declare_div(&mut self) {
        self.declare_trait(
            "trait Div[A,B] {
                 type Output;
                 def div(a:A, b:B): Div[A,B]::Output;
             }",
        );
    }
}
