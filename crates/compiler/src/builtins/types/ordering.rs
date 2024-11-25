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
        docs: "",
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
                docs: "",
                aqua: "def less(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Less)",
                    java: "(|| Ordering.Less)",
                    egglog: None,
                }),
                eval: |_ctx, _v| Ordering::Less.into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def equal(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Equal)",
                    java: "(|| Ordering.Equal)",
                    egglog: None,
                }),
                eval: |_ctx, _v| Ordering::Equal.into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def greater(): Ordering;",
                codegen: Some(Codegen {
                    rust: "(|| Ordering::Greater)",
                    java: "(|| Ordering.Greater)",
                    egglog: None,
                }),
                eval: |_ctx, _v| Ordering::Greater.into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_eq(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_eq",
                    java: "Ordering.isEq",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_eq().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_ne(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_ne",
                    java: "Ordering.isNe",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_ne().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_lt(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_lt",
                    java: "Ordering.isLt",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_lt().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_gt(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_gt",
                    java: "Ordering.isGt",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_gt().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_le(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_le",
                    java: "Ordering.isLe",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_le().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_ge(a:Ordering): bool;",
                codegen: Some(Codegen {
                    rust: "Ordering::is_ge",
                    java: "Ordering.isGe",
                    egglog: None,
                }),
                eval: |_ctx, v| v[0].as_ordering().is_ge().into(),
            },
        ],
    });
}
