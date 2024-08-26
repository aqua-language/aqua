use crate::ast::Map;
use crate::ast::Name;
use crate::ast::StmtImpl;
use crate::ast::Trait;
use crate::ast::Type;
use crate::traversal::mapper::Mapper;

use super::annotate::Annotate;
use super::Context;

pub struct Instantiate<'a>(&'a Map<Name, Type>);

impl<'a> Instantiate<'a> {
    pub fn new(sub: &'a Map<Name, Type>) -> Instantiate<'a> {
        Instantiate(sub)
    }
}

impl<'a> Mapper for Instantiate<'a> {
    fn map_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Generic(x) => self.0.get(x).unwrap().clone(),
            _ => self._map_type(ty),
        }
    }
}

impl Type {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> Type {
        Instantiate::new(sub).map_type(self)
    }
    pub fn annotate(&self, ctx: &mut Context) -> Type {
        Annotate::new(ctx).map_type(self)
    }
}

impl Trait {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> Trait {
        Instantiate::new(sub).map_trait(self)
    }
}

impl StmtImpl {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> StmtImpl {
        Instantiate::new(sub).map_stmt_impl(self)
    }
}
