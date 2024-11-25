use linkme::distributed_slice;
use runtime::builtins::time::Time;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Time;",
        codegen: Some(Codegen {
            rust: "Time",
            java: "Time",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Serde[Time]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Time",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def now(): Time;",
                codegen: Some(Codegen {
                    rust: "Time::now",
                    java: "Time.now",
                    egglog: None,
                }),
                eval: |_ctx, _v| Time::now().into(),
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def from_seconds(v0: i64): Time;",
                codegen: Some(Codegen {
                    rust: "Time::from_seconds",
                    java: "Time.fromSeconds",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_i64();
                    // Time::from_seconds(v0)
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def from_nanoseconds(v0: i128): Time;",
                codegen: Some(Codegen {
                    rust: "Time::from_nanoseconds",
                    java: "Time.fromNanoseconds",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_i128().0;
                    // Time::from_nanoseconds(v0)
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def seconds(v0: Time): i64;",
                codegen: Some(Codegen {
                    rust: "Time::seconds",
                    java: "Time.seconds",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    // let _v0 = v[0].as_time();
                    todo!()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def nanoseconds(v0: Time): i128;",
                codegen: Some(Codegen {
                    rust: "Time::nanoseconds",
                    java: "Time.nanoseconds",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    // let _v0 = v[0].as_time();
                    todo!()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def year(v0: Time): i32;",
                codegen: Some(Codegen {
                    rust: "Time::year",
                    java: "Time.year",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    // let v0 = v[0].as_time();
                    todo!()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def from_string(v0: String, v1: String): Time;",
                codegen: Some(Codegen {
                    rust: "Time::from_string",
                    java: "Time.fromString",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // let v1 = v[1].as_string();
                    // Time::from_string(v0, v1).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def into_string(v0: Time, v1: String): String;",
                codegen: Some(Codegen {
                    rust: "Time::to_string",
                    java: "Time.toString",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_time();
                    // let v1 = v[1].as_string();
                    // todo!()
                },
            },
        ],
    });
}
