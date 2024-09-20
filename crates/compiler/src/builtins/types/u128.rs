use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type u128;",
        codegen: Some(Codegen {
            rust: "u128",
            java: "BigInteger",
            egglog: None,
        }),
    });
}
