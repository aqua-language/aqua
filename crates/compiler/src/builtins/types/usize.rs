use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type usize;",
        codegen: Some(Codegen {
            rust: "usize",
            java: "Long",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[usize]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Display[usize]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def toString(x: usize): String;",
            codegen: Some(Codegen {
                rust: "|x| x.to_string()",
                java: "(x) -> x.toString()",
                egglog: None,
            }),
            eval: |_ctx, v| {
                let a0 = v[0].as_usize();
                runtime::prelude::String::from(a0.to_string()).into()
            },
        }],
    });
}
