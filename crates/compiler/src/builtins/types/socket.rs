use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type SocketAddr;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl SocketAddr",
        decls: &[ImplDecl::Def {
            aqua: "def new(s: String): SocketAddr;",
            codegen: None,
            eval: |_ctx, _v| {
                todo!()
                // let v0 = v[0].as_string();
                // SocketAddr::parse(v0).into()
            },
        }],
    });
}
