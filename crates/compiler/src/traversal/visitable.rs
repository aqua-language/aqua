use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::pass::infer::solver::Constraint;

use super::visitor::Visitor;

pub(crate) trait Visitable {
    fn visit(&self, visitor: &mut impl Visitor);
}

impl Visitable for Ast {
    fn visit(&self, visitor: &mut impl Visitor) {
        visitor.visit_program(self);
    }
}

impl Visitable for Stmt {
    fn visit(&self, visitor: &mut impl Visitor) {
        visitor.visit_stmt(self);
    }
}

impl Visitable for Expr {
    fn visit(&self, visitor: &mut impl Visitor) {
        visitor.visit_expr(self);
    }
}

impl Visitable for Type {
    fn visit(&self, visitor: &mut impl Visitor) {
        visitor.visit_type(self);
    }
}

impl Visitable for Constraint {
    fn visit(&self, visitor: &mut impl Visitor) {
        visitor.visit_constraint(self);
    }
}
