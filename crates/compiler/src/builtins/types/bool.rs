use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

// `and` and `or` desugar to `if` expressions instead of functions, so we don't implement them.
#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type bool;",
        codegen: Some(Codegen {
            rust: "bool",
            java: "boolean",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[bool]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Not[bool]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = bool;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def not(a:bool): bool;",
                codegen: Some(Codegen {
                    rust: "(|a| !a)",
                    java: "(a) -> !a",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_bool();
                    (!v0).into()
                },
            },
        ],
    });
}
