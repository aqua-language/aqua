use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait PartialOrd[T] where PartialEq[T] {
             def partial_cmp(a:T, b:T): Option[Ordering];
             def lt(a:T, b:T): bool;
             def le(a:T, b:T): bool;
             def gt(a:T, b:T): bool;
             def ge(a:T, b:T): bool;
         }",
    });
}
