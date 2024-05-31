use runtime::builtins::set::Set;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_set(&mut self) {
        self.declare_type("type Set[T];", BuiltinType { rust: "Set" });

        self.declare_def(
            "def set_new[T](): Set[T];",
            BuiltinDef {
                rust: "Set::new",
                fun: |_ctx, _t, _v| Set::new().into(),
            },
        );
    }
}
