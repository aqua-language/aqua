use linkme::distributed_slice;
use runtime::builtins::writer::Writer;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Writer;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Writer",
        decls: &[
            ImplDecl::Def {
                aqua: "def stdout_writer(): Writer;",
                codegen: None,
                fun: |_, _| Writer::stdout().into(),
            },
            ImplDecl::Def {
                aqua: "def file_writer(path: Path): Writer;",
                codegen: None,
                fun: |_, v| {
                    let v0 = v[0].as_path();
                    Writer::file(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def http_writer(a0: Url): Writer;",
                codegen: None,
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = _v[0].as_url();
                    // Writer::http(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def tcp_writer(a0: SocketAddr): Writer;",
                codegen: None,
                fun: |_ctx, _v| {
                    let v0 = _v[0].as_socket_addr();
                    Writer::tcp(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def kafka_writer(a0: SocketAddr, a1: String): Writer;",
                codegen: None,
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = _v[0].as_socket_addr();
                    // let v1 = _v[1].as_string();
                    // Writer::kafka(v0, v1).into()
                },
            },
        ],
    });
}
