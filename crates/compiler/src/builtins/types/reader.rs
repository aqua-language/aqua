use runtime::builtins::reader::Reader;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Reader;",
        codegen: None,
    });
    // ctx.declare_def(
    //     "def stdin_reader(): Reader;",
    //     ImplDecl::Def {
    //         rust: "Reader::stdin",
    //         fun: |_ctx, _v| Reader::stdin().into(),
    //     },
    // );

    ctx.declare(Decl::Impl {
        aqua: "impl Reader",
        decls: &[
            ImplDecl::Def {
                aqua: "def file(path: Path, watch: bool): Reader;",
                codegen: None,
                eval: |_ctx, v| {
                    let path = v[0].as_path();
                    let watch = v[1].as_bool();
                    Reader::file(path, watch).into()
                },
            },
            ImplDecl::Def {
                aqua: "def http(a0: Url): Reader;",
                codegen: None,
                eval: |_ctx, _v| {
                    todo!()
                    // let url = v[0].as_url();
                    // Reader::http(url).into()
                },
            },
            ImplDecl::Def {
                aqua: "def tcp(a0: SocketAddr): Reader;",
                codegen: None,
                eval: |_ctx, v| {
                    let addr = v[0].as_socket_addr();
                    Reader::tcp(addr).into()
                },
            },
            ImplDecl::Def {
                aqua: "def kafka(a0: SocketAddr, a1: String): Reader;",
                codegen: None,
                eval: |_ctx, _v| {
                    todo!()
                    // let addr = v[0].as_socket_addr();
                    // let topic = v[1].as_string();
                    // Reader::kafka(addr, topic).into()
                },
            },
        ],
    });
}
