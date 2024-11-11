use crate::ast::Impl;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
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
    fn map_type(&mut self, t0: &Type) -> Type {
        match t0 {
            Type::Generic(x) => {
                if let Some(t1) = self.0.get(x) {
                    t1.clone()
                } else {
                    t0.clone()
                }
            }
            _ => self._map_type(t0),
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

impl Impl {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> Impl {
        Instantiate::new(sub).map_impl(self)
    }
}

impl StmtImpl {
    pub fn instantiate(&self, ts: &[Type]) -> StmtImpl {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_impl(self);
        this.generics.clear();
        this
    }
}

impl StmtDef {
    pub fn instantiate(&self, ts: &[Type]) -> StmtDef {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_def(self);
        this.generics.clear();
        this
    }
}

impl StmtEnum {
    pub fn instantiate(&self, ts: &[Type]) -> StmtEnum {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_enum(self);
        this.generics.clear();
        this
    }
}

impl StmtStruct {
    pub fn instantiate(&self, ts: &[Type]) -> StmtStruct {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_struct(self);
        this.generics.clear();
        this
    }
}

impl StmtTrait {
    pub fn instantiate(&self, ts: &[Type]) -> StmtTrait {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_trait(self);
        this.generics.clear();
        this
    }
}

impl StmtTraitDef {
    pub fn instantiate(&self, ts: &[Type]) -> StmtTraitDef {
        let sub = gsub(&self.generics, ts);
        let mut this = Instantiate::new(&sub).map_stmt_trait_def(self);
        this.generics.clear();
        this
    }
}

fn gsub(gs: &[Name], ts: &[Type]) -> Map<Name, Type> {
    gs.iter().cloned().zip(ts.iter().cloned()).collect()
}
