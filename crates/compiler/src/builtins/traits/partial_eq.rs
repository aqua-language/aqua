use crate::aqua;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: aqua! {
           "trait PartialEq[T] {
                def eq(a: T, b: T): bool;
                def ne(a: T, b: T): bool;
            }"
        },
    });
}
