use crate::lexer::Span;

use super::Expr;
use super::Pat;
use super::Query;

impl Expr {
    pub fn with_span(self, span: Span) -> Expr {
        match self {
            Expr::Path(_, t, p) => Expr::Path(span, t, p),
            Expr::Record(_, t, xes) => Expr::Record(span, t, xes),
            Expr::While(_, t, e0, e1) => Expr::While(span, t, e0, e1),
            Expr::Int(_, t, v) => Expr::Int(span, t, v),
            Expr::Float(_, t, v) => Expr::Float(span, t, v),
            Expr::Bool(_, t, v) => Expr::Bool(span, t, v),
            Expr::String(_, t, v) => Expr::String(span, t, v),
            Expr::Struct(_, t, x, ts, xes) => Expr::Struct(span, t, x, ts, xes),
            Expr::Tuple(_, t, es) => Expr::Tuple(span, t, es),
            Expr::Enum(_, t, x0, ts, x1, e) => Expr::Enum(span, t, x0, ts, x1, e),
            Expr::Var(_, t, x) => Expr::Var(span, t, x),
            Expr::Def(_, t, x, ts) => Expr::Def(span, t, x, ts),
            Expr::Call(_, t, e, es) => Expr::Call(span, t, e, es),
            Expr::Block(_, t, b) => Expr::Block(span, t, b),
            Expr::Query(_, t, qs) => Expr::Query(span, t, qs),
            Expr::QueryInto(_, t, qs, x, ts, es) => Expr::QueryInto(span, t, qs, x, ts, es),
            Expr::Field(_, t, e, x) => Expr::Field(span, t, e, x),
            Expr::Assoc(_, t, b, x1, ts1) => Expr::Assoc(span, t, b, x1, ts1),
            Expr::Index(_, t, e, i) => Expr::Index(span, t, e, i),
            Expr::Array(_, t, es) => Expr::Array(span, t, es),
            Expr::Assign(_, t, e0, e1) => Expr::Assign(span, t, e0, e1),
            Expr::Return(_, t, e) => Expr::Return(span, t, e),
            Expr::Continue(_, t) => Expr::Continue(span, t),
            Expr::Break(_, t) => Expr::Break(span, t),
            Expr::Fun(_, t, ps, t1, e) => Expr::Fun(span, t, ps, t1, e),
            Expr::Match(_, t, e, pes) => Expr::Match(span, t, e, pes),
            Expr::Err(_, t) => Expr::Err(span, t),
            Expr::Value(t, v) => Expr::Value(t, v),
            Expr::For(_, t, x, e, b) => Expr::For(span, t, x, e, b),
            Expr::Char(_, t, v) => Expr::Char(span, t, v),
            Expr::Unresolved(_, t, x, ts) => Expr::Unresolved(span, t, x, ts),
            Expr::InfixBinaryOp(_, t, op, e0, e1) => Expr::InfixBinaryOp(span, t, op, e0, e1),
            Expr::PrefixUnaryOp(_, t, op, e) => Expr::PrefixUnaryOp(span, t, op, e),
            Expr::PostfixUnaryOp(_, t, op, e) => Expr::PostfixUnaryOp(span, t, op, e),
            Expr::Annotate(_, t, e) => Expr::Annotate(span, t, e),
            Expr::Paren(_, t, e) => Expr::Paren(span, t, e),
            Expr::Dot(_, t, e, x, ts, es) => Expr::Dot(span, t, e, x, ts, es),
            Expr::IfElse(_, t, e, b0, b1) => Expr::IfElse(span, t, e, b0, b1),
            Expr::IntSuffix(_, t, v, x) => Expr::IntSuffix(span, t, v, x),
            Expr::FloatSuffix(_, t, v, x) => Expr::FloatSuffix(span, t, v, x),
        }
    }
}

impl Pat {
    pub fn with_span(self, s: Span) -> Pat {
        match self {
            Pat::Path(_, t, p, a) => Pat::Path(s, t, p, a),
            Pat::Var(_, t, x) => Pat::Var(s, t, x),
            Pat::Tuple(_, t, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(_, t, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(_, t, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(_, t, v) => Pat::Int(s, t, v),
            Pat::Wildcard(_, t) => Pat::Wildcard(s, t),
            Pat::String(_, t, v) => Pat::String(s, t, v),
            Pat::Bool(_, t, v) => Pat::Bool(s, t, v),
            Pat::Err(_, t) => Pat::Err(s, t),
            Pat::Record(_, t, xps) => Pat::Record(s, t, xps),
            Pat::Or(_, t, p0, p1) => Pat::Or(s, t, p0, p1),
            Pat::Char(_, t, v) => Pat::Char(s, t, v),
            Pat::Annotate(_, t, p) => Pat::Annotate(s, t, p),
            Pat::Paren(_, t, p) => Pat::Paren(s, t, p),
        }
    }
}

impl Query {
    #[inline(always)]
    pub fn with_span(self, s: Span) -> Query {
        match self {
            Query::From(_, x, e) => Query::From(s, x, e),
            Query::Where(_, e) => Query::Where(s, e),
            Query::Select(_, xes) => Query::Select(s, xes),
            Query::Join(_, x, e0, e1, e2) => Query::Join(s, x, e0, e1, e2),
            Query::GroupOverCompute(_, x, e0, e1, aggs) => Query::GroupOverCompute(s, x, e0, e1, aggs),
            Query::OverCompute(_, e, aggs) => Query::OverCompute(s, e, aggs),
            Query::Var(_, x, e) => Query::Var(s, x, e),
            Query::Err(_) => Query::Err(s),
            Query::JoinOver(_, x, e0, e1, e2, e3) => Query::JoinOver(s, x, e0, e1, e2, e3),
        }
    }
}
