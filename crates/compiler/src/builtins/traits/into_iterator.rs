use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait IntoIterator[T]
         where Iterator[IntoIterator[T]::IntoIter,
                         Item = IntoIterator[T]::Item] {
             type Item;
             type IntoIter;
             def into_iter(data: T): IntoIterator[T]::IntoIter;
         }",
    });
}
