use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_socket(&mut self) {
        self.declare_type("type SocketAddr;", BuiltinType { rust: "SocketAddr" });
        self.declare_def(
            "def socket(s: String): SocketAddr;",
            BuiltinDef {
                rust: "SocketAddr::parse",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // SocketAddr::parse(v0).into()
                },
            },
        );
    }
}
