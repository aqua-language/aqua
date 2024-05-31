use std::rc::Rc;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;
use runtime::builtins::option::Option;

impl Compiler {
    pub(super) fn declare_option(&mut self) {
        self.declare_type("type Option[T];", BuiltinType { rust: "Option" });

        self.declare_impl(
            "impl[T] Option[T] {
                 def some(v: T): Option[T];
                 def none(): Option[T];
                 def is_some(v: Option[T]): bool;
                 def unwrap(v: Option[T]): T;
             }",
            [
                BuiltinDef {
                    rust: "Option::some",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].clone();
                        Option::some(Rc::new(v0)).into()
                    },
                },
                BuiltinDef {
                    rust: "Option::none",
                    fun: |_ctx, _t, _v| Option::none().into(),
                },
                BuiltinDef {
                    rust: "Option::is_some",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_option();
                        v0.is_some().into()
                    },
                },
                BuiltinDef {
                    rust: "Option::unwrap",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_option();
                        v0.unwrap().as_ref().clone()
                    },
                },
            ],
        );
    }
}
