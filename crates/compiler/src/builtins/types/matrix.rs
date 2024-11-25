use linkme::distributed_slice;

use crate::ast::Codegen;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::ImplDecl;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Matrix[T];",
        codegen: Some(Codegen {
            rust: "Matrix",
            java: "Matrix",
            egglog: None,
        }),
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Serde[Matrix[T]] where Serde[T]",
        decls: &[],
    });

    ctx.declare(Decl::Impl {
        aqua: "impl[T] Matrix[T]",
        decls: &[
            ImplDecl::Def {
                docs: "",
                aqua: "def zeros[T](v:[usize;2]): Matrix[T];",
                codegen: Some(Codegen {
                    rust: "Matrix::zeros",
                    java: "Matrix.zeros",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def insert_axis[T](m: Matrix[T], idx:usize): ();",
                codegen: Some(Codegen {
                    rust: "Matrix::insert_axis",
                    java: "Matrix.insert_axis",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.insert_axis(v1)).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def remove_axis[T](m: Matrix[T], idx:usize): ();",
                codegen: Some(Codegen {
                    rust: "Matrix::remove_axis",
                    java: "Matrix.remove_axis",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.remove_axis(v1)).into()
                },
            },
            ImplDecl::Def {
                docs: "",
                aqua: "def into_vec[T](m: Matrix[T]): Vec[T];",
                codegen: Some(Codegen {
                    rust: "Matrix::into_vec",
                    java: "Matrix.into_vec",
                    egglog: None,
                }),
                eval: |_ctx, _v| {
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
