use anyhow::Result;
use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Child;
use std::rc::Rc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

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

impl Compiler {
    pub(super) fn declare_instance(&mut self) {
        self.declare_type("type Instance;", BuiltinType { rust: "Instance" });
        self.declare_def(
            "def logpath(inst: Instance): Path;",
            BuiltinDef {
                rust: "Instance::logpath",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_instance();
                    // v0.log.into()
                },
            },
        );

        self.declare_def(
            "def stop(inst: Instance): ();",
            BuiltinDef {
                rust: "Instance::stop",
                fun: |_ctx, v| {
                    let v0 = v[0].as_instance();
                    if let Err(e) = v0.stop() {
                        eprintln!("{e}")
                    }
                    Tuple(vec![]).into()
                },
            },
        );
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
