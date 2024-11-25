use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Blob;",
        codegen: Some(Codegen {
            rust: "Blob",
            java: "Blob",
            egglog: None,
        }),
    });
}
