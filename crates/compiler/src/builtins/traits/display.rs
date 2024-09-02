use crate::builtins::value::Value;
use crate::Compiler;

impl Compiler {
    pub(super) fn declare_display(&mut self) {
        self.declare_trait(
            "trait Display[T] {
                 def to_string(v:T): String;
             }",
        );
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Aggregator(_) => unreachable!(),
            Value::Array(v) => write!(f, "{v}"),
            Value::Blob(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Char(v) => write!(f, "{v}"),
            Value::Dict(v) => write!(f, "{v}"),
            Value::Assigner(v) => write!(f, "{v}"),
            Value::Duration(v) => write!(f, "{v}"),
            Value::Encoding(v) => write!(f, "{v}"),
            Value::F32(v) => write!(f, "{v}"),
            Value::F64(v) => write!(f, "{v}"),
            Value::File(v) => write!(f, "{v}"),
            Value::Fun(v) => write!(f, "{v}"),
            Value::I128(v) => write!(f, "{v}"),
            Value::I16(v) => write!(f, "{v}"),
            Value::I32(v) => write!(f, "{v}"),
            Value::I64(v) => write!(f, "{v}"),
            Value::I8(v) => write!(f, "{v}"),
            Value::Option(v) => write!(f, "{v}"),
            Value::Path(v) => write!(f, "{v}"),
            Value::Reader(v) => write!(f, "{v}"),
            Value::Record(v) => write!(f, "{v}"),
            Value::Result(v) => write!(f, "{v}"),
            Value::Set(v) => write!(f, "{v}"),
            Value::SocketAddr(v) => write!(f, "{v}"),
            Value::Stream(v) => write!(f, "{v}"),
            Value::Dataflow(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Time(v) => write!(f, "{v}"),
            Value::TimeSource(v) => write!(f, "{v}"),
            Value::Tuple(v) => write!(f, "{v}"),
            Value::U128(v) => write!(f, "{v}"),
            Value::U16(v) => write!(f, "{v}"),
            Value::U32(v) => write!(f, "{v}"),
            Value::U64(v) => write!(f, "{v}"),
            Value::U8(v) => write!(f, "{v}"),
            Value::Usize(v) => write!(f, "{v}"),
            Value::Variant(v) => write!(f, "{v}"),
            Value::Vec(v) => write!(f, "{v}"),
            Value::Writer(v) => write!(f, "{v}"),
            Value::Instance(v) => write!(f, "{v}"),
            Value::Ordering(v) => write!(f, "{v:?}"),
            Value::Backend(v) => write!(f, "{v}"),
            Value::Range(v) => write!(f, "{v}"),
        }
    }
}
