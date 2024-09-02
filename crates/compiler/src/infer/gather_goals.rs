use crate::ast::Expr;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::span::Span;
use crate::traversal::visitor::AcceptVisitor;
use crate::traversal::visitor::Visitor;

use super::Context;
use super::Goal;

pub struct GatherGoals<'a> {
    ctx: &'a mut Context,
    span: Span,
}

impl GatherGoals<'_> {
    pub fn new<'a>(ctx: &'a mut Context, span: Span) -> GatherGoals<'a> {
        GatherGoals { ctx, span }
    }
}

impl Visitor for GatherGoals<'_> {
    fn visit_type(&mut self, t: &Type) {
        self._visit_type(t);
        if let Type::Assoc(b, _, _) = t {
            self.ctx.add_goal(Goal::new(self.span, b.clone()));
        }
    }

    fn visit_expr(&mut self, e: &Expr) {
        self.span = e.span_of();
        self._visit_expr(e);
        if let Expr::TraitMethod(s, _, b, _, _) = e {
            self.ctx.add_goal(Goal::new(*s, b.clone()));
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
    pub fn gather_goals(&self, ctx: &mut Context) {
        self.visit(&mut GatherGoals::new(ctx, self.span));
    }
}

impl Type {
    pub fn gather_goals(&self, ctx: &mut Context) {
        self.visit(&mut GatherGoals::new(ctx, Span::default()));
    }
}
