use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_i32(&mut self) {
        self.declare_type("type i32;", BuiltinType { rust: "i32" });

        self.declare_impl(
            "impl i32 {
                def abs(a:i32): i32;
             }",
            [BuiltinDef {
                rust: "i32::abs",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    v0.abs().into()
                },
            }],
        );

        self.declare_impl(
            "impl Add[i32,i32] {
                 type Output = i32;
                 def add(a:i32, b:i32): i32;
             }",
            [BuiltinDef {
                rust: "(|a,b| a+b)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 + v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Sub[i32,i32] {
                 type Output = i32;
                 def sub(a:i32, b:i32): Sub[i32,i32]::Output;
             }",
            [BuiltinDef {
                rust: "(|a,b| a-b)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 - v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Mul[i32,i32] {
                 type Output = i32;
                 def mul(a:i32, b:i32): Mul[i32,i32]::Output;
             }",
            [BuiltinDef {
                rust: "(|a,b| a*b)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 * v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Div[i32,i32] {
                 type Output = i32;
                 def div(a:i32, b:i32): Div[i32,i32]::Output;
             }",
            [BuiltinDef {
                rust: "(|a,b| a/b)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 / v1).into()
                },
            }],
        );

        self.declare_impl(
            "impl Neg[i32] {
                 type Output = i32;
                 def neg(a:i32): Neg[i32]::Output;
             }",
            [BuiltinDef {
                rust: "(|a| -a)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    (-v0).into()
                },
            }],
        );

        self.declare_impl(
            "impl Display[i32] {
                def to_string(v: i32): String;
             }",
            [BuiltinDef {
                rust: "(|v| v.to_string())",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    runtime::prelude::String::from(v0.to_string()).into()
                },
            }],
        );

        self.declare_impl(
            "impl PartialEq[i32,i32] {
                 def eq(a:i32, b:i32): bool;
                 def ne(a:i32, b:i32): bool;
             }",
            [
                BuiltinDef {
                    rust: "(|a,b| a == b)",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_i32();
                        let v1 = v[1].as_i32();
                        (v0 == v1).into()
                    },
                },
                BuiltinDef {
                    rust: "(|a,b| a != b)",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_i32();
                        let v1 = v[1].as_i32();
                        (v0 != v1).into()
                    },
                },
            ],
        );

        self.declare_impl(
            "impl Ord {
                 def cmp(a:i32, b:i32): Ordering;
                 def min(a:i32, b:i32): i32;
                 def max(a:i32, b:i32): i32;
             }",
            [
                BuiltinDef {
                    rust: "(|a,b| a.cmp(&b))",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_i32();
                        let v1 = v[1].as_i32();
                        v0.cmp(&v1).into()
                    },
                },
                BuiltinDef {
                    rust: "(|a,b| a.min(b))",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_i32();
                        let v1 = v[1].as_i32();
                        v0.min(v1).into()
                    },
                },
                BuiltinDef {
                    rust: "(|a,b| a.max(b))",
                    fun: |_ctx, _t, v| {
                        let v0 = v[0].as_i32();
                        let v1 = v[1].as_i32();
                        v0.max(v1).into()
                    },
                },
            ],
        );

        // self.declare_def(
        //     "def i32_ge(a: i32, b: i32): bool;",
        //     BuiltinDef {
        //         rust: "(|a,b| a<=b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 <= v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_le(a: i32, b: i32): bool;",
        //     BuiltinDef {
        //         rust: "(|a,b| a<=b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 <= v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_eq(a: i32, b: i32): bool;",
        //     BuiltinDef {
        //         rust: "(|a,b| a == b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 > v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_ne(a: i32, b: i32): bool;",
        //     BuiltinDef {
        //         rust: "(|a,b| a != b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 != v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_to_usize(a: i32): usize;",
        //     BuiltinDef {
        //         rust: "(|a,b| a > b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 > v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def u32_to_i32(a: u32): i32;",
        //     BuiltinDef {
        //         rust: "(|a| a as i32)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_u32();
        //             (v0 as i32).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_to_u32(a: i32): u32;",
        //     BuiltinDef {
        //         rust: "(|a| a as u32)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             (v0 as u32).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_to_usize(a: i32): usize;",
        //     BuiltinDef {
        //         rust: "(|a| a as usize)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             (v0 as usize).into()
        //         },
        //     },
        // );
    }
}
