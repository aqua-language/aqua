use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_string(&mut self) {
        self.declare_type("type String;", BuiltinType { rust: "String" });
        self.declare_impl(
            "impl String {
                 def new(): String;
                 def with_capacity(cap: usize): String;
                 def push_char(s: String, c: char): ();
                 def push(s1: String, s2: String): ();
                 def remove(s: String, idx: usize): char;
                 def insert(s: String, idx: usize, c: char): ();
                 def is_empty(s: String): bool;
                 def split_off(s: String, idx: usize): (String, String);
                 def clear(s: String): ();
                 def len(s: String): usize;
                 # def decode[T](s: String, e: Encoding): T;
                 # def encode[T](s: T, e: Encoding): String;
                 def lines[T](s: String): Vec[T];
             }",
            [
                BuiltinDef {
                    rust: "String::new",
                    fun: |_ctx, _v| {
                        todo!()
                        // String::new().into()
                    },
                },
                BuiltinDef {
                    rust: "String::with_capacity",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_usize();
                        // String::with_capacity(a0).into()
                    },
                },
                BuiltinDef {
                    rust: "String::push",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // let a1 = v[1].as_char();
                        // a0.push(a1).into()
                    },
                },
                BuiltinDef {
                    rust: "String::push_string",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // let a1 = v[1].as_string();
                        // a0.push_string(a1).into()
                    },
                },
                BuiltinDef {
                    rust: "String::remove",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // let a1 = v[1].as_usize();
                        // let (a, b) = a0.remove(a1);
                        // Tuple(vector![a.into(), b.into()]).into()
                    },
                },
                BuiltinDef {
                    rust: "String::insert",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // let a1 = v[1].as_usize();
                        // let a2 = v[2].as_char();
                        // a0.insert(a1, a2).into()
                    },
                },
                BuiltinDef {
                    rust: "String::is_empty",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // a0.is_empty().into()
                    },
                },
                BuiltinDef {
                    rust: "String::split_off",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // let a1 = v[1].as_usize();
                        // let (a, b) = a0.split_off(a1);
                        // Tuple(vector![a.into(), b.into()]).into()
                    },
                },
                BuiltinDef {
                    rust: "String::clear",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // a0.clear().into()
                    },
                },
                BuiltinDef {
                    rust: "String::len",
                    fun: |_ctx, _v| {
                        todo!()
                        // let a0 = v[0].as_string();
                        // a0.len().into()
                    },
                },
                // BuiltinDef {
                //     rust: "String::decode",
                //     fun: |_ctx, _t, _v| {
                //         todo!()
                //         // let a0 = a[0].as_string().0;
                //         // let a1 = a[1].as_encoding().0;
                //         // a0.decode(a1)
                //     },
                // },
                // BuiltinDef {
                //     rust: "String::encode",
                //     fun: |_ctx, _t, _v| {
                //         todo!()
                //         // let a0 = a[0].as_string().0;
                //         // a0.encode(a[1].as_encoding())
                //     },
                // },
                BuiltinDef {
                    rust: "String::lines",
                    fun: |_ctx, _v| {
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
            ],
        );
    }
}
