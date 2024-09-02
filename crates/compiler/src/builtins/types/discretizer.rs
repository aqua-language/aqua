use runtime::builtins::assigner::Assigner;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_discretizer(&mut self) {
        self.declare_type(
            "type Discretizer;",
            BuiltinType {
                rust: "Discretizer",
            },
        );

        self.declare_def(
            "def tumbling(): Discretizer;",
            BuiltinDef {
                rust: "Discretizer::tumbling",
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Assigner::tumbling(a0).into()
                },
            },
        );

        self.declare_def(
            "def sliding(): Discretizer;",
            BuiltinDef {
                rust: "Discretizer::sliding",
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    let a1 = v[1].as_duration();
                    Assigner::sliding(a0, a1).into()
                },
            },
        );

        self.declare_def(
            "def session(): Discretizer;",
            BuiltinDef {
                rust: "Discretizer::session",
                fun: |_ctx, v| {
                    let a0 = v[0].as_duration();
                    Assigner::session(a0).into()
                },
            },
        );

        self.declare_def(
            "def counting(): Discretizer;",
            BuiltinDef {
                rust: "Discretizer::counting",
                fun: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    Assigner::counting(a0).into()
                },
            },
        );

        self.declare_def(
            "def moving(): Discretizer;",
            BuiltinDef {
                rust: "Discretizer::moving",
                fun: |_ctx, v| {
                    let a0 = v[0].as_i32();
                    let a1 = v[1].as_i32();
                    Assigner::moving(a0, a1).into()
                },
            },
        );
    }
}
