use crate::ast::passes::infer::solver::Constraint;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Pat;
use crate::ast::QueryOp;
use crate::ast::Stmt;
use crate::report::span::Span;

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(s, ..) => *s,
            Expr::Float(s, ..) => *s,
            Expr::Bool(s, ..) => *s,
            Expr::String(s, ..) => *s,
            Expr::Struct(s, ..) => *s,
            Expr::Tuple(s, ..) => *s,
            Expr::Enum(s, ..) => *s,
            Expr::Field(s, ..) => *s,
            Expr::Local(s, ..) => *s,
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
            Expr::Lambda(s, ..) => *s,
            Expr::Match(s, ..) => *s,
            Expr::Err(s, ..) => *s,
            Expr::While(s, ..) => *s,
            Expr::Record(s, ..) => *s,
            Expr::Path(s, ..) => *s,
            Expr::For(s, ..) => *s,
            Expr::Char(s, ..) => *s,
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
            Expr::Anonymous(s, ..) => *s,
            Expr::Closure(s, ..) => *s,
            Expr::Ref(s, ..) => *s,
            Expr::Place(s, ..) => *s,
            Expr::Deref(s, ..) => *s,
            Expr::Loop(s, ..) => *s,
            Expr::Unit(s, ..) => *s,
        }
    }
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Path(s, ..) => *s,
            Pat::Local(s, ..) => *s,
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
            Pat::Unit(s, _) => *s,
        }
    }
}

impl QueryOp {
    pub fn span(&self) -> Span {
        match self {
            QueryOp::From(s, ..) => *s,
            QueryOp::Where(s, ..) => *s,
            QueryOp::Union(s, ..) => *s,
            QueryOp::Limit(s, ..) => *s,
            QueryOp::Select(s, ..) => *s,
            QueryOp::JoinOn(s, ..) => *s,
            QueryOp::GroupOverCompute(s, ..) => *s,
            QueryOp::Local(s, ..) => *s,
            QueryOp::OverCompute(s, ..) => *s,
            QueryOp::JoinOverOn(s, ..) => *s,
            QueryOp::Err(s) => *s,
            QueryOp::Drop(s, ..) => *s,
            QueryOp::Distinct(s) => *s,
        }
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Local(s) => s.span,
            Stmt::Def(s) => s.span,
            Stmt::Trait(s) => s.span,
            Stmt::Impl(s) => s.span,
            Stmt::Struct(s) => s.span,
            Stmt::Enum(s) => s.span,
            Stmt::Type(s) => s.span,
            Stmt::Expr(s) => s.span(),
            Stmt::Err(s) => *s,
        }
    }
}

impl Constraint {
    pub fn span(&self) -> &Span {
        match self {
            Constraint::WhereClause(s, _) => s,
            Constraint::AssocDef(s, ..) => s,
            Constraint::AssocType(s, ..) => s,
            Constraint::PlaceElem(s, ..) => s,
        }
    }
}

impl Block {
    pub fn span(&self) -> Span {
        self.span
    }
}
