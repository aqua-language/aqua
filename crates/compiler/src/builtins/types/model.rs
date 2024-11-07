use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Model;",
        codegen: Some(Codegen {
            rust: "Model",
            java: "Model",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl Model",
        decls: &[
            ImplDecl::Def {
                aqua: "def load_model(): Model;",
                codegen: Some(Codegen {
                    rust: "Model::load_model",
                    java: "Model.load_model",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_blob();
                    // Model::new(v0).into()
                },
            },
            ImplDecl::Def {
                aqua: "def predict[I,O](model: Model, input: Matrix[I]): Matrix[O];",
                codegen: Some(Codegen {
                    rust: "Model::predict",
                    java: "Model.predict",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
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
        ],
    });
}
