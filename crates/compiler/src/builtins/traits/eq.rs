use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        docs: "",
        aqua: "trait Eq[T] {
             def eq(a:T, b:T): bool;
             def ne(a:T, b:T): bool;
         }",
    });
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Blob(_), Value::Blob(_)) => unreachable!(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Window(_), Value::Window(_)) => unreachable!(),
            (Value::Duration(a), Value::Duration(b)) => a == b,
            (Value::Format(a), Value::Format(b)) => a == b,
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
            (Value::Option(a), Value::Option(b)) => a == b,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::Reader(a), Value::Reader(b)) => a == b,
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Variant(a), Value::Variant(b)) => a == b,
            (Value::Result(a), Value::Result(b)) => a == b,
            (Value::SocketAddr(a), Value::SocketAddr(b)) => a == b,
            (Value::Stream(a), Value::Stream(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Usize(a), Value::Usize(b)) => a == b,
            // (Value::Url(a), Value::Url(b)) => a == b,
            (Value::Vec(a), Value::Vec(b)) => a == b,
            (Value::Writer(a), Value::Writer(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Dataflow(a), Value::Dataflow(b)) => a == b,
            _ => unreachable!(
                "Attempted to compare incompatible types \n{:?} \nand \n{:?}",
                self, other
            ),
        }
    }
}

impl Eq for Value {}
