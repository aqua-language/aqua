use serde::Serialize;

use super::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Aggregator(_) => unreachable!(),
            Value::Array(v) => v.serialize(serializer),
            Value::Blob(v) => v.serialize(serializer),
            Value::Bool(v) => v.serialize(serializer),
            Value::Char(v) => v.serialize(serializer),
            Value::Dict(v) => v.serialize(serializer),
            Value::Discretizer(v) => v.serialize(serializer),
            Value::Duration(v) => v.serialize(serializer),
            Value::Encoding(v) => v.serialize(serializer),
            Value::F32(v) => v.serialize(serializer),
            Value::F64(v) => v.serialize(serializer),
            Value::File(_) => unreachable!(),
            Value::Fun(_) => unreachable!(),
            Value::I128(v) => v.serialize(serializer),
            Value::I16(v) => v.serialize(serializer),
            Value::I32(v) => v.serialize(serializer),
            Value::I64(v) => v.serialize(serializer),
            Value::I8(v) => v.serialize(serializer),
            // Value::Matrix(v) => v.serialize(serializer),
            // Value::Model(v) => v.serialize(serializer),
            Value::Option(v) => v.serialize(serializer),
            Value::Path(v) => v.serialize(serializer),
            Value::Reader(v) => v.serialize(serializer),
            Value::Record(v) => v.serialize(serializer),
            Value::Result(v) => v.serialize(serializer),
            Value::Set(v) => v.serialize(serializer),
            Value::SocketAddr(v) => v.serialize(serializer),
            Value::Stream(_) => unreachable!(),
            Value::String(v) => v.serialize(serializer),
            Value::Time(v) => v.serialize(serializer),
            Value::TimeSource(_) => unreachable!(),
            Value::Tuple(v) => v.serialize(serializer),
            Value::U128(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),
            Value::U8(v) => v.serialize(serializer),
            // Value::Url(v) => v.serialize(serializer),
            Value::Usize(v) => v.serialize(serializer),
            Value::Variant(v) => v.serialize(serializer),
            Value::Vec(v) => v.serialize(serializer),
            Value::Writer(v) => v.serialize(serializer),
            Value::Dataflow(_) => unreachable!(),
            Value::Instance(_) => unreachable!(),
            // Value::Image(v) => v.serialize(serializer),
        }
    }
}
