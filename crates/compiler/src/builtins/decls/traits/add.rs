use crate::Compiler;

impl Compiler {
    pub(super) fn declare_add(&mut self) {
        self.declare_trait(
            "trait Add[A,B] {
                 type Output;
                 def add(a:A, b:B): Add[A,B]::Output;
             }",
        );
    }
}
