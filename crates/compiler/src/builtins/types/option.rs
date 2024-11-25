use std::rc::Rc;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;
use runtime::builtins::option::Option;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Option[T];",
        codegen: Some(Codegen {
            rust: "Option",
            java: "Option",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Option[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Option[T]",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def some(v: T): Option[T];",
                codegen: Some(Codegen {
                    rust: "Option::some",
                    java: "Option.some",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].clone();
                    Option::some(Rc::new(v0)).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def none(): Option[T];",
                codegen: Some(Codegen {
                    rust: "Option::none",
                    java: "Option.none",
                    egglog: None,
                }),
                eval: |_ctx, _v| Option::none().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def is_some(v: Option[T]): bool;",
                codegen: Some(Codegen {
                    rust: "Option::is_some",
                    java: "Option.isSome",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_option();
                    v0.is_some().into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def unwrap(v: Option[T]): T;",
                codegen: Some(Codegen {
                    rust: "Option::unwrap",
                    java: "Option.unwrap",
                    egglog: None,
                }),
                eval: |_ctx, v| {
                    let v0 = v[0].as_option();
                    v0.unwrap().as_ref().clone()
                },
            },
        ],
    });
}
