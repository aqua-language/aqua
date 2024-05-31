use crate::lexer::Span;

use super::Expr;
use super::Pat;
use super::Query;

impl Expr {
    pub fn span_of(&self) -> Span {
        match self {
            Expr::Int(s, ..) => *s,
            Expr::Float(s, ..) => *s,
            Expr::Bool(s, ..) => *s,
            Expr::String(s, ..) => *s,
            Expr::Struct(s, ..) => *s,
            Expr::Tuple(s, ..) => *s,
            Expr::Enum(s, ..) => *s,
            Expr::Field(s, ..) => *s,
            Expr::Var(s, ..) => *s,
            Expr::Def(s, ..) => *s,
            Expr::Call(s, ..) => *s,
            Expr::Block(s, ..) => *s,
            Expr::Query(s, ..) => *s,
            Expr::Assoc(s, ..) => *s,
            Expr::Index(s, ..) => *s,
            Expr::Array(s, ..) => *s,
            Expr::Assign(s, ..) => *s,
            Expr::Return(s, ..) => *s,
            Expr::Continue(s, ..) => *s,
            Expr::Break(s, ..) => *s,
            Expr::Fun(s, ..) => *s,
            Expr::Match(s, ..) => *s,
            Expr::Err(s, ..) => *s,
            Expr::While(s, ..) => *s,
            Expr::Record(s, _, _) => *s,
            Expr::Path(s, _, _) => *s,
            Expr::Value(_, _) => unreachable!(),
            Expr::For(s, _, _, _, _) => *s,
            Expr::Char(s, _, _) => *s,
            Expr::Unresolved(s, _, _, _) => *s,
        }
    }
}

impl Pat {
    pub fn span_of(&self) -> Span {
        match self {
            Pat::Path(s, _, _, _) => *s,
            Pat::Var(s, ..) => *s,
            Pat::Tuple(s, ..) => *s,
            Pat::Struct(s, ..) => *s,
            Pat::Enum(s, ..) => *s,
            Pat::Int(s, ..) => *s,
            Pat::Wildcard(s, ..) => *s,
            Pat::String(s, ..) => *s,
            Pat::Bool(s, ..) => *s,
            Pat::Err(s, _) => *s,
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
        }
    }
}

impl Query {
    pub fn span_of(&self) -> Span {
        match self {
            Query::From(s, ..) => *s,
            Query::Where(s, ..) => *s,
            Query::Select(s, ..) => *s,
            Query::Into(s, ..) => *s,
            Query::Join(s, ..) => *s,
            Query::Group(s, ..) => *s,
            Query::Over(s, ..) => *s,
            Query::Order(s, ..) => *s,
            Query::Var(s, ..) => *s,
            Query::Compute(s, ..) => *s,
            Query::Err(s, _) => *s,
        }
    }
}
