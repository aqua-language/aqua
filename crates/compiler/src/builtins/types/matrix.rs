use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::Compiler;

// use serde::Serialize;

// #[derive(Clone, Serialize)]
// #[serde(untagged)]
// pub enum Matrix {
//     I8(runtime::builtins::matrix::Matrix<i8>),
//     I16(runtime::builtins::matrix::Matrix<i16>),
//     I32(runtime::builtins::matrix::Matrix<i32>),
//     I64(runtime::builtins::matrix::Matrix<i64>),
//     U8(runtime::builtins::matrix::Matrix<u8>),
//     U16(runtime::builtins::matrix::Matrix<u16>),
//     U32(runtime::builtins::matrix::Matrix<u32>),
//     U64(runtime::builtins::matrix::Matrix<u64>),
//     F32(runtime::builtins::matrix::Matrix<f32>),
//     F64(runtime::builtins::matrix::Matrix<f64>),
// }
//
// impl std::fmt::Debug for Matrix {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Matrix::I8(v) => v.fmt(f),
//             Matrix::I16(v) => v.fmt(f),
//             Matrix::I32(v) => v.fmt(f),
//             Matrix::I64(v) => v.fmt(f),
//             Matrix::U8(v) => v.fmt(f),
//             Matrix::U16(v) => v.fmt(f),
//             Matrix::U32(v) => v.fmt(f),
//             Matrix::U64(v) => v.fmt(f),
//             Matrix::F32(v) => v.fmt(f),
//             Matrix::F64(v) => v.fmt(f),
//         }
//     }
// }
//
// impl Matrix {
//     pub fn as_i8(self) -> runtime::builtins::matrix::Matrix<i8> {
//         match self {
//             Matrix::I8(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_i16(self) -> runtime::builtins::matrix::Matrix<i16> {
//         match self {
//             Matrix::I16(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_i32(self) -> runtime::builtins::matrix::Matrix<i32> {
//         match self {
//             Matrix::I32(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_i64(self) -> runtime::builtins::matrix::Matrix<i64> {
//         match self {
//             Matrix::I64(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_u8(self) -> runtime::builtins::matrix::Matrix<u8> {
//         match self {
//             Matrix::U8(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_u16(self) -> runtime::builtins::matrix::Matrix<u16> {
//         match self {
//             Matrix::U16(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_u32(self) -> runtime::builtins::matrix::Matrix<u32> {
//         match self {
//             Matrix::U32(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_u64(self) -> runtime::builtins::matrix::Matrix<u64> {
//         match self {
//             Matrix::U64(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_f32(self) -> runtime::builtins::matrix::Matrix<f32> {
//         match self {
//             Matrix::F32(v) => v,
//             _ => unreachable!(),
//         }
//     }
//     pub fn as_f64(self) -> runtime::builtins::matrix::Matrix<f64> {
//         match self {
//             Matrix::F64(v) => v,
//             _ => unreachable!(),
//         }
//     }
// }
//
// #[macro_export]
// macro_rules! map_matrix {
//     { $v:expr, $f:expr } => {
//         match $v {
//             Matrix::I8(v) => $f(v),
//             Matrix::I16(v) => $f(v),
//             Matrix::I32(v) => $f(v),
//             Matrix::I64(v) => $f(v),
//             Matrix::U8(v) => $f(v),
//             Matrix::U16(v) => $f(v),
//             Matrix::U32(v) => $f(v),
//             Matrix::U64(v) => $f(v),
//             Matrix::F32(v) => $f(v),
//             Matrix::F64(v) => $f(v),
//         }
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<i8>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<i8>) -> Self {
//         Matrix::I8(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<i16>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<i16>) -> Self {
//         Matrix::I16(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<i32>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<i32>) -> Self {
//         Matrix::I32(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<i64>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<i64>) -> Self {
//         Matrix::I64(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<u8>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<u8>) -> Self {
//         Matrix::U8(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<u16>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<u16>) -> Self {
//         Matrix::U16(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<u32>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<u32>) -> Self {
//         Matrix::U32(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<u64>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<u64>) -> Self {
//         Matrix::U64(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<f32>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<f32>) -> Self {
//         Matrix::F32(v)
//     }
// }
//
// impl From<runtime::builtins::matrix::Matrix<f64>> for Matrix {
//     fn from(v: runtime::builtins::matrix::Matrix<f64>) -> Self {
//         Matrix::F64(v)
//     }
// }

impl Compiler {
    #[allow(unused)]
    pub(super) fn declare_matrix(&mut self) {
        self.declare_type("type Matrix[T];", BuiltinType { rust: "Matrix" });
        self.declare_def(
            "def zeros[T](v:[usize;2]): Matrix[T];",
            BuiltinDef {
                rust: "Matrix::zeros",
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
        );

        self.declare_def(
            "def insert_axis[T](m: Matrix[T], idx:usize): ();",
            BuiltinDef {
                rust: "Matrix::insert_axis",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.insert_axis(v1)).into()
                },
            },
        );

        self.declare_def(
            "def remove_axis[T](m: Matrix[T], idx:usize): ();",
            BuiltinDef {
                rust: "Matrix::remove_axis",
                fun: |_ctx, _v| {
                    todo!()
                    // let v0 = v[0].as_matrix();
                    // let v1 = v[1].as_usize();
                    // map_matrix!(v0, |x: rt::matrix::Matrix<_>| x.remove_axis(v1)).into()
                },
            },
        );

        self.declare_def(
            "def into_vec[T](m: Matrix[T]): Vec[T];",
            BuiltinDef {
                rust: "Matrix::into_vec",
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
        );
    }
}
