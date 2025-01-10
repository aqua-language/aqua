use crate::syntax::span::Span;

use super::Expr;
use super::Pat;
use super::QueryOp;

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
            Expr::Local(_, t, x, m) => Expr::Local(span, t, x, m),
            Expr::Def(_, t, x, ts) => Expr::Def(span, t, x, ts),
            Expr::Call(_, t, e, es) => Expr::Call(span, t, e, es),
            Expr::Block(_, t, b) => Expr::Block(span, t, b),
            Expr::Query(_, t, x, t0, e, qs) => Expr::Query(span, t, x, t0, e, qs),
            Expr::QueryInto(_, t, x0, t0, e, qs, x1, ts, es) => {
                Expr::QueryInto(span, t, x0, t0, e, qs, x1, ts, es)
            }
            Expr::Field(_, t, e, x) => Expr::Field(span, t, e, x),
            Expr::Assoc(_, t, b, x1, ts1) => Expr::Assoc(span, t, b, x1, ts1),
            Expr::Index(_, t, e, i) => Expr::Index(span, t, e, i),
            Expr::Array(_, t, es) => Expr::Array(span, t, es),
            Expr::Assign(_, t, e0, e1) => Expr::Assign(span, t, e0, e1),
            Expr::Return(_, t, e) => Expr::Return(span, t, e),
            Expr::Continue(_, t) => Expr::Continue(span, t),
            Expr::Break(_, t) => Expr::Break(span, t),
            Expr::Lambda(_, t, ps, t1, e) => Expr::Lambda(span, t, ps, t1, e),
            Expr::Match(_, t, e, pes) => Expr::Match(span, t, e, pes),
            Expr::Closure(_, t, uid, xts0, xts1, t1, e) => Expr::Closure(span, t, uid, xts0, xts1, t1, e),
            Expr::For(_, t, x, e, b) => Expr::For(span, t, x, e, b),
            Expr::Char(_, t, v) => Expr::Char(span, t, v),
            Expr::InfixBinaryOp(_, t, op, e0, e1) => Expr::InfixBinaryOp(span, t, op, e0, e1),
            Expr::PrefixUnaryOp(_, t, op, e) => Expr::PrefixUnaryOp(span, t, op, e),
            Expr::PostfixUnaryOp(_, t, op, e) => Expr::PostfixUnaryOp(span, t, op, e),
            Expr::Annotate(_, t, e) => Expr::Annotate(span, t, e),
            Expr::Paren(_, t, e) => Expr::Paren(span, t, e),
            Expr::Dot(_, t, e, x, ts, es) => Expr::Dot(span, t, e, x, ts, es),
            Expr::IfElse(_, t, e, b0, b1) => Expr::IfElse(span, t, e, b0, b1),
            Expr::IntSuffix(_, t, v, x) => Expr::IntSuffix(span, t, v, x),
            Expr::FloatSuffix(_, t, v, x) => Expr::FloatSuffix(span, t, v, x),
            Expr::Anonymous(_, t) => Expr::Anonymous(span, t),
            Expr::Ref(_, t, e, m) => Expr::Ref(span, t, e, m),
            Expr::Place(_, t, e) => Expr::Place(span, t, e),
            Expr::Deref(_, t, e) => Expr::Deref(span, t, e),
            Expr::Unit(_, t) => Expr::Unit(span, t),
            Expr::Loop(_, t, b) => Expr::Loop(span, t, b),
            Expr::Err(_, t) => Expr::Err(span, t),
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
            Pat::Unit(_, t) => Pat::Unit(s, t),
        }
    }
}

impl QueryOp {
    #[inline(always)]
    pub fn with_span(self, s: Span) -> QueryOp {
        match self {
            QueryOp::From(_, x, t, e) => QueryOp::From(s, x, t, e),
            QueryOp::Limit(_, e) => QueryOp::Limit(s, e),
            QueryOp::Union(_, e1) => QueryOp::Union(s, e1),
            QueryOp::Where(_, e) => QueryOp::Where(s, e),
            QueryOp::Select(_, xes) => QueryOp::Select(s, xes),
            QueryOp::JoinOn(_, x, t, e0, e1) => QueryOp::JoinOn(s, x, t, e0, e1),
            QueryOp::GroupOverCompute(_, x, e0, e1, aggs) => {
                QueryOp::GroupOverCompute(s, x, e0, e1, aggs)
            }
            QueryOp::OverCompute(_, e, aggs) => QueryOp::OverCompute(s, e, aggs),
            QueryOp::Var(_, x, t, e) => QueryOp::Var(s, x, t, e),
            QueryOp::Err(_) => QueryOp::Err(s),
            QueryOp::JoinOverOn(_, x, e0, e1, e2) => QueryOp::JoinOverOn(s, x, e0, e1, e2),
            QueryOp::Drop(_, x) => QueryOp::Drop(s, x),
        }
    }
}
