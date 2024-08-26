use crate::ast::Trait;
use crate::ast::Type;
use crate::traversal::mapper::Mapper;

use super::Context;

pub struct Canonicalize<'a>(&'a mut Context);

impl Canonicalize<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Canonicalize<'a> {
        Canonicalize(ctx)
    }
}

impl<'a> Mapper for Canonicalize<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        if let Type::Var(x) = t {
            Type::Var(self.0.type_scope().table.find(*x))
        } else {
            self._map_type(t)
        }
    }
}

impl Trait {
    /// Canonicalize all type variables in a trait.
    pub fn canonicalize(&self, ctx: &mut Context) -> Trait {
        Canonicalize::new(ctx).map_trait(self)
    }
}
