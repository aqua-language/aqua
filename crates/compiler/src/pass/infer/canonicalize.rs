use crate::ast::Impl;
use crate::ast::Type;
use crate::traversal::mapper::Mappable;
use crate::traversal::mapper::Mapper;

use super::Constraint;
use super::Context;

pub struct Canonicalize<'a>(&'a mut Context);

impl<'a> Canonicalize<'a> {
    pub fn new(ctx: &'a mut Context) -> Canonicalize<'a> {
        Canonicalize(ctx)
    }
}

impl<'a> Mapper for Canonicalize<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        if let Type::Var(x) = t {
            Type::Var(self.0.type_scope().type_table.find(*x))
        } else {
            self._map_type(t)
        }
    }
}

impl Impl {
    pub fn canonicalize(&self, ctx: &mut Context) -> Impl {
        self.map(&mut Canonicalize::new(ctx))
    }
}

impl Constraint {
    pub fn canonicalize(&self, ctx: &mut Context) -> Constraint {
        self.map(&mut Canonicalize::new(ctx))
    }
}
