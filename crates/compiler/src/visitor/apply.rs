use crate::ast::Stmt;
use crate::ast::Type;

use super::Visitor;

pub struct Apply<F>(F);

impl<F> Visitor for Apply<F>
where
    F: FnMut(&Type) -> Type,
{
    fn visit_type(&mut self, ty: &Type) -> Type {
        (self.0)(ty)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Var(_) => todo!(),
            Stmt::Def(_) => todo!(),
            Stmt::Trait(_) => todo!(),
            Stmt::Impl(_) => todo!(),
            Stmt::Struct(_) => todo!(),
            Stmt::Enum(_) => todo!(),
            Stmt::Type(_) => todo!(),
            Stmt::Expr(_) => todo!(),
            Stmt::Err(_) => todo!(),
        }
    }
}
