use crate::Compiler;

impl Compiler {
    pub(super) fn declare_partial_ord(&mut self) {
        self.declare_trait(
            "trait PartialOrd[L,R] where PartialEq[L,R] {
                 def partial_cmp(a:L, b:R): Option[Ordering];
                 def lt(a:L, b:R): bool;
                 def le(a:L, b:R): bool;
                 def gt(a:L, b:R): bool;
                 def ge(a:L, b:R): bool;
             }",
        );
    }
}
