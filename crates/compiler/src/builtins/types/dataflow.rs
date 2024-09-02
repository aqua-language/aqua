use std::rc::Rc;

use runtime::builtins::encoding::Encoding;
use runtime::builtins::writer::Writer;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::builtins::types::stream::Stream;
use crate::builtins::value::Value;
use crate::codegen::Codegen;
use crate::Compiler;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Dataflow {
    Collocate(Rc<Dataflow>, Rc<Dataflow>),
    Sink(Stream, Writer, Encoding),
}

impl std::fmt::Display for Dataflow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Dataflow::Collocate(a, b) => write!(f, "collocate({a}, {b})"),
            Dataflow::Sink(s, w, e) => write!(f, "sink({s}, {w}, {e})"),
        }
    }
}

impl Compiler {
    pub(super) fn declare_dataflow(&mut self) {
        self.declare_type("type Dataflow;", BuiltinType { rust: "Dataflow" });
        self.declare_def(
            "def collocate(a:Dataflow, b:Dataflow): Dataflow;",
            BuiltinDef {
                rust: "Dataflow::collocate",
                fun: |_ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let v1 = v[1].as_dataflow();
                    Dataflow::Collocate(Rc::new(v0), Rc::new(v1)).into()
                },
            },
        );
        self.declare_def(
            "def run(df:Dataflow): Instance;",
            BuiltinDef {
                rust: "Dataflow::run",
                fun: |ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let package = ctx
                        .workspace
                        .new_package(Codegen::new(&ctx.decls, &v0).rust())
                        .unwrap();
                    let executable = package.compile().unwrap();
                    let instance = executable.run_locally().unwrap();
                    Value::Instance(instance).into()
                },
            },
        );
    }
}
