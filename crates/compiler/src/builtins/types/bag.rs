use std::rc::Rc;

use runtime::prelude::Duration;
use runtime::prelude::Format;
use runtime::prelude::Reader;

use crate::builtins::Context;
use crate::builtins::Decl;
use crate::builtins::DECLS;
use linkme::distributed_slice;

use super::function::Function;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bag(pub Rc<Operator>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator {
    Source(Reader, Format, Function, Duration, Duration),
    Take(Bag, i32),
    Map(Bag, Function),
    Filter(Bag, Function),
    Flatten(Bag),
    FlatMap(Bag, Function),
    Keyby(Bag, Function),
    Window(Bag, Bag, Function),
    IncrWindow(Bag, Bag, Function, Function, Function),
    Merge(Bag, Bag),
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "",
        aqua: "type Bag[T];",
        codegen: None,
    });
}
