// use super::Matrix;
use super::Value;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Aggregator(_), Value::Aggregator(_)) => unreachable!(),
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Blob(_), Value::Blob(_)) => unreachable!(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Discretizer(_), Value::Discretizer(_)) => unreachable!(),
            (Value::Duration(a), Value::Duration(b)) => a == b,
            (Value::Encoding(a), Value::Encoding(b)) => a == b,
            (Value::F32(_), Value::F32(_)) => unreachable!(),
            (Value::F64(_), Value::F64(_)) => unreachable!(),
            (Value::File(_), Value::File(_)) => unreachable!(),
            (Value::Fun(_), Value::Fun(_)) => unreachable!(),
            (Value::I8(a), Value::I8(b)) => a == b,
            (Value::I16(a), Value::I16(b)) => a == b,
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::U8(a), Value::U8(b)) => a == b,
            (Value::U16(a), Value::U16(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            // (Value::Matrix(a), Value::Matrix(b)) => match (a, b) {
            //     (Matrix::I8(a), Matrix::I8(b)) => a == b,
            //     (Matrix::I16(a), Matrix::I16(b)) => a == b,
            //     (Matrix::I32(a), Matrix::I32(b)) => a == b,
            //     (Matrix::I64(a), Matrix::I64(b)) => a == b,
            //     (Matrix::U8(a), Matrix::U8(b)) => a == b,
            //     (Matrix::U16(a), Matrix::U16(b)) => a == b,
            //     (Matrix::U32(a), Matrix::U32(b)) => a == b,
            //     (Matrix::U64(a), Matrix::U64(b)) => a == b,
            //     (Matrix::F32(_), Matrix::F32(_)) => unreachable!(),
            //     (Matrix::F64(_), Matrix::F64(_)) => unreachable!(),
            //     _ => unreachable!(),
            // },
            // (Value::Model(_), Value::Model(_)) => unreachable!(),
            (Value::Option(a), Value::Option(b)) => a == b,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::Reader(_), Value::Reader(_)) => unreachable!(),
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Variant(a), Value::Variant(b)) => a == b,
            (Value::Result(a), Value::Result(b)) => a == b,
            (Value::SocketAddr(a), Value::SocketAddr(b)) => a == b,
            (Value::Stream(_), Value::Stream(_)) => unreachable!(),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::TimeSource(_), Value::TimeSource(_)) => unreachable!(),
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Usize(a), Value::Usize(b)) => a == b,
            // (Value::Url(a), Value::Url(b)) => a == b,
            (Value::Vec(a), Value::Vec(b)) => a == b,
            (Value::Writer(_), Value::Writer(_)) => unreachable!(),
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            _ => unreachable!(
                "Attempted to compare incompatible types \n{:?} \nand \n{:?}",
                self, other
            ),
        }
    }
}

impl Eq for Value {}
