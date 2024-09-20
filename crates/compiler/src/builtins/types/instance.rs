use anyhow::Result;
use linkme::distributed_slice;
use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Child;
use std::rc::Rc;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Instance;",
        codegen: None,
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Instance",
        decls: &[
            ImplDecl::Def {
                aqua: "def logpath(inst: Instance): Path;",
                codegen: None,
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_instance();
                    // v0.log.into()
                },
            },
            ImplDecl::Def {
                aqua: "def wait(inst: Instance): ();",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_instance();
                    if let Err(e) = v0.wait() {
                        eprintln!("{e}")
                    }
                    Tuple(vec![]).into()
                },
            },
            ImplDecl::Def {
                aqua: "def stop(inst: Instance): ();",
                codegen: None,
                fun: |_ctx, v| {
                    let v0 = v[0].as_instance();
                    if let Err(e) = v0.stop() {
                        eprintln!("{e}")
                    }
                    Tuple(vec![]).into()
                },
            },
        ],
    });
}

use super::tuple::Tuple;

#[derive(Debug, Clone)]
pub struct Instance {
    // pub log: Path,
    pub child: Rc<RefCell<Child>>,
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Instance()")
    }
}

impl Instance {
    pub fn wait(&self) -> Result<()> {
        let mut child = self.child.borrow_mut();
        for line in BufReader::new(child.stderr.as_mut().unwrap()).lines() {
            tracing::info!("{}", line?);
        }
        child.wait()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.child.borrow_mut().kill()?;
        self.wait()
    }
}
