use runtime::builtins::set::Set;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Set[T];",
        codegen: Some(Codegen {
            rust: "Set",
            java: "Set",
            egglog: None,
        }),
    });


    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Set[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Set[T]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def new(): Set[T];",
            codegen: Some(Codegen {
                rust: "Set::new",
                java: "Set.new",
                egglog: None,
            }),
            eval: |_ctx, _v| Set::new().into(),
        }],
    });
}
