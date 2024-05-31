use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

// `and` and `or` desugar to `if` expressions instead of functions, so we don't implement them.
impl Compiler {
    pub(super) fn declare_bool(&mut self) {
        self.declare_type("type bool;", BuiltinType { rust: "bool" });

        self.declare_impl(
            "impl Not[bool] {
                 type Output = bool;
                 def not(a:bool): bool;
             }",
            [BuiltinDef {
                rust: "(|a| !a)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_bool();
                    (!v0).into()
                },
            }],
        );
    }
}
