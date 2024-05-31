use runtime::builtins::dict::Dict;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

pub(super) fn declare(c: &mut Compiler) {
    c.declare_type("type Dict[K,V];", BuiltinType { rust: "Dict" });

    c.declare_def(
        "def dict_new[K,V](): Dict[K,V];",
        BuiltinDef {
            rust: "Dict::new",
            fun: |_ctx, _t, _v| Dict::new().into(),
        },
    );
}
