use linkme::distributed_slice;
use runtime::builtins::encoding::Encoding;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Encoding;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Encoding",
        decls: &[ImplDecl::Def {
            aqua: "def csv(sep: char): Encoding;",
            codegen: None,
            eval: |_ctx, v| {
                let v0 = v[0].as_char();
                Encoding::csv(v0).into()
            },
        }],
    });

    // ctx.declare_def(
    //     "def json(): Encoding;",
    //     BuiltinDef {
    //         rust: "Encoding::json",
    //         fun: |_ctx, _v| Encoding::Json.into(),
    //     },
    // );
}
