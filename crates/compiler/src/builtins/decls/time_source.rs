use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_time_source(&mut self) {
        self.declare_type("type TimeSource[T];", BuiltinType { rust: "TimeSource" });
        self.declare_def(
            "def ingestion[T](v0: Duration): TimeSource[T];",
            BuiltinDef {
                rust: "TimeSource::ingestion",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_duration();
                    // TimeSource::Ingestion {
                    //     watermark_interval: v0,
                    // }
                    // .into()
                },
            },
        );
        self.declare_def(
            "def event[T](v0: Duration, v1: Duration, v2: fun(T):Duration): TimeSource[T];",
            BuiltinDef {
                rust: "TimeSource::event",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_function();
                    // let v1 = v[1].as_duration();
                    // let v2 = v[2].as_duration();
                    // TimeSource::Event {
                    //     extractor: v0,
                    //     watermark_interval: v1,
                    //     slack: v2,
                    // }
                    // .into()
                },
            },
        );
    }
}
