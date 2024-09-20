use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        aqua: "type Matrix[T];",
        codegen: Some(Codegen {
            rust: "Matrix",
            java: "Matrix",
            egglog: None,
        }),
    });
    ctx.declare(Decl::Impl {
        aqua: "impl[T] Matrix[T]",
        decls: &[
            ImplDecl::Def {
                aqua: "def zeros[T](v:[usize;2]): Matrix[T];",
                codegen: Some(Codegen {
                    rust: "Matrix::zeros",
                    java: "Matrix.zeros",
                    egglog: None,
                }),
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0]
                    //     .as_array()
                    //     .0
                    //     .iter()
                    //     .map(|x| x.as_usize())
                    //     .collect::<std::vec::Vec<usize>>()
                    //     .into();
                    // let TNominal(x, _) = t[0].kind.as_ref().clone() else {
                    //     unreachable!()
                    // };
                    // let matrix: Matrix = match x.as_str() {
                    //     "i8" => rt::matrix::Matrix::<i8>::zeros(v0).into(),
                    //     "i16" => rt::matrix::Matrix::<i16>::zeros(v0).into(),
                    //     "i32" => rt::matrix::Matrix::<i32>::zeros(v0).into(),
                    //     "i64" => rt::matrix::Matrix::<i64>::zeros(v0).into(),
                    //     "u8" => rt::matrix::Matrix::<u8>::zeros(v0).into(),
                    //     "u16" => rt::matrix::Matrix::<u16>::zeros(v0).into(),
                    //     "u32" => rt::matrix::Matrix::<u32>::zeros(v0).into(),
                    //     "u64" => rt::matrix::Matrix::<u64>::zeros(v0).into(),
                    //     "f32" => rt::matrix::Matrix::<f32>::zeros(v0).into(),
                    //     "f64" => rt::matrix::Matrix::<f64>::zeros(v0).into(),
                    //     _ => unreachable!(),
                    // };
                    // matrix.into()
                },
            },
            ImplDecl::Def {
                aqua: "def insert_axis[T](m: Matrix[T], idx:usize): ();",
                codegen: Some(Codegen {
                    rust: "Matrix::insert_axis",
                    java: "Matrix.insert_axis",
                    egglog: None,
                }),
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.insert_axis(v1)).into()
                },
            },
            ImplDecl::Def {
                aqua: "def remove_axis[T](m: Matrix[T], idx:usize): ();",
                codegen: Some(Codegen {
                    rust: "Matrix::remove_axis",
                    java: "Matrix.remove_axis",
                    egglog: None,
                }),
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.remove_axis(v1)).into()
                },
            },
            ImplDecl::Def {
                aqua: "def into_vec[T](m: Matrix[T]): Vec[T];",
                codegen: Some(Codegen {
                    rust: "Matrix::into_vec",
                    java: "Matrix.into_vec",
                    egglog: None,
                }),
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| {
                    //     let v: rt::vec::Vec<Value> = x
                    //         .into_vec()
                    //         .iter()
                    //         .map(|x| Value::from(x))
                    //         .collect_vec()
                    //         .into();
                    //     v.into()
                    // })
                },
            },
        ],
    });
}
