use runtime::builtins::reader::Reader;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_reader(&mut self) {
        self.declare_type("type Reader;", BuiltinType { rust: "Reader" });
        self.declare_def(
            "def stdin_reader(): Reader;",
            BuiltinDef {
                rust: "Reader::stdin",
                fun: |_ctx, _t, _v| Reader::stdin().into(),
            },
        );

        self.declare_def(
            "def file_reader(a0: Path, a1: bool): Reader;",
            BuiltinDef {
                rust: "Reader::file",
                fun: |_ctx, _t, v| {
                    let path = v[0].as_path();
                    let watch = v[1].as_bool();
                    Reader::file(path, watch).into()
                },
            },
        );

        self.declare_def(
            "def http_reader(a0: Url): Reader;",
            BuiltinDef {
                rust: "Reader::http",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let url = v[0].as_url();
                    // Reader::http(url).into()
                },
            },
        );

        self.declare_def(
            "def tcp_reader(a0: SocketAddr): Reader;",
            BuiltinDef {
                rust: "Reader::tcp",
                fun: |_ctx, _t, v| {
                    let addr = v[0].as_socket_addr();
                    Reader::tcp(addr).into()
                },
            },
        );

        self.declare_def(
            "def kafka_reader(a0: SocketAddr, a1: String): Reader;",
            BuiltinDef {
                rust: "Reader::kafka",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let addr = v[0].as_socket_addr();
                    // let topic = v[1].as_string();
                    // Reader::kafka(addr, topic).into()
                },
            },
        );
    }
}
