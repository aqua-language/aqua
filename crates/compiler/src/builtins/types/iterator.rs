use std::rc::Rc;

use crate::aqua;
use crate::builtins::value::Value;
use crate::builtins::Context;
use crate::builtins::Decl;

use linkme::distributed_slice;
use runtime::prelude::Dict;
use runtime::prelude::Set;

use crate::builtins::DECLS;

use super::function::Function;

#[derive(Debug, Clone)]
pub enum Adaptor {
    ScanVec(Vec<Value>),
    ScanSet(Set<Value>),
    ScanDict(Dict<Value, Value>),
    Map(Rc<Adaptor>, Function),
    Filter(Rc<Adaptor>, Function),
    FlatMap(Rc<Adaptor>, Function),
}

impl Adaptor {
    pub fn as_scan_vec(&self) -> Option<&Vec<Value>> {
        match self {
            Adaptor::ScanVec(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_scan_set(&self) -> Option<&Set<Value>> {
        match self {
            Adaptor::ScanSet(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<(&Adaptor, &Function)> {
        match self {
            Adaptor::Map(a, f) => Some((a, f)),
            _ => None,
        }
    }

    pub fn as_filter(&self) -> Option<(&Adaptor, &Function)> {
        match self {
            Adaptor::Filter(a, f) => Some((a, f)),
            _ => None,
        }
    }
}

#[distributed_slice(DECLS)]
fn declare(ctx: &mut Context) {
    ctx.declare(Decl::Type {
        docs: "An iterator over a Vec.",
        aqua: aqua!("type VecIterator[T];"),
        codegen: None,
    });

    // ctx.declare(Decl::Impl {
    //     aqua: aqua!("impl[T] IntoIterator[VecIterator[T]]"),
    //     decls: &[ImplDecl::Def {
    //         docs: "",
    //         aqua: aqua!("def next(v: VecIterator[T]): Option[T];"),
    //         codegen: None,
    //         eval: |_ctx, _v| todo!(),
    //     }],
    // });

    // ctx.declare(Decl::Impl {
    //     aqua: aqua!("impl[T] Iterator[VecIterator[T]]"),
    //     decls: &[ImplDecl::Def {
    //         docs: "",
    //         aqua: aqua!("def next(v: VecIterator[T]): Option[T];"),
    //         codegen: None,
    //         eval: |_ctx, _v| todo!(),
    //     }],
    // })
}
