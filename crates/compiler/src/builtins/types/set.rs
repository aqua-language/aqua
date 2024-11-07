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
        aqua: "type Set[T];",
        codegen: Some(Codegen {
            rust: "Set",
            java: "Set",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Set",
        decls: &[ImplDecl::Def {
            aqua: "def new[T](): Set[T];",
            codegen: Some(Codegen {
                rust: "Set::new",
                java: "Set.new",
                egglog: None,
            }),
            eval: |_ctx, _v| Set::new().into(),
        }],
    });
}
