use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_f64(&mut self) {
        self.declare_type("type f64;", BuiltinType { rust: "f64" });
        self.declare_def(
            "def neg_f64(a:f64): f64;",
            BuiltinDef {
                rust: "(|v|-v)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    (-v0).into()
                },
            },
        );
        self.declare_def(
            "def pos_f64(a:f64): f64;",
            BuiltinDef {
                rust: "(|v|v)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f64();
                    v0.into()
                },
            },
        );
    }
}
