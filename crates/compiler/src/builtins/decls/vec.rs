use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

pub type Vec = runtime::builtins::vec::Vec<crate::builtins::Value>;

impl Compiler {
    pub(super) fn declare_vec(&mut self) {
        self.declare_type("type Vec[T];", BuiltinType { rust: "Vec" });

        self.declare_def(
            "def vec_new[T](): Vec[T];",
            BuiltinDef {
                rust: "Vec::new",
                fun: |_ctx, _t, _v| Vec::new().into(),
            },
        );

        self.declare_def(
            "def vec_push[T](a0: Vec[T], a1: T): ();",
            BuiltinDef {
                rust: "Vec::push",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // let a1 = v[1].clone();
                    // a0.push(a1).into()
                },
            },
        );

        self.declare_def(
            "def vec_pop[T](a0: Vec[T]): Option[T];",
            BuiltinDef {
                rust: "Vec::pop",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // let b = a0.pop();
                    // Tuple(vector![b.into()]).into()
                },
            },
        );

        self.declare_def(
            "def vec_len[T](a0: Vec[T]): usize;",
            BuiltinDef {
                rust: "Vec::len",
                fun: |_ctx, _t, v| {
                    let a0 = v[0].as_vec();
                    a0.len().into()
                },
            },
        );

        self.declare_def(
            "def vec_get[T](a0: Vec[T], a1: usize): Option[T];",
            BuiltinDef {
                rust: "Vec::get",
                fun: |_ctx, _t, v| {
                    let a0 = v[0].as_vec();
                    let a1 = v[1].as_usize();
                    a0.get(a1).map(Into::into).into()
                },
            },
        );

        self.declare_def(
            "def vec_insert[T](a0: Vec[T], a1: usize, a2: T): ();",
            BuiltinDef {
                rust: "Vec::insert",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // let a1 = v[1].as_usize();
                    // let a2 = v[2].clone();
                    // a0.insert(a1, a2).into()
                },
            },
        );

        self.declare_def(
            "def vec_is_empty[T](a0: Vec[T]): bool;",
            BuiltinDef {
                rust: "Vec::is_empty",
                fun: |_ctx, _t, v| {
                    let a0 = v[0].as_vec();
                    a0.is_empty().into()
                },
            },
        );

        self.declare_def(
            "def vec_sort[T](a0: Vec[T]): ();",
            BuiltinDef {
                rust: "Vec::sort",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // a0.sort().into()
                },
            },
        );

        self.declare_def(
            "def vec_remove[T](a0: Vec[T], a1: usize): T;",
            BuiltinDef {
                rust: "Vec::remove",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // let a1 = v[1].as_usize();
                    // a0.remove(a1).into()
                },
            },
        );

        self.declare_def(
            "def vec_clear[T](a0: Vec[T]): ();",
            BuiltinDef {
                rust: "Vec::clear",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // a0.clear().into()
                },
            },
        );

        self.declare_def(
            "def vec_extend[T](a0: Vec[T], a1: Vec[T]): ();",
            BuiltinDef {
                rust: "Vec::extend",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_vec();
                    // let a1 = v[1].as_vec();
                    // a0.extend(a1).into()
                },
            },
        );
    }
}
