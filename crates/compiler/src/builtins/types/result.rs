use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_result(&mut self) {
        self.declare_type("type Result[T];", BuiltinType { rust: "Result" });
        self.declare_def(
            "def ok[T](v: T): Result[T];",
            BuiltinDef {
                rust: "Result::ok",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].clone();
                    // Result::ok(v0).into()
                },
            },
        );

        self.declare_def(
            "def error[T](v: T): Result[T];",
            BuiltinDef {
                rust: "Result::error",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // Result::error(v0).into()
                },
            },
        );

        self.declare_def(
            "def is_ok[T](v: Result[T]): bool;",
            BuiltinDef {
                rust: "Result::is_ok",
                fun: |_ctx, _v| {
                    let v0 = _v[0].as_result();
                    v0.is_ok().into()
                },
            },
        );

        self.declare_def(
            "def is_error[T](v: Result[T]): bool;",
            BuiltinDef {
                rust: "Result::is_error",
                fun: |_ctx, _v| {
                    let v0 = _v[0].as_result();
                    v0.is_error().into()
                },
            },
        );

        self.declare_def(
            "def unwrap_ok[T](v: Result[T]): T;",
            BuiltinDef {
                rust: "Result::unwrap_ok",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_result();
                    // v0.unwrap_ok()
                },
            },
        );

        self.declare_def(
            "def unwrap_error[T](v: Result[T]): T;",
            BuiltinDef {
                rust: "Result::unwrap_error",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_result();
                    // v0.unwrap_error().into()
                },
            },
        );
    }
}
