//! Replace type variables with types in the AST.
use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Impl;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::Type;
use crate::traversal::mappable::Mappable;
use crate::traversal::mapper::Mapper;

use super::impl_var::ImplVarValue;
use super::type_var::TypeVarValue;
use super::Constraint;
use super::Context;

pub struct Apply<'a>(&'a mut Context);

impl Apply<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Apply<'a> {
        Apply(ctx)
    }
}

impl Mapper for Apply<'_> {
    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Var(x) => match self.0.get_type_value(*x).clone() {
                TypeVarValue::Unknown(_) => t.clone(),
                TypeVarValue::Known(t) => self.map_type(&t),
            },
            _ => self._map_type(t),
        }
    }

    fn map_impl(&mut self, i: &Impl) -> Impl {
        match i {
            Impl::Var(x) => match self.0.get_impl_value(*x).clone() {
                ImplVarValue::Unknown => i.clone(),
                ImplVarValue::Known(i1) => self.map_impl(&i1),
            },
            _ => self._map_impl(i),
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Local(v) => Stmt::Local(Rc::new(self.map_stmt_local(v))),
            s => s.clone(),
        }
    }
}

impl Impl {
    pub fn apply(&self, ctx: &mut Context) -> Impl {
        self.map(&mut Apply::new(ctx))
    }
}

impl Ast {
    pub fn apply(&self, ctx: &mut Context) -> Ast {
        self.map(&mut Apply::new(ctx))
    }
}

impl Type {
    pub fn apply(&self, ctx: &mut Context) -> Type {
        self.map(&mut Apply::new(ctx))
    }
}

impl StmtDef {
    pub fn apply(&self, ctx: &mut Context) -> StmtDef {
        self.map(&mut Apply::new(ctx))
    }
}

impl Constraint {
    pub fn apply(&self, ctx: &mut Context) -> Constraint {
        self.map(&mut Apply::new(ctx))
    }
}
