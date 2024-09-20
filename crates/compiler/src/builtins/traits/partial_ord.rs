use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait PartialOrd[L,R] where PartialEq[L,R] {
             def partial_cmp(a:L, b:R): Option[Ordering];
             def lt(a:L, b:R): bool;
             def le(a:L, b:R): bool;
             def gt(a:L, b:R): bool;
             def ge(a:L, b:R): bool;
         }",
    });
}
