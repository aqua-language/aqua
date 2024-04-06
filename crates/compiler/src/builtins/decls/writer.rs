use runtime::builtins::writer::Writer;

use crate::Compiler;
use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;

impl Compiler {
    pub(super) fn declare_writer(&mut self) {
        self.declare_type("type Writer;", BuiltinType { rust: "Writer" });
        self.declare_def(
            "def stdout_writer(): Writer;",
            BuiltinDef {
                rust: "Writer::stdout",
                fun: |_ctx, _t, _v| Writer::stdout().into(),
            },
        );
        self.declare_def(
            "def file_writer(a0: Path): Writer;",
            BuiltinDef {
                rust: "Writer::file",
                fun: |_ctx, _t, _v| {
                    let v0 = _v[0].as_path();
                    Writer::file(v0).into()
                },
            },
        );
        self.declare_def(
            "def http_writer(a0: Url): Writer;",
            BuiltinDef {
                rust: "Writer::http",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = _v[0].as_url();
                    // Writer::http(v0).into()
                },
            },
        );
        self.declare_def(
            "def tcp_writer(a0: SocketAddr): Writer;",
            BuiltinDef {
                rust: "Writer::tcp",
                fun: |_ctx, _t, _v| {
                    let v0 = _v[0].as_socket_addr();
                    Writer::tcp(v0).into()
                },
            },
        );
        self.declare_def(
            "def kafka_writer(a0: SocketAddr, a1: String): Writer;",
            BuiltinDef {
                rust: "Writer::kafka",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = _v[0].as_socket_addr();
                    // let v1 = _v[1].as_string();
                    // Writer::kafka(v0, v1).into()
                },
            },
        );
    }
}
