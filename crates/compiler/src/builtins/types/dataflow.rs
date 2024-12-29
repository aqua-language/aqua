use std::rc::Rc;

use linkme::distributed_slice;
use runtime::builtins::format::Format;
use runtime::builtins::writer::Writer;

use crate::backend::runtime::flink::package::FLINK_WORKSPACE;
use crate::backend::runtime::native::package::NATIVE_WORKSPACE;
use crate::builtins::types::stream::Stream;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

use super::keyed_stream::KeyedStream;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Dataflow {
    Collocate(Rc<Dataflow>, Rc<Dataflow>),
    Sink(Stream, Writer, Format),
    KeyedSink(KeyedStream, Writer, Format),
}

impl std::fmt::Display for Dataflow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Dataflow::Collocate(a, b) => write!(f, "collocate({a}, {b})"),
            Dataflow::Sink(s, w, e) => write!(f, "sink({s}, {w}, {e})"),
            Dataflow::KeyedSink(s, w, e) => write!(f, "keyed_sink({s}, {w}, {e})"),
        }
    }
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Dataflow;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Dataflow",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def merge(a:Dataflow, b:Dataflow): Dataflow;",
                codegen: None,
                eval: |_ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let v1 = v[1].as_dataflow();
                    Dataflow::Collocate(Rc::new(v0), Rc::new(v1)).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def run(df:Dataflow, backend:Backend): Instance;",
                codegen: None,
                eval: |ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let v1 = v[1].as_backend();
                    match v1 {
                        super::backend::Backend::Native => {
                            let source = v0.native(&ctx.decls);
                            let package = NATIVE_WORKSPACE.new_package(source).unwrap();
                            let executable = package.compile().unwrap();
                            let instance = executable.run_locally().unwrap();
                            Value::Instance(instance).into()
                        }
                        super::backend::Backend::Flink => {
                            let source = v0.flink(&ctx.decls);
                            let package = FLINK_WORKSPACE.new_package(source).unwrap();
                            let executable = package.compile().unwrap();
                            let instance = executable.run().unwrap();
                            Value::Instance(instance).into()
                        }
                    }
                },
            },
        ],
    });
}
