use crate::ast::Expr;
use crate::ast::Type;
use crate::builtins::value::Dataflow;
use crate::declare;
use crate::traversal::visitor::Visitor;

pub struct Context<'a> {
    pub reachable: declare::Context,
    pub decls: &'a declare::Context,
}

impl declare::Context {
    pub fn reachable(&self, dataflow: &Dataflow) -> Self {
        let mut ctx = Context {
            reachable: declare::Context::default(),
            decls: self,
        };
        ctx._visit_value_dataflow(dataflow);
        ctx.reachable
    }
}

impl Visitor for Context<'_> {
    fn visit_expr(&mut self, expr: &Expr) {
        if let Expr::Def(_, _, x, _) = expr {
            if self.reachable.defs.contains_key(x) {
                return;
            } else {
                let stmt = self.decls.defs.get(x).unwrap();
                self.reachable.defs.insert(*x, stmt.clone());
                self.visit_stmt_def(stmt);
            }
        }
        self._visit_expr(expr);
    }

    fn visit_type(&mut self, ty: &Type) {
        if let Type::Cons(x, _) = ty {
            if let Some(stmt) = self.decls.enums.get(x) {
                if self.reachable.enums.contains_key(x) {
                    return;
                } else {
                    self.reachable.enums.insert(*x, stmt.clone());
                    self.visit_stmt_enum(stmt);
                }
            }
            if let Some(stmt) = self.decls.structs.get(x) {
                if self.reachable.structs.contains_key(x) {
                    return;
                } else {
                    self.reachable.structs.insert(*x, stmt.clone());
                    self.visit_stmt_struct(stmt);
                }
            }
        }
        self._visit_type(ty);
    }
}
