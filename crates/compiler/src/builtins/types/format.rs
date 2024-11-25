use linkme::distributed_slice;
use runtime::builtins::format::Format;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Format;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Format",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def csv(sep: char): Format;",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_char();
                    Format::csv(v0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def json(): Format;",
                codegen: None,
                eval: |_ctx, _v| Format::json().into(),
            },
        ],
    });
}
