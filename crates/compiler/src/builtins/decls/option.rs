use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;
use runtime::builtins::option::Option;

impl Compiler {
    pub(super) fn declare_option(&mut self) {
        self.declare_type("type Option[T];", BuiltinType { rust: "Option" });

        self.declare_def(
            "def some[T](v: T): Option[T];",
            BuiltinDef {
                rust: "Option::some",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].clone();
                    // Option::some(v0).into()
                },
            },
        );

        self.declare_def(
            "def none[T](): Option[T];",
            BuiltinDef {
                rust: "Option::none",
                fun: |_ctx, _t, _v| Option::none().into(),
            },
        );

        self.declare_def(
            "def is_some[T](v: Option[T]): bool;",
            BuiltinDef {
                rust: "Option::is_some",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_option();
                    // v0.is_some().into()
                },
            },
        );

        self.declare_def(
            "def is_none[T](v: Option[T]): bool;",
            BuiltinDef {
                rust: "Option::is_none",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_option();
                    // v0.is_none().into()
                },
            },
        );

        self.declare_def(
            "def unwrap[T](v: Option[T]): T;",
            BuiltinDef {
                rust: "Option::unwrap",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_option();
                    // v0.unwrap()
                },
            },
        );
    }
}
