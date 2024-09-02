use runtime::builtins::duration::Duration;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_duration(&mut self) {
        self.declare_type("type Duration;", BuiltinType { rust: "Duration" });
        self.declare_def(
            "def s(): Duration;",
            BuiltinDef {
                rust: "Duration::seconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_seconds(v0 as i64).into()
                },
            },
        );

        self.declare_def(
            "def ms(): Duration;",
            BuiltinDef {
                rust: "Duration::milliseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_milliseconds(v0 as i64).into()
                },
            },
        );

        self.declare_def(
            "def us(): Duration;",
            BuiltinDef {
                rust: "Duration::microseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_microseconds(v0 as i64).into()
                },
            },
        );

        self.declare_def(
            "def ns(): Duration;",
            BuiltinDef {
                rust: "Duration::nanoseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i32();
                    Duration::from_nanoseconds(v0 as i64).into()
                },
            },
        );

        self.declare_def(
            "def from_seconds(): Duration;",
            BuiltinDef {
                rust: "Duration::from_seconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_seconds(v0).into()
                },
            },
        );

        self.declare_def(
            "def from_milliseconds(): Duration;",
            BuiltinDef {
                rust: "Duration::from_milliseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_milliseconds(v0).into()
                },
            },
        );

        self.declare_def(
            "def from_microseconds(): Duration;",
            BuiltinDef {
                rust: "Duration::from_microseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_microseconds(v0).into()
                },
            },
        );

        self.declare_def(
            "def from_nanoseconds(): Duration;",
            BuiltinDef {
                rust: "Duration::from_nanoseconds",
                fun: |_ctx, v| {
                    let v0 = v[0].as_i64();
                    Duration::from_nanoseconds(v0).into()
                },
            },
        );
    }
}
