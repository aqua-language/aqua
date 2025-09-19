use linkme::distributed_slice;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: indoc::indoc! {
            "trait Fn[E,I] {
                 type Output;
                 def call(env:E, args:I): Fn[E,I]::Output;
             }"
        },
    });
}
