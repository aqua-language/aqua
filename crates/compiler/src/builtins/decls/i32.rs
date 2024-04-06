use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_i32(&mut self) {
        self.declare_type("type i32;", BuiltinType { rust: "i32" });

        self.declare_impl(
            "impl Add[i32,i32] {
                 type Output = i32;
                 def add(a:i32, b:i32): i32 = i32_add(a, b);
             }",
        );
        //
        // self.declare_def(
        //     "def i32_neg(a: i32): i32;",
        //     BuiltinDef {
        //         rust: "(|a| !a)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             (!v0).into()
        //         },
        //     },
        // );

        self.declare_def(
            "def i32_add(a: i32, b: i32): i32;",
            BuiltinDef {
                rust: "(|a,b| a+b)",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_i32();
                    let v1 = v[1].as_i32();
                    (v0 + v1).into()
                },
            },
        );

        // self.declare_def(
        //     "def i32_sub(a: i32, b: i32): i32;",
        //     BuiltinDef {
        //         rust: "(|a,b| a-b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 - v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_mul(a: i32, b: i32): i32;",
        //     BuiltinDef {
        //         rust: "(|a,b| a*b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 * v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def i32_div(a: i32, b: i32): i32;",
        //     BuiltinDef {
        //         rust: "(|a,b| a/b)",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_i32();
        //             let v1 = v[1].as_i32();
        //             (v0 / v1).into()
        //         },
        //     },
        // );
        //
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
        //
        // self.declare_def(
        //     "def i32_to_string(a: i32): String;",
        //     BuiltinDef {
        //         rust: "(|a| a.to_string())",
        //         fun: |_ctx, _t, _v| {
        //             // let v0 = v[0].as_i32();
        //             // String::from(v0.to_string()).into()
        //             todo!()
        //         },
        //     },
        // );
    }
}