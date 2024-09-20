use crate::builtins::value::Tuple;
use crate::builtins::Context;
use crate::builtins::Decl;

use linkme::distributed_slice;

use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Def {
        aqua: "def print[T](s: T): () where Display[T];",
        codegen: None,
        fun: |_ctx, v| {
            let v0 = v[0].to_string();
            eprintln!("{}", v0);
            Tuple(vec![]).into()
        },
    });

    ctx.declare(Decl::Def {
        aqua: "def debug[T](v: T): () where Debug[T];",
        codegen: None,
        fun: |_ctx, _v| {
            todo!()
            // let v0 = &v[0];
            // eprintln!("{:?}", v0);
            // ().into()
        },
    });

    ctx.declare(Decl::Def {
        aqua: "def bifs(): ();",
        codegen: None,
        fun: |_ctx, _v| {
            todo!()
            // compiler_codegen_ast::write(&mut compiler_codegen::Context::stderr().colors(true), &crate::prelude::prelude());
            // ().into()
        },
    });
}
