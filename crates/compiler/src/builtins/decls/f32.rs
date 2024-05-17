use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_f32(&mut self) {
        self.declare_type("type f32;", BuiltinType { rust: "f32" });

        self.declare_def(
            "def add_f32(a:f32,b:f32): f32;",
            BuiltinDef {
                rust: "f32::add_f32",
                fun: |_ctx, _t, v| {
                    let v0 = v[0].as_f32();
                    let v1 = v[1].as_f32();
                    (v0 + v1).into()
                },
            },
        );

        // self.declare_def(
        //     "def sub_f32(a:f32,b:f32): f32;",
        //     BuiltinDef {
        //         rust: "f32::sub_f32",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_f32();
        //             let v1 = v[1].as_f32();
        //             (v0 - v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def mul_f32(a:f32,b:f32): f32;",
        //     BuiltinDef {
        //         rust: "f32::mul_f32",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_f32();
        //             let v1 = v[1].as_f32();
        //             (v0 * v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def div_f32(a:f32,b:f32): f32;",
        //     BuiltinDef {
        //         rust: "f32::div_f32",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_f32();
        //             let v1 = v[1].as_f32();
        //             (v0 / v1).into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def sqrt_f32(a:f32,b:f32): f32;",
        //     BuiltinDef {
        //         rust: "f32::sqrt_f32",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_f32();
        //             v0.sqrt().into()
        //         },
        //     },
        // );
        //
        // self.declare_def(
        //     "def pow_f32(a:f32,b:f32): f32;",
        //     BuiltinDef {
        //         rust: "f32::pow_f32",
        //         fun: |_ctx, _t, v| {
        //             let v0 = v[0].as_f32();
        //             let v1 = v[1].as_f32();
        //             v0.powf(v1).into()
        //         },
        //     },
        // );
    }
}
