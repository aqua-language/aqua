use linkme::distributed_slice;
use runtime::prelude::Send;
use runtime::prelude::Sync;

use crate::ast::ExprBody;
use crate::ast::Local;
use crate::builtins::value::Value;

use crate::builtins::Context;
use crate::builtins::DECLS;

#[distributed_slice(DECLS)]
fn declare(_ctx: &mut Context) {}

#[derive(Debug, Clone, Eq, PartialEq, Send, Sync)]
pub struct Function {
    pub params: Vec<Local>,
    pub body: ExprBody,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        let mut iter = self.params.iter();
        if let Some(l) = iter.next() {
            write!(f, "{l}")?;
            for l in iter {
                write!(f, ", {l}")?;
            }
        }
        write!(f, ") => {}", self.body)
    }
}

impl Function {
    pub fn new(ls: Vec<Local>, body: ExprBody) -> Self {
        Self { params: ls, body }
    }

    pub fn call(&self, ctx: &mut crate::interpret::Context, args: &[Value]) -> Value {
        match &self.body {
            ExprBody::UserDefined(e) => ctx.scoped(|ctx| {
                for (l, v) in self.params.iter().zip(args) {
                    ctx.stack.bind(l.name, v.clone())
                }
                ctx.eval_expr(&e)
            }),
            ExprBody::Builtin(b) => (b.fun)(ctx, args),
        }
    }
}
