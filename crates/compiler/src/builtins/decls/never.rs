use crate::ast::BuiltinDef;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_never(&mut self) {
        self.declare_def(
            "def unreachable(): !;",
            BuiltinDef {
                rust: "(|| unreachable!())",
                fun: |_ctx, _t, _v| unreachable!(),
            },
        );

        self.declare_def(
            "def panic(msg: String): !;",
            BuiltinDef {
                rust: r#"(|msg| panic!("{}", msg))"#,
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_string();
                    panic!("{}", v0);
                },
            },
        );

        self.declare_def(
            "def exit(): !;",
            BuiltinDef {
                rust: "(|| std::process::exit(0))",
                fun: |_ctx, _t, _v| {
                    std::process::exit(0);
                },
            },
        );
    }
}
