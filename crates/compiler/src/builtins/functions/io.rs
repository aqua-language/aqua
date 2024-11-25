use crate::builtins::value::Tuple;
use crate::builtins::Context;
use crate::builtins::Decl;

use linkme::distributed_slice;

use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Def {
        docs: "",
        aqua: "def print[T](v: T): () where Display[T];",
        codegen: None,
        fun: |_ctx, v| {
            let v0 = &v[0];
            eprintln!("{}", v0);
            Tuple(vec![]).into()
        },
    });

    ctx.declare(Decl::Def {
        docs: "",
        aqua: "def debug[T](v: T): () where Debug[T];",
        codegen: None,
        fun: |_ctx, v| {
            let v0 = &v[0];
            eprintln!("{:?}", v0);
            Tuple(vec![]).into()
        },
    });
}
