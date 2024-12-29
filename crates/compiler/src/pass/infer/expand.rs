use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Stmt;
use crate::ast::Impl;
use crate::ast::Type;
use crate::traversal::mapper::Mappable;
use crate::traversal::mapper::Mapper;

use super::Constraint;

pub struct Expand;

impl Expand {
    pub fn new() -> Expand {
        Expand
    }
}

impl Mapper for Expand {
    fn map_type(&mut self, t0: &Type) -> Type {
        if let Type::Assoc(_b, _x, _) = t0 {
            todo!()
            // if let Some(t1) = b.as_trait().unwrap().xts.get(x) {
            //     t1.map(self)
            // } else {
            //     t0.clone()
            // }
        } else {
            self._map_type(t0)
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.map_stmt_var(s))),
            Stmt::Expr(s) => Stmt::Expr(Rc::new(self.map_expr(s))),
            _ => s.clone(),
        }
    }
}

impl Impl {
    /// Expand all associated types in a trait.
    pub fn expand(&self) -> Impl {
        self.map(&mut Expand::new())
    }
}

impl Ast {
    /// Expand all associated types in a program.
    pub fn expand(&self) -> Ast {
        self.map(&mut Expand::new())
    }
}

impl Type {
    /// Expand all associated types in a type.
    pub fn expand(&self) -> Type {
        self.map(&mut Expand::new())
    }
}

impl Constraint {
    /// Expand all associated types in a constraint.
    pub fn expand(&self) -> Constraint {
        self.map(&mut Expand::new())
    }
}
