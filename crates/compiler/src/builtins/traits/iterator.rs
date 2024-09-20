use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait Iterator[T] {
             type Item;
             def next(data: T): Option[Iterator[T]::Item];
         }",
    });
}
