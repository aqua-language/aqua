use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

pub type Vec = runtime::builtins::vec::Vec<crate::builtins::Value>;

impl Compiler {
    pub(super) fn declare_vec(&mut self) {
        self.declare_type("type Vec[T];", BuiltinType { rust: "Vec" });

        self.declare_impl(
            "impl[T] Vec[T] {
                 def new(): Vec[T];
                 def push(v:Vec[T], x:T): ();
                 def pop(v:Vec[T]): Option[T];
                 def len(v:Vec[T]): usize;
                 def get(v:Vec[T], i:usize): Option[T];
                 def insert(v:Vec[T], i:usize, x:T): ();
                 def is_empty(v:Vec[T]): bool;
                 def sort(v:Vec[T]): ();
                 def remove(v:Vec[T], i:usize): Option[T];
                 def clear(v:Vec[T]): ();
             }",
            [
                BuiltinDef {
                    rust: "Vec::new",
                    fun: |_ctx, _t, _v| Vec::new().into(),
                },
                BuiltinDef {
                    rust: "Vec::push",
                    fun: |_ctx, _t, _v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::pop",
                    fun: |_ctx, _t, _v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::len",
                    fun: |_ctx, _t, v| {
                        let a0 = v[0].as_vec();
                        a0.len().into()
                    },
                },
                BuiltinDef {
                    rust: "Vec::get",
                    fun: |_ctx, _t, v| {
                        let a0 = v[0].as_vec();
                        let a1 = v[1].as_usize();
                        a0.get(a1).map(Into::into).into()
                    },
                },
                BuiltinDef {
                    rust: "Vec::insert",
                    fun: |_ctx, _t, _v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::is_empty",
                    fun: |_ctx, _t, v| {
                        let a0 = v[0].as_vec();
                        a0.is_empty().into()
                    },
                },
                BuiltinDef {
                    rust: "Vec::sort",
                    fun: |_ctx, _t, v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::remove",
                    fun: |_ctx, _t, v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::clear",
                    fun: |_ctx, _t, v| todo!(),
                },
                BuiltinDef {
                    rust: "Vec::extend",
                    fun: |_ctx, _t, v| todo!(),
                },
            ],
        );
    }
}
