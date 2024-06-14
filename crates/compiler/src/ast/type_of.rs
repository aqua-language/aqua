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
            Expr::QueryInto(_, t, ..) => t,
            Expr::Field(_, t, ..) => t,
            Expr::TraitMethod(_, t, ..) => t,
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
            Expr::Record(_, t, ..) => t,
            Expr::Path(_, t, ..) => t,
            Expr::Value(t, ..) => t,
            Expr::For(_, t, ..) => t,
            Expr::Char(_, t, ..) => t,
            Expr::Unresolved(_, t, ..) => t,
            Expr::InfixBinaryOp(_, t, ..) => t,
            Expr::PrefixUnaryOp(_, t, ..) => t,
            Expr::PostfixUnaryOp(_, t, ..) => t,
            Expr::Annotate(_, t, ..) => t,
            Expr::Paren(_, t, ..) => t,
            Expr::Dot(_, t, ..) => t,
            Expr::IfElse(_, t, ..) => t,
            Expr::IntSuffix(_, t, ..) => t,
            Expr::FloatSuffix(_, t, ..) => t,
            Expr::LetIn(_, t, ..) => t,
            Expr::Update(_, t, ..) => t,
        }
    }
}

impl Pat {
    pub fn type_of(&self) -> &Type {
        match self {
            Pat::Path(_, t, ..) => t,
            Pat::Var(_, t, ..) => t,
            Pat::Tuple(_, t, ..) => t,
            Pat::Struct(_, t, ..) => t,
            Pat::Enum(_, t, ..) => t,
            Pat::Int(_, t, ..) => t,
            Pat::Wildcard(_, t, ..) => t,
            Pat::String(_, t, ..) => t,
            Pat::Bool(_, t, ..) => t,
            Pat::Err(_, t) => t,
            Pat::Record(_, t, ..) => t,
            Pat::Or(_, t, ..) => t,
            Pat::Char(_, t, ..) => t,
            Pat::Annotate(_, t, ..) => t,
            Pat::Paren(_, t, ..) => t,
        }
    }
}
