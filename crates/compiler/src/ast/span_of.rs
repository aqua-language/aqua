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
            Expr::Record(s, ..) => *s,
            Expr::Path(s, ..) => *s,
            Expr::Value(..) => unreachable!(),
            Expr::For(s, ..) => *s,
            Expr::Char(s, ..) => *s,
            Expr::Unresolved(s, ..) => *s,
            Expr::QueryInto(s, ..) => *s,
            Expr::InfixBinaryOp(s, ..) => *s,
            Expr::PrefixUnaryOp(s, ..) => *s,
            Expr::PostfixUnaryOp(s, ..) => *s,
            Expr::Annotate(s, ..) => *s,
            Expr::Paren(s, ..) => *s,
            Expr::Dot(s, ..) => *s,
            Expr::IfElse(s, ..) => *s,
            Expr::IntSuffix(s, ..) => *s,
            Expr::FloatSuffix(s, ..) => *s,
        }
    }
}

impl Pat {
    pub fn span_of(&self) -> Span {
        match self {
            Pat::Path(s, ..) => *s,
            Pat::Var(s, ..) => *s,
            Pat::Tuple(s, ..) => *s,
            Pat::Struct(s, ..) => *s,
            Pat::Enum(s, ..) => *s,
            Pat::Int(s, ..) => *s,
            Pat::Wildcard(s, ..) => *s,
            Pat::String(s, ..) => *s,
            Pat::Bool(s, ..) => *s,
            Pat::Err(s, ..) => *s,
            Pat::Record(s, ..) => *s,
            Pat::Or(s, ..) => *s,
            Pat::Char(s, ..) => *s,
            Pat::Annotate(s, ..) => *s,
            Pat::Paren(s, ..) => *s,
        }
    }
}

impl Query {
    pub fn span_of(&self) -> Span {
        match self {
            Query::From(s, ..) => *s,
            Query::Where(s, ..) => *s,
            Query::Select(s, ..) => *s,
            Query::Join(s, ..) => *s,
            Query::GroupOverCompute(s, ..) => *s,
            Query::Var(s, ..) => *s,
            Query::OverCompute(s, ..) => *s,
            Query::JoinOver(s, ..) => *s,
            Query::Err(s) => *s,
        }
    }
}