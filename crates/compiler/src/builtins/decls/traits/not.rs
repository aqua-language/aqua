use crate::Compiler;

impl Compiler {
    pub(super) fn declare_not(&mut self) {
        self.declare_trait(
            "trait Not[T] {
                 type Output;
                 def not(v:T): Not[T]::Output;
             }",
        );
    }
}
