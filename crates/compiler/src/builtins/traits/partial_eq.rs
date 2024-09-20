use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait PartialEq[L,R] {
             def eq(a:L, b:R): bool;
             def ne(a:L, b:R): bool;
         }",
    });
}
