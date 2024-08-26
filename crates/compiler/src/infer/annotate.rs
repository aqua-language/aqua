use std::rc::Rc;

use crate::ast::Expr;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtImpl;
use crate::ast::Type;
use crate::traversal::mapper::Mapper;

use super::type_var::TypeVarKind;
use super::Context;

pub struct Annotate<'a>(&'a mut Context);

impl Annotate<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Annotate<'a> {
        Annotate(ctx)
    }
}

impl<'a> Mapper for Annotate<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        if let Type::Unknown = t {
            self.0.fresh(TypeVarKind::General)
        } else {
            self._map_type(t)
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Var(s) => Stmt::Var(Rc::new(self.map_stmt_var(s))),
            s => s.clone(),
        }
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Int(s, Type::Unknown, e) => {
                let t = self.0.fresh(TypeVarKind::Int);
                Expr::Int(*s, t, e.clone())
            }
            Expr::Float(s, Type::Unknown, e) => {
                let t = self.0.fresh(TypeVarKind::Float);
                Expr::Float(*s, t, e.clone())
            }
            _ => self._map_expr(e),
        }
    }

    fn map_pattern(&mut self, p: &Pat) -> Pat {
        if let Pat::Int(s, Type::Unknown, e) = p {
            let t = self.0.fresh(TypeVarKind::Int);
            Pat::Int(*s, t, e.clone())
        } else {
            self._map_pattern(p)
        }
    }
}

impl StmtImpl {
    pub fn annotate(&self, ctx: &mut Context) -> StmtImpl {
        Annotate::new(ctx).map_stmt_impl(self)
    }
}

impl Program {
    pub fn annotate(&self, ctx: &mut Context) -> Program {
        Annotate::new(ctx).map_program(self)
    }
}

impl Expr {
    pub fn annotate(&self, ctx: &mut Context) -> Expr {
        Annotate::new(ctx).map_expr(self)
    }
}
