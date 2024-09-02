use runtime::builtins::set::Set;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_set(&mut self) {
        self.declare_type("type Set[T];", BuiltinType { rust: "Set" });

        self.declare_def(
            "def set_new[T](): Set[T];",
            BuiltinDef {
                rust: "Set::new",
                fun: |_ctx, _v| Set::new().into(),
            },
        );
    }
}
