use crate::Compiler;

impl Compiler {
    pub(super) fn declare_ord(&mut self) {
        self.declare_trait(
            "trait Ord[T] where PartialOrd[T,T] {
                 def cmp(a:T, b:T): Ordering;
                 def min(a:T, b:T): T;
                 def max(a:T, b:T): T;
             }",
        );
    }
}
