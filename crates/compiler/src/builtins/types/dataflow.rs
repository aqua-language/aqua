use std::rc::Rc;

use linkme::distributed_slice;
use runtime::builtins::encoding::Encoding;
use runtime::builtins::writer::Writer;

use crate::builtins::types::stream::Stream;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;
use crate::codegen::runtime::flink::package::FLINK_WORKSPACE;
use crate::codegen::runtime::flink::DisplayFlink;
use crate::codegen::runtime::native::package::NATIVE_WORKSPACE;
use crate::codegen::runtime::native::DisplayNative;
use crate::codegen::Codegen;

use super::backend::Backend;

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

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Dataflow;",
        codegen: None,
    });
    ctx.declare(Decl::Impl {
        aqua: "impl Dataflow",
        decls: &[
            ImplDecl::Def {
                aqua: "def collocate(a:Dataflow, b:Dataflow): Dataflow;",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let v1 = v[1].as_dataflow();
                    Dataflow::Collocate(Rc::new(v0), Rc::new(v1)).into()
                },
            },
            ImplDecl::Def {
                aqua: "def run(df:Dataflow, backend:Backend): Instance;",
                codegen: None,
                fun: |ctx, v| {
                    let v0 = v[0].as_dataflow();
                    let v1 = v[1].as_backend();
                    let source = Codegen::new(&ctx.decls, &v0);
                    match v1 {
                        Backend::Native => {
                            let package = NATIVE_WORKSPACE.new_package(source.native()).unwrap();
                            let executable = package.compile().unwrap();
                            let instance = executable.run_locally().unwrap();
                            Value::Instance(instance).into()
                        }
                        Backend::Flink => {
                            let package = FLINK_WORKSPACE.new_package(source.flink()).unwrap();
                            let executable = package.compile().unwrap();
                            let instance = executable.run().unwrap();
                            Value::Instance(instance).into()
                        }
                        Backend::Spark => {
                            todo!()
                            // let package = FLINK_WORKSPACE.new_package(source.spark()).unwrap();
                            // let executable = package.compile().unwrap();
                            // let instance = executable.run().unwrap();
                            // Value::Instance(instance).into()
                        }
                    }
                },
            },
        ],
    });
}
