use linkme::distributed_slice;
use runtime::builtins::dict::Dict;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Dict[K,V];",
        codegen: Some(Codegen {
            rust: "Dict",
            java: "Dict",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[K,V] Dict[K,V]",
        decls: &[ImplDecl::Def {
            aqua: "def new(): Dict[K,V];",
            codegen: Some(Codegen {
                rust: "Dict::new",
                java: "Dict.new",
                egglog: None,
            }),
            fun: |_ctx, _v| Dict::new().into(),
        }],
    });
}
