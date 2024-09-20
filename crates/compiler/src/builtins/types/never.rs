use linkme::distributed_slice;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Def {
        aqua: "def unreachable(): !;",
        codegen: None,
        fun: |_ctx, _v| unreachable!(),
    });

    ctx.declare(Decl::Def {
        aqua: "def panic(msg: String): !;",
        codegen: None,
        fun: |_ctx, v| {
            let v0 = v[0].as_string();
            panic!("{}", v0);
        },
    });

    ctx.declare(Decl::Def {
        aqua: "def exit(): !;",
        codegen: None,
        fun: |_ctx, _v| {
            std::process::exit(0);
        },
    });
}
