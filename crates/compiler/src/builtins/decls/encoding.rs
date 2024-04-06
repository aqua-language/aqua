use runtime::builtins::encoding::Encoding;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_encoding(&mut self) {
        self.declare_type("type Encoding;", BuiltinType { rust: "Encoding" });

        self.declare_def(
            "def csv(): Encoding;",
            BuiltinDef {
                rust: "Encoding::csv",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_char();
                    Encoding::csv(v0).into()
                },
            },
        );

        self.declare_def(
            "def json(): Encoding;",
            BuiltinDef {
                rust: "Encoding::json",
                fun: |_ctx, _t, _v| Encoding::Json.into(),
            },
        );
    }
}
