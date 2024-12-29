use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;
use runtime::prelude::DeepClone;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait DeepClone[T] {
             def deep_clone(v:T): T;
         }",
    });
}

impl DeepClone for Value {
    fn deep_clone(&self) -> Self {
        match self {
            Value::Array(v) => Value::Array(v.deep_clone()),
            Value::Blob(v) => Value::Blob(v.deep_clone()),
            Value::Bool(v) => Value::Bool(v.deep_clone()),
            Value::Char(v) => Value::Char(v.deep_clone()),
            Value::Dict(v) => Value::Dict(v.deep_clone()),
            Value::Window(v) => Value::Window(v.deep_clone()),
            Value::Duration(v) => Value::Duration(v.deep_clone()),
            Value::Format(v) => Value::Format(v.deep_clone()),
            Value::F32(v) => Value::F32(v.deep_clone()),
            Value::F64(v) => Value::F64(v.deep_clone()),
            Value::File(_) => unreachable!(),
            Value::Fun(v) => Value::Fun(v.clone()),
            Value::I128(v) => Value::I128(v.deep_clone()),
            Value::I16(v) => Value::I16(v.deep_clone()),
            Value::I32(v) => Value::I32(v.deep_clone()),
            Value::I64(v) => Value::I64(v.deep_clone()),
            Value::I8(v) => Value::I8(v.deep_clone()),
            Value::Option(v) => Value::Option(v.deep_clone()),
            Value::Path(_) => unreachable!(),
            Value::Reader(v) => Value::Reader(v.deep_clone()),
            Value::Record(v) => Value::Record(v.deep_clone()),
            Value::Result(v) => Value::Result(v.deep_clone()),
            Value::Set(v) => Value::Set(v.deep_clone()),
            Value::SocketAddr(_) => unreachable!(),
            Value::Stream(_) => unreachable!(),
            Value::Dataflow(_) => unreachable!(),
            Value::String(v) => Value::String(v.deep_clone()),
            Value::Time(v) => Value::Time(v.deep_clone()),
            Value::Tuple(v) => Value::Tuple(v.deep_clone()),
            Value::U128(v) => Value::U128(v.deep_clone()),
            Value::U16(v) => Value::U16(v.deep_clone()),
            Value::U32(v) => Value::U32(v.deep_clone()),
            Value::U64(v) => Value::U64(v.deep_clone()),
            Value::U8(v) => Value::U8(v.deep_clone()),
            Value::Usize(v) => Value::Usize(v.deep_clone()),
            Value::Url(v) => Value::Url(v.deep_clone()),
            Value::Variant(v) => Value::Variant(v.deep_clone()),
            Value::Vec(v) => Value::Vec(v.deep_clone()),
            Value::Writer(v) => Value::Writer(v.deep_clone()),
            Value::Instance(_) => unreachable!(),
            Value::Ordering(v) => Value::Ordering(v.deep_clone()),
            Value::Backend(_) => unreachable!(),
            Value::Range(v) => Value::Range(v.deep_clone()),
            Value::Iterator(_) => unreachable!(),
            Value::KeyedStream(_) => unreachable!(),
            Value::Storage(_) => todo!(),
            Value::Bag(_) => todo!(),
            Value::Unit(v) => Value::Unit(v.deep_clone()),
        }
    }
}
