use crate::ast::BuiltinDef;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_never(&mut self) {
        self.declare_def(
            "def unreachable(): !;",
            BuiltinDef {
                rust: "(|| unreachable!())",
                fun: |_ctx, _v| unreachable!(),
            },
        );

        self.declare_def(
            "def panic(msg: String): !;",
            BuiltinDef {
                rust: r#"(|msg| panic!("{}", msg))"#,
                fun: |_ctx, v| {
                    let v0 = v[0].as_string();
                    panic!("{}", v0);
                },
            },
        );

        self.declare_def(
            "def exit(): !;",
            BuiltinDef {
                rust: "(|| std::process::exit(0))",
                fun: |_ctx, _v| {
                    std::process::exit(0);
                },
            },
        );
    }
}
