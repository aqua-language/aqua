use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_model(&mut self) {
        self.declare_type("type Model;", BuiltinType { rust: "Model" });

        self.declare_def(
            "def load_model(): Model;",
            BuiltinDef {
                rust: "Model::load",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_blob();
                    // Model::new(v0).into()
                },
            },
        );

        self.declare_def(
            "def predict[I,O](model: Model, input: Matrix[I]): Matrix[O];",
            BuiltinDef {
                rust: "Model::predict",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_model();
                    // let v1 = v[1].as_matrix();
                    // let t1 = &t[1];
                    // let TypeKind::TNominal(x, _) = t1.kind.as_ref() else {
                    //     todo!()
                    // };
                    // map_matrix!(v1, |v1| {
                    //     match x.as_str() {
                    //         "i8" => Matrix::I8(v0.predict::<_, i8>(v1)),
                    //         "i16" => Matrix::I16(v0.predict::<_, i16>(v1)),
                    //         "i32" => Matrix::I32(v0.predict::<_, i32>(v1)),
                    //         "i64" => Matrix::I64(v0.predict::<_, i64>(v1)),
                    //         "u8" => Matrix::U8(v0.predict::<_, u8>(v1)),
                    //         "u16" => Matrix::U16(v0.predict::<_, u16>(v1)),
                    //         "u32" => Matrix::U32(v0.predict::<_, u32>(v1)),
                    //         "u64" => Matrix::U64(v0.predict::<_, u64>(v1)),
                    //         "f32" => Matrix::F32(v0.predict::<_, f32>(v1)),
                    //         "f64" => Matrix::F64(v0.predict::<_, f64>(v1)),
                    //         x => panic!("Output type must be known at this point {x:?}"),
                    //     }
                    //     .into()
                    // })
                },
            },
        );
    }
}
