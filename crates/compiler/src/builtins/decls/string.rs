use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_string(&mut self) {
        self.declare_type("type String;", BuiltinType { rust: "String" });
        self.declare_def(
            "def string_new(): String;",
            BuiltinDef {
                rust: "String::new",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // String::new().into()
                },
            },
        );
        self.declare_def(
            "def string_with_capacity(a0: usize): String;",
            BuiltinDef {
                rust: "String::with_capacity",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_usize();
                    // String::with_capacity(a0).into()
                },
            },
        );
        self.declare_def(
            "def push_char(a0: String, a1: char): String;",
            BuiltinDef {
                rust: "String::push",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_char();
                    // a0.push(a1).into()
                },
            },
        );
        self.declare_def(
            "def push_string(a0: String, a1: String): String;",
            BuiltinDef {
                rust: "String::push_string",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_string();
                    // a0.push_string(a1).into()
                },
            },
        );
        self.declare_def(
            "def remove_char(a0: String, a1: usize): (String, String);",
            BuiltinDef {
                rust: "String::remove",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let (a, b) = a0.remove(a1);
                    // Tuple(vector![a.into(), b.into()]).into()
                },
            },
        );
        self.declare_def(
            "def insert_char(a0: String, a1: usize, a2: char): String;",
            BuiltinDef {
                rust: "String::insert",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let a2 = v[2].as_char();
                    // a0.insert(a1, a2).into()
                },
            },
        );
        self.declare_def(
            "def string_is_empty(a0: String): bool;",
            BuiltinDef {
                rust: "String::is_empty",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // a0.is_empty().into()
                },
            },
        );
        self.declare_def(
            "def split_off(a0: String, a1: usize): (String, String);",
            BuiltinDef {
                rust: "String::split_off",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a1 = v[1].as_usize();
                    // let (a, b) = a0.split_off(a1);
                    // Tuple(vector![a.into(), b.into()]).into()
                },
            },
        );
        self.declare_def(
            "def clear(a0: String): String;",
            BuiltinDef {
                rust: "String::clear",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // a0.clear().into()
                },
            },
        );
        self.declare_def(
            "def len(a0: String): usize;",
            BuiltinDef {
                rust: "String::len",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // a0.len().into()
                },
            },
        );
        self.declare_def(
            "def decode[T](a0: String, a1: Encoding): T;",
            BuiltinDef {
                rust: "String::decode",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = a[0].as_string().0;
                    // let a1 = a[1].as_encoding().0;
                    // a0.decode(a1)
                },
            },
        );
        self.declare_def(
            "def encode[T](a0: T, a1: Encoding): String;",
            BuiltinDef {
                rust: "String::encode",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = a[0].as_string().0;
                    // a0.encode(a[1].as_encoding())
                },
            },
        );
        self.declare_def(
            "def lines[T](a0: String): Vec[T];",
            BuiltinDef {
                rust: "String::lines",
                fun: |_ctx, _t, _v| {
                    todo!()
                    // let a0 = v[0].as_string();
                    // let a0: Vec<_> = a0
                    //     .as_ref()
                    //     .lines()
                    //     .map(|x| String::from(x).into())
                    //     .collect::<std::vec::Vec<Value>>()
                    //     .into();
                    // a0.into()
                },
            },
        );
    }
}
