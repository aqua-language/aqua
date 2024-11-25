use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type f64;",
        codegen: Some(Codegen {
            rust: "f64",
            java: "double",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl f64",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def abs(a:f64): f64;",
            codegen: Some(Codegen {
                rust: "f64::abs",
                java: "Math.abs",
                egglog: None,
            }),
            eval: |_ctx, v| {
                let v0 = v[0].as_f64();
                v0.abs().into()
            },
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[f64]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Add[f64,f64]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def add(a:f64, b:f64): f64;",
                codegen: Some(Codegen {
                    rust: "f64::add_f64",
                    java: "(a,b) -> a+b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 + v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Sub[f64,f64]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def sub(a:f64, b:f64): f64;",
                codegen: Some(Codegen {
                    rust: "f64::sub_f64",
                    java: "(a,b) -> a-b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 - v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Mul[f64,f64]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def mul(a:f64, b:f64): f64; ",
                codegen: Some(Codegen {
                    rust: "f64::mul_f64",
                    java: "(a,b) -> a*b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 * v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Div[f64,f64]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def div(a:f64, b:f64): f64;",
                codegen: Some(Codegen {
                    rust: "f64::div_f64",
                    java: "(a,b) -> a/b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_f64();
                    (v0 / v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Add[f64,i32]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def add(a:f64, b:i32): f64;",
                codegen: Some(Codegen {
                    rust: "f64::add_i32",
                    java: "(a,b) -> a+b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_f64();
                    let v1 = v[1].as_i32();
                    (v0 + v1 as f64).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Add[i32,f64]",
        decls: &[
            ImplDecl::Type {
                docs: "",
                aqua: "type Output = f64;",
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def add(a:i32, b:f64): f64;",
                codegen: Some(Codegen {
                    rust: "f64::add_i32",
                    java: "(a,b) -> a+b",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_f64();
                    (v0 as f64 + v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Display[f64]",
        decls: &[ImplDecl::Def {
            docs: "",
            aqua: "def toString(a:f64): String;",
            codegen: Some(Codegen {
                rust: "f64::display",
                java: "String.valueOf",
                egglog: None,
            }),
            eval: |_ctx, v| {
                let v0 = v[0].as_f64();
                Value::String(runtime::prelude::String::from(v0.to_string())).into()
            },
        }],
    })
}
