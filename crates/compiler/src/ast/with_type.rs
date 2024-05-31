use super::Expr;
use super::Pat;
use super::Type;

impl Expr {
    #[inline(always)]
    pub fn with_ty(self, t: Type) -> Expr {
        match self {
            Expr::Int(s, _, v) => Expr::Int(s, t, v),
            Expr::Float(s, _, v) => Expr::Float(s, t, v),
            Expr::Bool(s, _, v) => Expr::Bool(s, t, v),
            Expr::String(s, _, v) => Expr::String(s, t, v),
            Expr::Var(s, _, x) => Expr::Var(s, t, x),
            Expr::Def(s, _, x, ts) => Expr::Def(s, t, x, ts),
            Expr::Call(s, _, e, es) => Expr::Call(s, t, e, es),
            Expr::Block(s, _, b) => Expr::Block(s, t, b),
            Expr::Query(s, _, qs) => Expr::Query(s, t, qs),
            Expr::Struct(s, _, x, ts, xes) => Expr::Struct(s, t, x, ts, xes),
            Expr::Enum(s, _, x0, ts, x1, e) => Expr::Enum(s, t, x0, ts, x1, e),
            Expr::Field(s, _, e, x) => Expr::Field(s, t, e, x),
            Expr::Tuple(s, _, es) => Expr::Tuple(s, t, es),
            Expr::Assoc(s, _, b, x1, ts1) => Expr::Assoc(s, t, b, x1, ts1),
            Expr::Index(s, _, e, i) => Expr::Index(s, t, e, i),
            Expr::Array(s, _, es) => Expr::Array(s, t, es),
            Expr::Assign(s, _, e0, e1) => Expr::Assign(s, t, e0, e1),
            Expr::Return(s, _, e) => Expr::Return(s, t, e),
            Expr::Continue(s, _) => Expr::Continue(s, t),
            Expr::Break(s, _) => Expr::Break(s, t),
            Expr::Fun(s, _, ps, t1, e) => Expr::Fun(s, t, ps, t1, e),
            Expr::Match(s, _, e, pes) => Expr::Match(s, t, e, pes),
            Expr::Err(s, _) => Expr::Err(s, t),
            Expr::While(s, _, e0, e1) => Expr::While(s, t, e0, e1),
            Expr::Record(s, _, xes) => Expr::Record(s, t, xes),
            Expr::Path(s, _, p) => Expr::Path(s, t, p),
            Expr::Value(_, v) => Expr::Value(t, v),
            Expr::For(s, _, x, e, b) => Expr::For(s, t, x, e, b),
            Expr::Char(s, _, v) => Expr::Char(s, t, v),
            Expr::Unresolved(s, _, x, ts) => Expr::Unresolved(s, t, x, ts),
        }
    }
}

impl Pat {
    #[inline(always)]
    pub fn with_ty(self, t: Type) -> Pat {
        match self {
            Pat::Path(s, _, p, a) => Pat::Path(s, t, p, a),
            Pat::Var(s, _, x) => Pat::Var(s, t, x),
            Pat::Tuple(s, _, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(s, _, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(s, _, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(s, _, v) => Pat::Int(s, t, v),
            Pat::String(s, _, v) => Pat::String(s, t, v),
            Pat::Wildcard(s, _) => Pat::Wildcard(s, t),
            Pat::Bool(s, _, v) => Pat::Bool(s, t, v),
            Pat::Err(s, _) => Pat::Err(s, t),
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
        }
    }
}
