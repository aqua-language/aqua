use crate::report::span::Span;

use crate::ast::Expr;
use crate::ast::Pat;
use crate::ast::QueryOp;

impl Expr {
    pub fn with_span(self, s: Span) -> Expr {
        match self {
            Expr::Path(_, t, p) => Expr::Path(s, t, p),
            Expr::Record(_, t, xes) => Expr::Record(s, t, xes),
            Expr::While(_, t, l, e0, e1) => Expr::While(s, t, l, e0, e1),
            Expr::Int(_, t, v) => Expr::Int(s, t, v),
            Expr::Float(_, t, v) => Expr::Float(s, t, v),
            Expr::Bool(_, t, v) => Expr::Bool(s, t, v),
            Expr::String(_, t, v) => Expr::String(s, t, v),
            Expr::Struct(_, t, x, ts, xes) => Expr::Struct(s, t, x, ts, xes),
            Expr::Tuple(_, t, es) => Expr::Tuple(s, t, es),
            Expr::Enum(_, t, x0, ts, x1, e) => Expr::Enum(s, t, x0, ts, x1, e),
            Expr::Local(_, t, x, m) => Expr::Local(s, t, x, m),
            Expr::Def(_, t, x, ts) => Expr::Def(s, t, x, ts),
            Expr::Call(_, t, e, es) => Expr::Call(s, t, e, es),
            Expr::Block(_, t, b) => Expr::Block(s, t, b),
            Expr::Query(_, t, l, e, qs) => Expr::Query(s, t, l, e, qs),
            Expr::QueryInto(_, t, l, e, qs, x1, ts, es) => {
                Expr::QueryInto(s, t, l, e, qs, x1, ts, es)
            }
            Expr::Field(_, t, e, x) => Expr::Field(s, t, e, x),
            Expr::Assoc(_, t, b, x1, ts1) => Expr::Assoc(s, t, b, x1, ts1),
            Expr::Index(_, t, e, i) => Expr::Index(s, t, e, i),
            Expr::Array(_, t, es) => Expr::Array(s, t, es),
            Expr::Assign(_, t, e0, e1) => Expr::Assign(s, t, e0, e1),
            Expr::Return(_, t, e) => Expr::Return(s, t, e),
            Expr::Continue(_, t, l) => Expr::Continue(s, t, l),
            Expr::Break(_, t, l) => Expr::Break(s, t, l),
            Expr::Lambda(_, t, ps, t1, e) => Expr::Lambda(s, t, ps, t1, e),
            Expr::Match(_, t, e, pes) => Expr::Match(s, t, e, pes),
            Expr::Closure(_, t, uid, xts0, xts1, t1, e) => {
                Expr::Closure(s, t, uid, xts0, xts1, t1, e)
            }
            Expr::For(_, t, l, x, e, b) => Expr::For(s, t, l, x, e, b),
            Expr::Char(_, t, v) => Expr::Char(s, t, v),
            Expr::InfixBinaryOp(_, t, op, e0, e1) => Expr::InfixBinaryOp(s, t, op, e0, e1),
            Expr::PrefixUnaryOp(_, t, op, e) => Expr::PrefixUnaryOp(s, t, op, e),
            Expr::PostfixUnaryOp(_, t, op, e) => Expr::PostfixUnaryOp(s, t, op, e),
            Expr::Annotate(_, t, e) => Expr::Annotate(s, t, e),
            Expr::Paren(_, t, e) => Expr::Paren(s, t, e),
            Expr::Dot(_, t, e, x, ts, es) => Expr::Dot(s, t, e, x, ts, es),
            Expr::IfElse(_, t, e, b0, b1) => Expr::IfElse(s, t, e, b0, b1),
            Expr::IntSuffix(_, t, v, x) => Expr::IntSuffix(s, t, v, x),
            Expr::FloatSuffix(_, t, v, x) => Expr::FloatSuffix(s, t, v, x),
            Expr::Anonymous(_, t) => Expr::Anonymous(s, t),
            Expr::Ref(_, t, e, m) => Expr::Ref(s, t, e, m),
            Expr::Place(_, t, e) => Expr::Place(s, t, e),
            Expr::Deref(_, t, e) => Expr::Deref(s, t, e),
            Expr::Unit(_, t) => Expr::Unit(s, t),
            Expr::Loop(_, t, l, b) => Expr::Loop(s, t, l, b),
            Expr::Err(_, t) => Expr::Err(s, t),
        }
    }
}

impl Pat {
    pub fn with_span(self, s: Span) -> Pat {
        match self {
            Pat::Path(_, t, p, a) => Pat::Path(s, t, p, a),
            Pat::Local(_, t, x, m) => Pat::Local(s, t, x, m),
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
            QueryOp::From(_, l, e) => QueryOp::From(s, l, e),
            QueryOp::Limit(_, e) => QueryOp::Limit(s, e),
            QueryOp::Union(_, e1) => QueryOp::Union(s, e1),
            QueryOp::Where(_, e) => QueryOp::Where(s, e),
            QueryOp::Select(_, xes) => QueryOp::Select(s, xes),
            QueryOp::JoinOn(_, l, e0, e1) => QueryOp::JoinOn(s, l, e0, e1),
            QueryOp::GroupOverCompute(_, x, e0, e1, aggs) => {
                QueryOp::GroupOverCompute(s, x, e0, e1, aggs)
            }
            QueryOp::OverCompute(_, e, aggs) => QueryOp::OverCompute(s, e, aggs),
            QueryOp::Local(_, l, e) => QueryOp::Local(s, l, e),
            QueryOp::Err(_) => QueryOp::Err(s),
            QueryOp::JoinOverOn(_, x, e0, e1, e2) => QueryOp::JoinOverOn(s, x, e0, e1, e2),
            QueryOp::Drop(_, x) => QueryOp::Drop(s, x),
            QueryOp::Distinct(s) => QueryOp::Distinct(s),
        }
    }
}
