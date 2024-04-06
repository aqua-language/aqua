use runtime::builtins::dict::Dict;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_dict(&mut self) {
        self.declare_type("type Dict[K,V];", BuiltinType { rust: "Dict" });

        self.declare_def(
            "def dict_new[K,V](): Dict[K,V];",
            BuiltinDef {
                rust: "Dict::new",
                fun: |_ctx, _t, _v| Dict::new().into(),
            },
        );
    }
}
