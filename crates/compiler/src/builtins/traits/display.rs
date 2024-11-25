use crate::aqua;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: aqua! {
            "trait Display[T] {
                 def toString(v: T): String;
             }"
        },
    });
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Array(v) => write!(f, "{v}"),
            Value::Blob(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Char(v) => write!(f, "{v}"),
            Value::Dict(v) => write!(f, "{v}"),
            Value::Window(v) => write!(f, "{v}"),
            Value::Duration(v) => write!(f, "{v}"),
            Value::Format(v) => write!(f, "{v}"),
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
            Value::KeyedStream(v) => write!(f, "{v}"),
            Value::Dataflow(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Time(v) => write!(f, "{v}"),
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
            Value::Url(v) => write!(f, "{v}"),
            Value::Iterator(_) => unreachable!(),
            Value::Storage(_) => todo!(),
        }
    }
}
