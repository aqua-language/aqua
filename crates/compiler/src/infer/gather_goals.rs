use crate::ast::Expr;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::span::Span;
use crate::traversal::visitor::AcceptVisitor;
use crate::traversal::visitor::Visitor;

use super::Constraint;
use super::Context;

struct GatherConstraints<'a> {
    ctx: &'a mut Context,
    span: Span,
}

impl GatherConstraints<'_> {
    pub fn new<'a>(ctx: &'a mut Context, span: Span) -> GatherConstraints<'a> {
        GatherConstraints { ctx, span }
    }
}

impl Visitor for GatherConstraints<'_> {
    fn visit_type(&mut self, t: &Type) {
        self._visit_type(t);
        if let Type::Assoc(i, x, ts) = t {
            self.ctx.add_constraint(Constraint::TypeAssoc(
                self.span,
                t.clone(),
                i.clone(),
                *x,
                ts.clone(),
            ))
        }
    }

    fn visit_expr(&mut self, e: &Expr) {
        self.span = e.span_of();
        self._visit_expr(e);
        if let Expr::Assoc(s, t, i, x, ts) = e {
            self.ctx.add_constraint(Constraint::ExprAssoc(
                *s,
                t.clone(),
                i.clone(),
                *x,
                ts.clone(),
            ))
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        self.span = s.span_of();
        match s {
            Stmt::Var(s) => self.visit_stmt_var(s),
            Stmt::Expr(s) => self.visit_expr(s),
            _ => {}
        }
    }
}

impl Program {
    pub fn gather_constraints(&self, ctx: &mut Context) {
        self.visit(&mut GatherConstraints::new(ctx, self.span));
    }
}

impl Type {
    pub fn gather_constraints(&self, ctx: &mut Context) {
        self.visit(&mut GatherConstraints::new(ctx, Span::default()));
    }
}
