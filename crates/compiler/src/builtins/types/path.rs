use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Path;",
        codegen: Some(Codegen {
            rust: "Path",
            java: "Path",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Path",
        decls: &[
            ImplDecl::Def {
                aqua: "def new(full: String): Path;",
                codegen: Some(Codegen {
                    rust: "Path::new",
                    java: "Path.new",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let a0 = v[0].as_string();
                    runtime::builtins::path::Path::new(a0.to_string()).into()
                },
            },
            ImplDecl::Def {
                aqua: "def join(a0: Path, a1: String): Path;",
                codegen: Some(Codegen {
                    rust: "Path::join",
                    java: "Path.join",
                    egglog: None,
                }),
                fun: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_path();
                    // let a1 = v[0].as_string();
                    // a0.join(a1).into()
                },
            },
        ],
    });
}
