use crate::Compiler;
use crate::ast::BuiltinDef;

impl Compiler {
    pub(super) fn declare_unit(&mut self) {
        self.declare_def(
            "def print(s: String): ();",
            BuiltinDef {
                rust: r#"|x| println!("{}", x)"#,
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // eprintln!("{}", v0.as_ref());
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def debug[T](v: T): ();",
            BuiltinDef {
                rust: r#"|x| println!("{:?}", x)"#,
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = &v[0];
                    // eprintln!("{:?}", v0);
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def dataflow(): ();",
            BuiltinDef {
                rust: "!",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def connect(s: String): ();",
            BuiltinDef {
                rust: "!",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let v0 = v[0].as_string();
                    // match compiler_kafka::context::Context::new(Some(v0.as_ref().to_string())) {
                    //     Ok(v) => {
                    //         todo!()
                    //         // tracing::info!("Connected to Kafka broker {}", v0);
                    //         // ctx.ctx11 = v;
                    //     }
                    //     Err(v) => eprintln!("{}", v),
                    // }
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def topics(): ();",
            BuiltinDef {
                rust: "!",
                fun: |_ctx, _t, _v| {
                    todo!();
                    // if let Err(e) = ctx.ctx11.topics() {
                    //     eprintln!("{}", e);
                    // }
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def bifs(): ();",
            BuiltinDef {
                rust: "!",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // compiler_codegen_ast::write(&mut compiler_codegen::Context::stderr().colors(true), &crate::prelude::prelude());
                    // ().into()
                },
            },
        );

        self.declare_def(
            "def typeof[T](t: T): ();",
            BuiltinDef {
                rust: "!",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let t0 = &t[0];
                    // let ctx = compiler_codegen::Context::stderr()
                    //     .colors(true)
                    //     .writeln(t0, compiler_codegen_hir::write_type)
                    //     .unwrap();
                    // ().into()
                },
            },
        );
    }
}
