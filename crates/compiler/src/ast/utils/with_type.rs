use crate::ast::Expr;
use crate::ast::Pat;
use crate::ast::Type;

impl Expr {
    pub fn with_type(self, t: Type) -> Expr {
        match self {
            Expr::Int(s, _, v) => Expr::Int(s, t, v),
            Expr::Float(s, _, v) => Expr::Float(s, t, v),
            Expr::Bool(s, _, v) => Expr::Bool(s, t, v),
            Expr::String(s, _, v) => Expr::String(s, t, v),
            Expr::Local(s, _, x, m) => Expr::Local(s, t, x, m),
            Expr::Def(s, _, x, ts) => Expr::Def(s, t, x, ts),
            Expr::Call(s, _, e, es) => Expr::Call(s, t, e, es),
            Expr::Block(s, _, b) => Expr::Block(s, t, b),
            Expr::Query(s, _, l, e, qs) => Expr::Query(s, t, l, e, qs),
            Expr::QueryInto(s, _, l, e, qs, x1, ts, es) => {
                Expr::QueryInto(s, t, l, e, qs, x1, ts, es)
            }
            Expr::Struct(s, _, x, ts, xes) => Expr::Struct(s, t, x, ts, xes),
            Expr::Enum(s, _, x0, ts, x1, e) => Expr::Enum(s, t, x0, ts, x1, e),
            Expr::Field(s, _, e, x) => Expr::Field(s, t, e, x),
            Expr::Tuple(s, _, es) => Expr::Tuple(s, t, es),
            Expr::Assoc(s, _, b, x1, ts1) => Expr::Assoc(s, t, b, x1, ts1),
            Expr::Index(s, _, e, i) => Expr::Index(s, t, e, i),
            Expr::Array(s, _, es) => Expr::Array(s, t, es),
            Expr::Assign(s, _, e0, e1) => Expr::Assign(s, t, e0, e1),
            Expr::Return(s, _, e) => Expr::Return(s, t, e),
            Expr::Continue(s, _, l) => Expr::Continue(s, t, l),
            Expr::Break(s, _, l) => Expr::Break(s, t, l),
            Expr::Lambda(s, _, ps, t1, e) => Expr::Lambda(s, t, ps, t1, e),
            Expr::Match(s, _, e, pes) => Expr::Match(s, t, e, pes),
            Expr::Err(s, _) => Expr::Err(s, t),
            Expr::While(s, _, l, e, b) => Expr::While(s, t, l, e, b),
            Expr::Record(s, _, xes) => Expr::Record(s, t, xes),
            Expr::Path(s, _, p) => Expr::Path(s, t, p),
            Expr::Closure(s, _, uid, xts0, xts1, t1, e) => {
                Expr::Closure(s, t, uid, xts0, xts1, t1, e)
            }
            Expr::For(s, _, l, x, e, b) => Expr::For(s, t, l, x, e, b),
            Expr::Char(s, _, v) => Expr::Char(s, t, v),
            Expr::InfixBinaryOp(s, _, op, e0, e1) => Expr::InfixBinaryOp(s, t, op, e0, e1),
            Expr::PrefixUnaryOp(s, _, op, e) => Expr::PrefixUnaryOp(s, t, op, e),
            Expr::PostfixUnaryOp(s, _, op, e) => Expr::PostfixUnaryOp(s, t, op, e),
            Expr::Annotate(s, _, e) => Expr::Annotate(s, t, e),
            Expr::Paren(s, _, e) => Expr::Paren(s, t, e),
            Expr::Dot(s, _, e, x, ts, es) => Expr::Dot(s, t, e, x, ts, es),
            Expr::IfElse(s, _, e, b0, b1) => Expr::IfElse(s, t, e, b0, b1),
            Expr::IntSuffix(s, _, v, x) => Expr::IntSuffix(s, t, v, x),
            Expr::FloatSuffix(s, _, v, x) => Expr::FloatSuffix(s, t, v, x),
            Expr::Anonymous(s, _) => Expr::Anonymous(s, t),
            Expr::Ref(s, _, e, m) => Expr::Ref(s, t, e, m),
            Expr::Place(s, _, p) => Expr::Place(s, t, p),
            Expr::Deref(s, _, e) => Expr::Deref(s, t, e),
            Expr::Loop(s, _, l, b) => Expr::Loop(s, t, l, b),
            Expr::Unit(s, _) => Expr::Unit(s, t),
        }
    }
}

impl Pat {
    pub fn with_type(self, t: Type) -> Pat {
        match self {
            Pat::Path(s, _, p, a) => Pat::Path(s, t, p, a),
            Pat::Local(s, _, x, m) => Pat::Local(s, t, x, m),
            Pat::Tuple(s, _, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(s, _, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(s, _, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(s, _, v) => Pat::Int(s, t, v),
            Pat::String(s, _, v) => Pat::String(s, t, v),
            Pat::Wildcard(s, _) => Pat::Wildcard(s, t),
            Pat::Bool(s, _, v) => Pat::Bool(s, t, v),
            Pat::Err(s, _) => Pat::Err(s, t),
            Pat::Record(s, _, xps) => Pat::Record(s, t, xps),
            Pat::Or(s, _, p0, p1) => Pat::Or(s, t, p0, p1),
            Pat::Char(s, _, v) => Pat::Char(s, t, v),
            Pat::Annotate(s, _, p) => Pat::Annotate(s, t, p),
            Pat::Paren(s, _, p) => Pat::Paren(s, t, p),
            Pat::Unit(s, _) => Pat::Unit(s, t),
        }
    }
}
