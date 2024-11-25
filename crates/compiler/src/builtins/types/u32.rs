use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type u32;",
        codegen: Some(Codegen {
            rust: "u32",
            java: "int",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[u32]",
        decls: &[],
    });
}
