use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Matrix {
    I8(runtime::builtins::matrix::Matrix<i8>),
    I16(runtime::builtins::matrix::Matrix<i16>),
    I32(runtime::builtins::matrix::Matrix<i32>),
    I64(runtime::builtins::matrix::Matrix<i64>),
    U8(runtime::builtins::matrix::Matrix<u8>),
    U16(runtime::builtins::matrix::Matrix<u16>),
    U32(runtime::builtins::matrix::Matrix<u32>),
    U64(runtime::builtins::matrix::Matrix<u64>),
    F32(runtime::builtins::matrix::Matrix<f32>),
    F64(runtime::builtins::matrix::Matrix<f64>),
}

impl std::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Matrix::I8(v) => v.fmt(f),
            Matrix::I16(v) => v.fmt(f),
            Matrix::I32(v) => v.fmt(f),
            Matrix::I64(v) => v.fmt(f),
            Matrix::U8(v) => v.fmt(f),
            Matrix::U16(v) => v.fmt(f),
            Matrix::U32(v) => v.fmt(f),
            Matrix::U64(v) => v.fmt(f),
            Matrix::F32(v) => v.fmt(f),
            Matrix::F64(v) => v.fmt(f),
        }
    }
}

impl Matrix {
    pub fn as_i8(self) -> runtime::builtins::matrix::Matrix<i8> {
        match self {
            Matrix::I8(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_i16(self) -> runtime::builtins::matrix::Matrix<i16> {
        match self {
            Matrix::I16(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_i32(self) -> runtime::builtins::matrix::Matrix<i32> {
        match self {
            Matrix::I32(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_i64(self) -> runtime::builtins::matrix::Matrix<i64> {
        match self {
            Matrix::I64(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_u8(self) -> runtime::builtins::matrix::Matrix<u8> {
        match self {
            Matrix::U8(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_u16(self) -> runtime::builtins::matrix::Matrix<u16> {
        match self {
            Matrix::U16(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_u32(self) -> runtime::builtins::matrix::Matrix<u32> {
        match self {
            Matrix::U32(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_u64(self) -> runtime::builtins::matrix::Matrix<u64> {
        match self {
            Matrix::U64(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_f32(self) -> runtime::builtins::matrix::Matrix<f32> {
        match self {
            Matrix::F32(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn as_f64(self) -> runtime::builtins::matrix::Matrix<f64> {
        match self {
            Matrix::F64(v) => v,
            _ => unreachable!(),
        }
    }
}

#[macro_export]
macro_rules! map_matrix {
    { $v:expr, $f:expr } => {
        match $v {
            Matrix::I8(v) => $f(v),
            Matrix::I16(v) => $f(v),
            Matrix::I32(v) => $f(v),
            Matrix::I64(v) => $f(v),
            Matrix::U8(v) => $f(v),
            Matrix::U16(v) => $f(v),
            Matrix::U32(v) => $f(v),
            Matrix::U64(v) => $f(v),
            Matrix::F32(v) => $f(v),
            Matrix::F64(v) => $f(v),
        }
    }
}

impl From<runtime::builtins::matrix::Matrix<i8>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<i8>) -> Self {
        Matrix::I8(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<i16>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<i16>) -> Self {
        Matrix::I16(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<i32>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<i32>) -> Self {
        Matrix::I32(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<i64>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<i64>) -> Self {
        Matrix::I64(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<u8>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<u8>) -> Self {
        Matrix::U8(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<u16>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<u16>) -> Self {
        Matrix::U16(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<u32>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<u32>) -> Self {
        Matrix::U32(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<u64>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<u64>) -> Self {
        Matrix::U64(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<f32>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<f32>) -> Self {
        Matrix::F32(v)
    }
}

impl From<runtime::builtins::matrix::Matrix<f64>> for Matrix {
    fn from(v: runtime::builtins::matrix::Matrix<f64>) -> Self {
        Matrix::F64(v)
    }
}
