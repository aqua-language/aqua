use super::Value;

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::F32(a), Value::F32(b)) => a.partial_cmp(b),
            (Value::F64(a), Value::F64(b)) => a.partial_cmp(b),
            _ => Some(self.cmp(other)),
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::Aggregator(_), Value::Aggregator(_)) => unreachable!(),
            (Value::Array(a), Value::Array(b)) => a.cmp(b),
            (Value::Blob(_), Value::Blob(_)) => unreachable!(),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Char(a), Value::Char(b)) => a.cmp(b),
            (Value::Discretizer(_), Value::Discretizer(_)) => unreachable!(),
            (Value::Duration(a), Value::Duration(b)) => a.cmp(b),
            (Value::Encoding(a), Value::Encoding(b)) => a.cmp(b),
            (Value::F32(_), Value::F32(_)) => unreachable!(),
            (Value::F64(_), Value::F64(_)) => unreachable!(),
            (Value::File(_), Value::File(_)) => unreachable!(),
            (Value::Fun(_), Value::Fun(_)) => unreachable!(),
            (Value::I8(a), Value::I8(b)) => a.cmp(b),
            (Value::I16(a), Value::I16(b)) => a.cmp(b),
            (Value::I32(a), Value::I32(b)) => a.cmp(b),
            (Value::I64(a), Value::I64(b)) => a.cmp(b),
            (Value::U8(a), Value::U8(b)) => a.cmp(b),
            (Value::U16(a), Value::U16(b)) => a.cmp(b),
            (Value::U32(a), Value::U32(b)) => a.cmp(b),
            (Value::U64(a), Value::U64(b)) => a.cmp(b),
            (Value::Usize(a), Value::Usize(b)) => a.cmp(b),
            // (Value::Matrix(_), Value::Matrix(_)) => unreachable!(),
            // (Value::Model(_), Value::Model(_)) => unreachable!(),
            (Value::Option(a), Value::Option(b)) => a.cmp(b),
            (Value::Path(a), Value::Path(b)) => a.cmp(b),
            (Value::Reader(_), Value::Reader(_)) => unreachable!(),
            (Value::Record(a), Value::Record(b)) => a.cmp(b),
            (Value::Variant(a), Value::Variant(b)) => a.cmp(b),
            (Value::Result(a), Value::Result(b)) => a.cmp(b),
            (Value::SocketAddr(a), Value::SocketAddr(b)) => a.cmp(b),
            (Value::Stream(_), Value::Stream(_)) => unreachable!(),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Time(a), Value::Time(b)) => a.cmp(b),
            (Value::TimeSource(_), Value::TimeSource(_)) => unreachable!(),
            (Value::Tuple(a), Value::Tuple(b)) => a.cmp(b),
            // (Value::Url(a), Value::Url(b)) => a.cmp(b),
            (Value::Vec(a), Value::Vec(b)) => a.cmp(b),
            (Value::Writer(_), Value::Writer(_)) => unreachable!(),
            (x, y) if std::mem::discriminant(x) == std::mem::discriminant(y) => {
                panic!("uncovered case")
            }
            _ => unreachable!(),
        }
    }
}
