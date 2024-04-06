use runtime::builtins::time::Time;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_time(&mut self) {
        self.declare_type("type Time;", BuiltinType { rust: "Time" });
        self.declare_def(
            "def now(): Time;",
            BuiltinDef {
                rust: "Time::now",
                fun: |_ctx, _t, _v| Time::now().into(),
            },
        );
        self.declare_def(
            "def from_seconds(v0: i64): Time;",
            BuiltinDef {
                rust: "Time::from_seconds",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_i64();
                    // Time::from_seconds(v0)
                },
            },
        );
        self.declare_def(
            "def from_nanoseconds(v0: i128): Time;",
            BuiltinDef {
                rust: "Time::from_nanoseconds",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_i128().0;
                    // Time::from_nanoseconds(v0)
                },
            },
        );
        self.declare_def(
            "def seconds(v0: Time): i64;",
            BuiltinDef {
                rust: "Time::seconds",
                fun: |_ctx, _t, _v| {
                    // let _v0 = v[0].as_time();
                    todo!()
                },
            },
        );
        self.declare_def(
            "def nanoseconds(v0: Time): i128;",
            BuiltinDef {
                rust: "Time::nanoseconds",
                fun: |_ctx, _t, _v| {
                    // let _v0 = v[0].as_time();
                    todo!()
                },
            },
        );
        self.declare_def(
            "def year(v0: Time): i32;",
            BuiltinDef {
                rust: "Time::year",
                fun: |_ctx, _t, _v| {
                    // let v0 = v[0].as_time();
                    todo!()
                },
            },
        );
        self.declare_def(
            "def from_string(v0: String, v1: String): Time;",
            BuiltinDef {
                rust: "Time::from_string",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // let v1 = v[1].as_string();
                    // Time::from_string(v0, v1).into()
                },
            },
        );
        self.declare_def(
            "def into_string(v0: Time, v1: String): String;",
            BuiltinDef {
                rust: "Time::to_string",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_time();
                    // let v1 = v[1].as_string();
                    // todo!()
                },
            },
        );
    }
}
