use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Trait {
        aqua: "trait Neg[T] {
             type Output;
             def neg(v:T): Neg[T]::Output;
         }",
    });
}

impl std::ops::Neg for Value {
    type Output = Value;
    fn neg(self) -> Value {
        match self {
            Value::I32(a) => Value::I32(-a),
            Value::F32(a) => Value::F32(-a),
            _ => unreachable!(),
        }
    }
}
