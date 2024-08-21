use std::rc::Rc;

use crate::ast::Stmt;
use crate::ast::Trait;
use crate::ast::Type;
use crate::traversal::mapper::Mapper;

pub struct Expand;

impl Expand {
    pub fn new() -> Expand {
        Expand
    }
}

impl Mapper for Expand {
    fn map_type(&mut self, t0: &Type) -> Type {
        if let Type::Assoc(b, x, _) = t0 {
            if let Some(t1) = b.as_type(x) {
                t1.clone()
            } else {
                t0.clone()
            }
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

impl Trait {
    pub fn expand(&self) -> Trait {
        Expand::new().map_trait(self)
    }
}
