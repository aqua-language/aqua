use runtime::builtins::path::Path;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

#[derive(Debug, Clone)]
pub struct Instance {
    pub log: Path,
}

impl Compiler {
    pub(super) fn declare_instance(&mut self) {
        self.declare_type("type Instance;", BuiltinType { rust: "Instance" });
        self.declare_def(
            "def logpath(inst: Instance): Path;",
            BuiltinDef {
                rust: "Instance::logpath",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_instance();
                    v0.log.into()
                },
            },
        );

        self.declare_def(
            "def stop(inst: Instance): ();",
            BuiltinDef {
                rust: "Instance::stop",
                fun: |_ctx, _t, _v| todo!(),
            },
        );
    }
}
