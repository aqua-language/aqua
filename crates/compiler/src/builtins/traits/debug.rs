use linkme::distributed_slice;

use crate::aqua;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn decl(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: aqua! {
            "trait Debug[T] {
                 def debug(v: T): String;
             }"
        },
    });
}
