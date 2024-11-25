use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Range[T];",
        codegen: Some(Codegen {
            rust: "Range",
            java: "Range",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Range[T]] where Serde[T]",
        decls: &[],
    });
}
