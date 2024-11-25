use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Result[T];",
        codegen: Some(Codegen {
            rust: "Result",
            java: "Result",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Result[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Result",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def ok[T](v: T): Result[T];",
                codegen: Some(Codegen {
                    rust: "Result::ok",
                    java: "Result.ok",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].clone();
                    // Result::ok(v0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def error[T](v: T): Result[T];",
                codegen: Some(Codegen {
                    rust: "Result::error",
                    java: "Result.error",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // Result::error(v0).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_ok[T](v: Result[T]): bool;",
                codegen: Some(Codegen {
                    rust: "Result::is_ok",
                    java: "Result.isOk",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    let v0 = _v[0].as_result();
                    v0.is_ok().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_error[T](v: Result[T]): bool;",
                codegen: Some(Codegen {
                    rust: "Result::is_error",
                    java: "Result.isError",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    let v0 = _v[0].as_result();
                    v0.is_error().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def unwrap_ok[T](v: Result[T]): T;",
                codegen: Some(Codegen {
                    rust: "Result::unwrap_ok",
                    java: "Result.unwrapOk",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_result();
                    // v0.unwrap_ok()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def unwrap_error[T](v: Result[T]): T;",
                codegen: Some(Codegen {
                    rust: "Result::unwrap_error",
                    java: "Result.unwrapError",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_result();
                    // v0.unwrap_error().into()
                },
            },
        ],
    });
}
