use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;
use runtime::prelude::Path;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
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
                docs: "",
                aqua: "def new(s: String): Path;",
                codegen: Some(Codegen {
                    rust: "Path::new",
                    java: "Path.new",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let a0 = v[0].as_string();
                    Path::new(a0.to_string()).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def extend(a0: Path, a1: String): Path;",
                codegen: Some(Codegen {
                    rust: "Path::join",
                    java: "Path.join",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let a0 = v[0].as_path();
                    // let a1 = v[0].as_string();
                    // a0.join(a1).into()
                },
            },
        ],
    });
}
