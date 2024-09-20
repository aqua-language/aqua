use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;
use std::cmp::Ordering;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Ordering;",
        codegen: Some(Codegen {
            rust: "Ordering",
            java: "Ordering",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Ordering",
        decls: &[
            ImplDecl::Def {
                aqua: "def less(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Less)",
                    java: "(|| Ordering.Less)",
                    egglog: None,
                }),
                fun: |_ctx, _v| Ordering::Less.into(),
            },
            ImplDecl::Def {
                aqua: "def equal(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Equal)",
                    java: "(|| Ordering.Equal)",
                    egglog: None,
                }),
                fun: |_ctx, _v| Ordering::Equal.into(),
            },
            ImplDecl::Def {
                aqua: "def greater(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Greater)",
                    java: "(|| Ordering.Greater)",
                    egglog: None,
                }),
                fun: |_ctx, _v| Ordering::Greater.into(),
            },
            ImplDecl::Def {
                aqua: "def is_eq(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_eq",
                    java: "Ordering.isEq",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_eq().into(),
            },
            ImplDecl::Def {
                aqua: "def is_ne(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_ne",
                    java: "Ordering.isNe",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_ne().into(),
            },
            ImplDecl::Def {
                aqua: "def is_lt(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_lt",
                    java: "Ordering.isLt",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_lt().into(),
            },
            ImplDecl::Def {
                aqua: "def is_gt(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_gt",
                    java: "Ordering.isGt",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_gt().into(),
            },
            ImplDecl::Def {
                aqua: "def is_le(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_le",
                    java: "Ordering.isLe",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_le().into(),
            },
            ImplDecl::Def {
                aqua: "def is_ge(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_ge",
                    java: "Ordering.isGe",
                    egglog: None,
                }),
                fun: |_ctx, v| v[0].as_ordering().is_ge().into(),
            },
        ],
    });
}
