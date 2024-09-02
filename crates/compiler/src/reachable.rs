use std::rc::Rc;

use crate::ast::Expr;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtStruct;
use crate::ast::Type;
use crate::builtins::value::Dataflow;
use crate::declare;
use crate::traversal::visitor::Visitor;

pub struct Context<'a> {
    pub reachable: Reachable,
    pub decls: &'a declare::Context,
}

#[derive(Default)]
pub struct Reachable {
    pub defs: Vec<Rc<StmtDef>>,
    pub enums: Vec<Rc<StmtEnum>>,
    pub structs: Vec<Rc<StmtStruct>>,
}

impl declare::Context {
    pub fn new(&self, dataflow: &Dataflow) -> Reachable {
        let mut ctx = Context {
            reachable: Reachable::default(),
            decls: self,
        };
        ctx._visit_value_dataflow(dataflow);
        ctx.reachable
    }
}

impl Visitor for Context<'_> {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Def(_, _, x, _) = expr {
            let stmt = self.decls.defs.get(x).unwrap();
            self.reachable.defs.push(stmt.clone());
        }
        self._visit_expr(expr);
    }

    fn visit_type(&mut self, ty: &Type) {
        if let Type::Cons(x, _) = ty {
            if let Some(stmt) = self.decls.enums.get(x) {
                self.reachable.enums.push(stmt.clone());
            }
            if let Some(stmt) = self.decls.structs.get(x) {
                self.reachable.structs.push(stmt.clone());
            }
        }
        self._visit_type(ty);
    }
}
