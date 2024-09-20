use linkme::distributed_slice;
use runtime::builtins::assigner::Assigner;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Assigner;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Assigner",
        decls: &[
            ImplDecl::Def {
                aqua: "def tumbling(): Assigner;",
                codegen: None,
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Assigner::tumbling(a0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def sliding(): Assigner;",
                codegen: None,
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    let a1 = v[1].as_duration();
                    Assigner::sliding(a0, a1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def session(): Assigner;",
                codegen: None,
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Assigner::session(a0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def counting(): Assigner;",
                codegen: None,
                fun: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    Assigner::counting(a0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def moving(): Assigner;",
                codegen: None,
                fun: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    let a1 = v[1].as_i32();
                    Assigner::moving(a0, a1).into()
                },
            },
        ],
    });
}
