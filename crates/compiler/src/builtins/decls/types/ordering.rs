use std::cmp::Ordering;

use runtime::builtins::writer::Writer;

use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_ordering(&mut self) {
        self.declare_type("type Ordering;", BuiltinType { rust: "Ordering" });

        self.declare_impl(
            "impl Ordering {
                 def less(): Ordering;
                 def equal(): Ordering;
                 def greater(): Ordering;
                 def is_eq(a:Ordering): bool;
                 def is_ne(a:Ordering): bool;
                 def is_lt(a:Ordering): bool;
                 def is_gt(a:Ordering): bool;
                 def is_le(a:Ordering): bool;
                 def is_ge(a:Ordering): bool;
             }",
            [
                BuiltinDef {
                    rust: "(|| Ordering::Less)",
                    fun: |_ctx, _t, _v| Ordering::Less.into(),
                },
                BuiltinDef {
                    rust: "(|| Ordering::Equal)",
                    fun: |_ctx, _t, _v| Ordering::Equal.into(),
                },
                BuiltinDef {
                    rust: "(|| Ordering::Greater)",
                    fun: |_ctx, _t, _v| Ordering::Greater.into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_eq",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_eq().into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_ne",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_ne().into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_lt",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_lt().into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_gt",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_gt().into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_le",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_le().into(),
                },
                BuiltinDef {
                    rust: "Ordering::is_ge",
                    fun: |_ctx, _t, v| v[0].as_ordering().is_ge().into(),
                },
            ],
        );
    }
}
