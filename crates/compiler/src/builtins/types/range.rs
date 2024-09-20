use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Range[T];",
        codegen: Some(Codegen {
            rust: "Range",
            java: "Range",
            egglog: None,
        }),
    });
}
