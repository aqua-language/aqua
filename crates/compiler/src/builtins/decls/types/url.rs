use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_url(&mut self) {
        self.declare_type("type Url;", BuiltinType { rust: "Url" });
        self.declare_def(
            "def url(s: String): Url;",
            BuiltinDef {
                rust: "Url::parse",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // Url::parse(v0).map(Into::into).into()
                },
            },
        );
    }
}
