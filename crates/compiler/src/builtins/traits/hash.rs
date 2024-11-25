use std::hash::Hash;

use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait Hash[T] {}",
    });
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Array(v) => v.hash(state),
            Value::Blob(_) => unreachable!(),
            Value::Bool(v) => v.hash(state),
            Value::Char(v) => v.hash(state),
            Value::Window(_) => unreachable!(),
            Value::Duration(v) => v.hash(state),
            Value::Format(_) => unreachable!(),
            Value::F64(_) => unreachable!(),
            Value::File(_) => unreachable!(),
            Value::Fun(_) => unreachable!(),
            Value::I32(v) => v.hash(state),
            // Value::Matrix(_) => unreachable!(),
            // Value::Model(_) => unreachable!(),
            Value::Option(v) => v.hash(state),
            Value::Path(v) => v.hash(state),
            Value::Reader(_) => unreachable!(),
            Value::Record(v) => v.hash(state),
            Value::Variant(v) => v.hash(state),
            Value::Result(v) => v.hash(state),
            Value::SocketAddr(_) => unreachable!(),
            Value::Stream(_) => unreachable!(),
            Value::KeyedStream(_) => unreachable!(),
            Value::String(v) => v.hash(state),
            Value::Time(v) => v.hash(state),
            Value::Tuple(v) => v.hash(state),
            Value::Usize(v) => v.hash(state),
            Value::Url(v) => v.hash(state),
            Value::Vec(v) => v.hash(state),
            Value::Writer(_) => unreachable!(),
            Value::Dict(_) => unreachable!(),
            Value::F32(_) => unreachable!(),
            Value::I128(v) => v.hash(state),
            Value::I16(v) => v.hash(state),
            Value::I64(v) => v.hash(state),
            Value::I8(v) => v.hash(state),
            // Value::Image(_) => unreachable!(),
            Value::Set(_) => unreachable!(),
            Value::Dataflow(_) => unreachable!(),
            Value::U128(v) => v.hash(state),
            Value::U16(v) => v.hash(state),
            Value::U32(v) => v.hash(state),
            Value::U64(v) => v.hash(state),
            Value::U8(v) => v.hash(state),
            Value::Instance(_) => unreachable!(),
            Value::Ordering(v) => v.hash(state),
            Value::Backend(v) => v.hash(state),
            Value::Range(v) => v.hash(state),
            Value::Iterator(_) => unreachable!(),
            Value::Storage(_) => todo!(),
        }
    }
}
