use linkme::distributed_slice;
use runtime::builtins::window::Window;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Window;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[Window]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Window",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def tumbling(size:Duration): Window;",
                codegen: None,
                eval: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Window::tumbling(a0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def sliding(size:Duration, step:Duration): Window;",
                codegen: None,
                eval: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    let a1 = v[1].as_duration();
                    Window::sliding(a0, a1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def session(gap:Duration): Window;",
                codegen: None,
                eval: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Window::session(a0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def counting(size:i32): Window;",
                codegen: None,
                eval: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    Window::counting(a0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def moving(size:i32,step:i32): Window;",
                codegen: None,
                eval: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    let a1 = v[1].as_i32();
                    Window::moving(a0, a1).into()
                },
            },
        ],
    });
}
