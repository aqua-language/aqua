mod constructors;
mod downcasts;
mod span_of;
mod type_of;
mod upcasts;
mod with_span;
mod with_type;

use std::rc::Rc;

use crate::builtins::Value;
use crate::interpret::Context;
use crate::lexer::Span;
use crate::symbol::Symbol;

pub use crate::collections::map::Map;

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

// An impl is like a rule
// forall <quantifiers> <head> :- <body>.
// impl<quantifiers> <head> where <body>
// i.e., impl<T> Clone for Vec<T> where T: Clone {}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtImpl {
    pub span: Span,
    pub generics: Vec<Name>,
    pub head: Bound,
    pub where_clause: Vec<Bound>,
    pub defs: Vec<Rc<StmtDef>>,
    pub types: Vec<Rc<StmtType>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ImplKind {
    Trait,
    Type,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTrait {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub where_clause: Vec<Bound>,
    pub defs: Vec<Rc<StmtTraitDef>>,
    pub types: Vec<Rc<StmtTraitType>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Bound {
    Path(Span, Path),
    // Trait bound. For example: impl Clone[i32] { ... }
    Trait(Span, Name, Vec<Type>, Map<Name, Type>),
    // Type bound. For example: impl i32 { ... }
    Type(Span, Rc<Type>),
    Err(Span),
}

// A type is like a proposition
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Path(Path),
    Cons(Name, Vec<Type>),
    Alias(Name, Vec<Type>),
    Assoc(Bound, Name, Vec<Type>),
    Var(Name, TypeVar),
    Generic(Name),
    Fun(Vec<Type>, Rc<Type>),
    Tuple(Vec<Type>),
    Record(Map<Name, Type>),
    Array(Rc<Type>, Option<usize>),
    Never,
    Hole,
    Err,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TypeVar {
    General,
    Float,
    Int,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Candidate {
    // An implementation of a trait for a type.
    Impl(StmtImpl),
    // A bound in a where clause
    Bound(Bound),
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
    pub where_clause: Vec<Bound>,
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
    pub where_clause: Vec<Bound>,
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
    Bool(Span, Type, bool),
    String(Span, Type, Symbol),
    Char(Span, Type, char),
    Struct(Span, Type, Name, Vec<Type>, Map<Name, Expr>),
    Tuple(Span, Type, Vec<Expr>),
    Record(Span, Type, Map<Name, Expr>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Expr>),
    Field(Span, Type, Rc<Expr>, Name),
    Index(Span, Type, Rc<Expr>, Index),
    Var(Span, Type, Name),
    Def(Span, Type, Name, Vec<Type>),
    Call(Span, Type, Rc<Expr>, Vec<Expr>),
    Block(Span, Type, Block),
    Query(Span, Type, Vec<Query>),
    Assoc(Span, Type, Bound, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Vec<Arm>),
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
pub struct Arm {
    pub span: Span,
    pub p: Pat,
    pub e: Expr,
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Query {
    From(Span, Type, Name, Rc<Expr>),
    Where(Span, Type, Rc<Expr>),
    Select(Span, Type, Map<Name, Expr>),
    Join(Span, Type, Name, Rc<Expr>, Rc<Expr>),
    Group(Span, Type, Rc<Expr>, Vec<Query>),
    Over(Span, Type, Rc<Expr>, Vec<Query>),
    Order(Span, Type, Rc<Expr>, bool),
    Var(Span, Type, Name, Rc<Expr>),
    Into(Span, Type, Name, Vec<Type>, Vec<Expr>),
    Compute(Span, Type, Name, Rc<Expr>, Rc<Expr>),
    Err(Span, Type),
}
