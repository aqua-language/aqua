use crate::ast::Expr;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::span::Span;
use crate::traversal::visitor::Visitor;

pub struct Context {
    span: Span,
    output: Option<Output>,
}

pub enum Output {
    Expr(Expr),
    Stmt(Stmt),
}

impl Context {
    pub fn lookup(program: &Program, span: Span) -> Option<Output> {
        let mut this = Self { span, output: None };
        this.visit_program(program);
        this.output
    }
}

impl Visitor for Context {
    fn visit_expr(&mut self, e: &Expr) {
        if self.output.is_none() && e.span_of().contains(&self.span) {
            self._visit_expr(e);
            if self.output.is_none() {
                self.output = Some(Output::Expr(e.clone()));
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        if self.output.is_none() && s.span_of().contains(&self.span) {
            self._visit_stmt(s);
            if self.output.is_none() {
                self.output = Some(Output::Stmt(s.clone()));
            }
        }
    }
}
