mod constructors;
mod downcasts;
pub mod into_egg;
mod span_of;
mod type_of;
mod upcasts;
mod utils;
mod with_span;
mod with_type;

use std::rc::Rc;

use crate::builtins::Value;
use crate::interpret::Context;
use crate::lexer::Token;
use crate::span::Span;
use crate::symbol::Symbol;

pub use crate::collections::map::Map;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Program {
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IR {
    pub defs: Map<Name, StmtDef>,
    pub tys: Map<Name, StmtType>,
    pub traits: Map<Name, StmtTrait>,
    pub impls: Map<Name, StmtImpl>,
    pub structs: Map<Name, StmtStruct>,
    pub enums: Map<Name, StmtEnum>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Path {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Segment {
    pub span: Span,
    pub name: Name,
    pub ts: Vec<Type>,
    pub xts: Map<Name, Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Name {
    pub span: Span,
    pub data: Symbol,
}

// An impl is like a rule
// forall <quantifiers> <head> :- <body>.
// impl<quantifiers> <head> where <body>
// i.e., impl<T> Clone for Vec<T> where T: Clone {}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtImpl {
    pub span: Span,
    pub generics: Vec<Name>,
    pub head: Trait,
    pub where_clause: Vec<Trait>,
    pub defs: Vec<Rc<StmtDef>>,
    pub types: Vec<Rc<StmtType>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTrait {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub where_clause: Vec<Trait>,
    pub defs: Vec<Rc<StmtTraitDef>>,
    pub types: Vec<Rc<StmtTraitType>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Trait {
    Path(Span, Path),
    // Trait bound. For example: impl Clone[i32] { ... } and Clone[T]::clone();
    Cons(Name, Vec<Type>, Map<Name, Type>),
    Var(TraitVar),
    // Type bound. For example: impl[T] Vec[T] { ... } and Vec[T]::new();
    Type(Rc<Type>),
    Err,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TraitVar(u32);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Path(Path),
    Cons(Name, Vec<Type>),
    Alias(Name, Vec<Type>),
    Assoc(Trait, Name, Vec<Type>),
    Var(TypeVar),
    Generic(Name),
    Fun(Vec<Type>, Rc<Type>),
    Tuple(Vec<Type>),
    Record(Map<Name, Type>),
    Array(Rc<Type>, Option<usize>),
    Never,
    Paren(Rc<Type>),
    Err,
    // A placeholder for a type that has not been annotated yet.
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Candidate {
    // An implementation of a trait for a type.
    Impl(StmtImpl),
    // A bound in a where clause
    Bound(Trait),
}

impl std::fmt::Display for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Candidate::Impl(stmt) => write!(f, "{}", stmt),
            Candidate::Bound(bound) => write!(f, "{}", bound),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stmt {
    Var(Rc<StmtVar>),
    Def(Rc<StmtDef>),
    Trait(Rc<StmtTrait>),
    Impl(Rc<StmtImpl>),
    Struct(Rc<StmtStruct>),
    Enum(Rc<StmtEnum>),
    Type(Rc<StmtType>),
    Expr(Rc<Expr>),
    Err(Span),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTraitType {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTraitDef {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Map<Name, Type>,
    pub ty: Type,
    pub where_clause: Vec<Trait>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtVar {
    pub span: Span,
    pub name: Name,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtDef {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Map<Name, Type>,
    pub ty: Type,
    pub where_clause: Vec<Trait>,
    pub body: StmtDefBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtStruct {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub fields: Map<Name, Type>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtEnum {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub variants: Map<Name, Type>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtType {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub body: StmtTypeBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StmtDefBody {
    UserDefined(Expr),
    Builtin(BuiltinDef),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StmtTypeBody {
    UserDefined(Type),
    Builtin(BuiltinType),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuiltinDef {
    // pub codegen: fn(&mut Formatter, &[Value]) -> Value,
    pub fun: fn(&mut Context, &[Type], &[Value]) -> Value,
    pub rust: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuiltinType {
    pub rust: &'static str,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Index {
    pub span: Span,
    pub data: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Path(Span, Type, Path),
    Unresolved(Span, Type, Name, Vec<Type>),
    Int(Span, Type, Symbol),
    Float(Span, Type, Symbol),
    IntSuffix(Span, Type, Symbol, Symbol),
    FloatSuffix(Span, Type, Symbol, Symbol),
    Bool(Span, Type, bool),
    String(Span, Type, Symbol),
    Char(Span, Type, char),
    Struct(Span, Type, Name, Vec<Type>, Map<Name, Expr>),
    Tuple(Span, Type, Vec<Expr>),
    Record(Span, Type, Map<Name, Expr>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Expr>),
    Field(Span, Type, Rc<Expr>, Name),
    Update(Span, Type, Rc<Expr>, Name, Rc<Expr>),
    Index(Span, Type, Rc<Expr>, Index),
    Var(Span, Type, Name),
    Def(Span, Type, Name, Vec<Type>),
    Call(Span, Type, Rc<Expr>, Vec<Expr>),
    Block(Span, Type, Block),
    Query(Span, Type, Name, Rc<Expr>, Vec<Query>),
    QueryInto(
        Span,
        Type,
        Name,
        Rc<Expr>,
        Vec<Query>,
        Name,
        Vec<Type>,
        Vec<Expr>,
    ),
    TraitMethod(Span, Type, Trait, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Map<Pat, Expr>),
    IfElse(Span, Type, Rc<Expr>, Block, Block),
    Array(Span, Type, Vec<Expr>),
    Assign(Span, Type, Rc<Expr>, Rc<Expr>),
    Return(Span, Type, Rc<Expr>),
    Continue(Span, Type),
    Break(Span, Type),
    While(Span, Type, Rc<Expr>, Block),
    Fun(Span, Type, Map<Name, Type>, Type, Rc<Expr>),
    For(Span, Type, Name, Rc<Expr>, Block),
    Err(Span, Type),
    Value(Type, Value),
    InfixBinaryOp(Span, Type, Token, Rc<Expr>, Rc<Expr>),
    PrefixUnaryOp(Span, Type, Token, Rc<Expr>),
    PostfixUnaryOp(Span, Type, Token, Rc<Expr>),
    Annotate(Span, Type, Rc<Expr>),
    Paren(Span, Type, Rc<Expr>),
    Dot(Span, Type, Rc<Expr>, Name, Vec<Type>, Vec<Expr>),
    LetIn(Span, Type, Name, Type, Rc<Expr>, Rc<Expr>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub expr: Rc<Expr>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathPatField {
    Named(Name, Pat),
    // Could be a punned field or a positional field
    Unnamed(Pat),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Pat {
    Path(Span, Type, Path, Option<Vec<PathPatField>>),
    Var(Span, Type, Name),
    Tuple(Span, Type, Vec<Pat>),
    Struct(Span, Type, Name, Vec<Type>, Map<Name, Pat>),
    Record(Span, Type, Map<Name, Pat>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Pat>),
    Int(Span, Type, Symbol),
    String(Span, Type, Symbol),
    Char(Span, Type, char),
    Bool(Span, Type, bool),
    Wildcard(Span, Type),
    Or(Span, Type, Rc<Pat>, Rc<Pat>),
    Err(Span, Type),
    Annotate(Span, Type, Rc<Pat>),
    Paren(Span, Type, Rc<Pat>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Query {
    From(Span, Name, Rc<Expr>),
    Var(Span, Name, Rc<Expr>),
    Where(Span, Rc<Expr>),
    Select(Span, Map<Name, Expr>),
    OverCompute(Span, Rc<Expr>, Vec<Aggr>),
    GroupOverCompute(Span, Name, Rc<Expr>, Rc<Expr>, Vec<Aggr>),
    JoinOn(Span, Name, Rc<Expr>, Rc<Expr>),
    JoinOverOn(Span, Name, Rc<Expr>, Rc<Expr>, Rc<Expr>),
    // Into(Span, Name, Vec<Type>, Vec<Expr>),
    // Compute(Span, Name, Rc<Expr>, Rc<Expr>),
    Err(Span),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Aggr {
    pub x: Name,
    pub e0: Rc<Expr>,
    pub e1: Rc<Expr>,
}
