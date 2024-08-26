use std::rc::Rc;

use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::Trait;
use crate::ast::Type;
use crate::traversal::mapper::AcceptMapper;
use crate::traversal::mapper::Mapper;

use super::type_var::TypeVarValue;
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
            Type::Var(x) => match self.0.get_value(*x) {
                TypeVarValue::Unknown(_) => t.clone(),
                TypeVarValue::Known(t) => self.map_type(&t),
            },
            _ => self._map_type(t),
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Var(v) => Stmt::Var(Rc::new(self.map_stmt_var(v))),
            s => s.clone(),
        }
    }
}

impl Trait {
    pub fn apply(&self, ctx: &mut Context) -> Trait {
        self.map(Apply::new(ctx))
    }
}

impl Program {
    pub fn apply(&self, ctx: &mut Context) -> Program {
        self.map(Apply::new(ctx))
    }
}

impl Type {
    pub fn apply(&self, ctx: &mut Context) -> Type {
        self.map(Apply::new(ctx))
    }
}

impl StmtDef {
    pub fn apply(&self, ctx: &mut Context) -> StmtDef {
        self.map(Apply::new(ctx))
    }
}
