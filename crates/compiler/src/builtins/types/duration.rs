use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use runtime::builtins::duration::Duration;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Duration;",
        codegen: Some(Codegen {
            rust: "Duration",
            java: "Duration",
            egglog: None,
        }),
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Duration",
        decls: &[
            ImplDecl::Def {
                aqua: "def postfix_s(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::seconds",
                    java: "Duration.seconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_seconds(v0 as i64).into()
                },
            },
            ImplDecl::Def {
                aqua: "def postfix_ms(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::milliseconds",
                    java: "Duration.milliseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_milliseconds(v0 as i64).into()
                },
            },
            ImplDecl::Def {
                aqua: "def postfix_us(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::microseconds",
                    java: "Duration.microseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_microseconds(v0 as i64).into()
                },
            },
            ImplDecl::Def {
                aqua: "def postfix_ns(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::nanoseconds",
                    java: "Duration.nanoseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_nanoseconds(v0 as i64).into()
                },
            },
            ImplDecl::Def {
                aqua: "def from_seconds(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::from_seconds",
                    java: "Duration.fromSeconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_seconds(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def from_milliseconds(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::from_milliseconds",
                    java: "Duration.fromMilliseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_milliseconds(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def from_microseconds(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::from_microseconds",
                    java: "Duration.fromMicroseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_microseconds(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def from_nanoseconds(v:i32): Duration;",
                codegen: Some(Codegen {
                    rust: "Duration::from_nanoseconds",
                    java: "Duration.fromNanoseconds",
                    egglog: None,
                }),
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_nanoseconds(v0).into()
                },
            },
        ],
    });
}
