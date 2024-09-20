use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type u16;",
        codegen: Some(Codegen {
            rust: "u16",
            java: "char",
            egglog: None,
        }),
    });
}
