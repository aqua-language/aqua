use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type i32;",
        codegen: Some(Codegen {
            rust: "i32",
            java: "int",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl i32",
        decls: &[ImplDecl::Def {
            aqua: "def abs(a:i32): i32;",
            codegen: Some(Codegen {
                rust: "i32::abs",
                java: "Math.abs",
                egglog: None,
            }),
            fun: |_ctx, v| {
                let v0 = v[0].as_i32();
                v0.abs().into()
            },
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Add[i32,i32]",
        decls: &[
            ImplDecl::Type {
                aqua: "type Output = i32;",
            },
            ImplDecl::Def {
                aqua: "def add(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a+b)",
                    java: "(a,b) -> a+b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 + v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Sub[i32,i32]",
        decls: &[
            ImplDecl::Type {
                aqua: "type Output = i32;",
            },
            ImplDecl::Def {
                aqua: "def sub(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a-b)",
                    java: "(a,b) -> a-b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 - v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Mul[i32,i32]",
        decls: &[
            ImplDecl::Type {
                aqua: "type Output = i32;",
            },
            ImplDecl::Def {
                aqua: "def mul(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a*b)",
                    java: "(a,b) -> a*b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 * v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Div[i32,i32]",
        decls: &[
            ImplDecl::Type {
                aqua: "type Output = i32;",
            },
            ImplDecl::Def {
                aqua: "def div(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a/b)",
                    java: "(a,b) -> a/b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 / v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Neg[i32]",
        decls: &[
            ImplDecl::Type {
                aqua: "type Output = i32;",
            },
            ImplDecl::Def {
                aqua: "def neg(a:i32): Neg[i32]::Output;",
                codegen: Some(Codegen {
                    rust: "(|a| -a)",
                    java: "(a) -> -a",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    (-v0).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Display[i32]",
        decls: &[ImplDecl::Def {
            aqua: "def toString(v: i32): String;",
            codegen: Some(Codegen {
                rust: "(|v| v.to_string())",
                java: "(v) -> v.toString()",
                egglog: None,
            }),
            fun: |_ctx, v| {
                let v0 = v[0].as_i32();
                runtime::prelude::String::from(v0.to_string()).into()
            },
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Default[i32]",
        decls: &[ImplDecl::Def {
            aqua: "def default(): i32;",
            codegen: Some(Codegen {
                rust: "<i32 as Default>::default",
                java: "Default::default",
                egglog: None,
            }),
            fun: |_ctx, _v| 0i32.into(),
        }],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl PartialEq[i32,i32]",
        decls: &[
            ImplDecl::Def {
                aqua: "def eq(a:i32, b:i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a == b)",
                    java: "(a,b) -> a == b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 == v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def ne(a:i32, b:i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a != b)",
                    java: "(a,b) -> a != b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 != v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Ord[i32]",
        decls: &[
            ImplDecl::Def {
                aqua: "def cmp(a:i32, b:i32): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a.cmp(&b))",
                    java: "(a,b) -> a.compareTo(b)",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    v0.cmp(&v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def min(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a.min(b))",
                    java: "(a,b) -> Math.min(a,b)",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    v0.min(v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def max(a:i32, b:i32): i32;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a.max(b))",
                    java: "Math.max",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    v0.max(v1).into()
                },
            },
        ],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl PartialOrd[i32, i32]",
        decls: &[
            ImplDecl::Def {
                aqua: "def partial_cmp(a: i32, b: i32): Option[Ordering];",
                codegen: Some(Codegen {
                    rust: "(|a,b| a.partial_cmp(&b))",
                    java: "(a,b) -> a.compareTo(b)",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    crate::builtins::value::Value::Option(runtime::builtins::option::Option(
                        v0.partial_cmp(&v1)
                            .map(|o| std::rc::Rc::new(crate::builtins::value::Value::Ordering(o))),
                    ))
                },
            },
            ImplDecl::Def {
                aqua: "def lt(a: i32, b: i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a < b)",
                    java: "(a,b) -> a < b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 < v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def le(a: i32, b: i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a <= b)",
                    java: "(a,b) -> a <= b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 <= v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def gt(a: i32, b: i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a > b)",
                    java: "(a,b) -> a > b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 > v1).into()
                },
            },
            ImplDecl::Def {
                aqua: "def ge(a: i32, b: i32): bool;",
                codegen: Some(Codegen {
                    rust: "(|a,b| a >= b)",
                    java: "(a,b) -> a >= b",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 >= v1).into()
                },
            },
        ],
    });
}
