use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_path(&mut self) {
        self.declare_type("type Path;", BuiltinType { rust: "Path" });

        self.declare_def(
            "def path(): Path;",
            BuiltinDef {
                rust: "Path::new",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // Path::new(a0).into()
                },
            },
        );

        self.declare_def(
            "def path_join(a0: Path, a1: String): Path;",
            BuiltinDef {
                rust: "Path::join",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_path();
                    // let a1 = v[0].as_string();
                    // a0.join(a1).into()
                },
            },
        );
    }
}
