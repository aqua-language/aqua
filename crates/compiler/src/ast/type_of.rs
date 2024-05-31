use super::Expr;
use super::Pat;
use super::Type;

impl Expr {
    pub fn type_of(&self) -> &Type {
        match self {
            Expr::Int(_, t, ..) => t,
            Expr::Float(_, t, ..) => t,
            Expr::Bool(_, t, ..) => t,
            Expr::String(_, t, ..) => t,
            Expr::Struct(_, t, ..) => t,
            Expr::Tuple(_, t, ..) => t,
            Expr::Enum(_, t, ..) => t,
            Expr::Var(_, t, ..) => t,
            Expr::Def(_, t, ..) => t,
            Expr::Call(_, t, ..) => t,
            Expr::Block(_, t, ..) => t,
            Expr::Query(_, t, ..) => t,
            Expr::Field(_, t, ..) => t,
            Expr::Assoc(_, t, ..) => t,
            Expr::Err(_, t) => t,
            Expr::Index(_, t, ..) => t,
            Expr::Array(_, t, ..) => t,
            Expr::Assign(_, t, ..) => t,
            Expr::Return(_, t, ..) => t,
            Expr::Continue(_, t) => t,
            Expr::Break(_, t) => t,
            Expr::Fun(_, t, ..) => t,
            Expr::Match(_, t, ..) => t,
            Expr::While(_, t, ..) => t,
            Expr::Record(_, t, _) => t,
            Expr::Path(_, t, _) => t,
            Expr::Value(t, _) => t,
            Expr::For(_, t, _, _, _) => t,
            Expr::Char(_, t, _) => t,
            Expr::Unresolved(_, t, _, _) => t,
        }
    }
}

impl Pat {
    pub fn type_of(&self) -> &Type {
        match self {
            Pat::Path(_, t, _, _) => t,
            Pat::Var(_, t, ..) => t,
            Pat::Tuple(_, t, ..) => t,
            Pat::Struct(_, t, ..) => t,
            Pat::Enum(_, t, ..) => t,
            Pat::Int(_, t, ..) => t,
            Pat::Wildcard(_, t, ..) => t,
            Pat::String(_, t, ..) => t,
            Pat::Bool(_, t, ..) => t,
            Pat::Err(_, t) => t,
            Pat::Record(_, t, _) => t,
            Pat::Or(_, t, _, _) => t,
            Pat::Char(_, t, _) => t,
        }
    }
}
