mod constructors;
mod downcasts;
mod get_span;
mod get_type;
pub mod key;
mod upcasts;
mod utils;
mod with_span;
mod with_type;

use std::rc::Rc;

use runtime::prelude::Send;
use runtime::prelude::Sync;

use crate::builtins::value::Value;

use crate::collections::keyvec::KeyVec;
pub use crate::collections::map::Map;
use crate::interpret::Context;
use crate::syntax::span::Span;
use crate::syntax::symbol::Symbol;
use crate::syntax::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IR {
    pub defs: KeyVec<StmtDef>,
    pub tys: KeyVec<StmtType>,
    pub traits: KeyVec<StmtTrait>,
    pub impls: KeyVec<StmtImpl>,
    pub structs: KeyVec<StmtStruct>,
    pub enums: KeyVec<StmtEnum>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Path {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Segment {
    pub span: Span,
    pub x: Name,
    pub ts: Vec<Type>,
    pub xts: Map<Name, Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Name {
    pub span: Span,
    pub data: Symbol,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtImpl {
    pub span: Span,
    pub generics: Vec<Name>,
    pub head: Impl,
    pub where_clause: Vec<Impl>,
    pub defs: Vec<Rc<StmtDef>>,
    pub types: Vec<Rc<StmtType>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTrait {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub where_clause: Vec<Impl>,
    pub defs: Vec<Rc<StmtTraitDef>>,
    pub types: Vec<Rc<StmtTraitType>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Impl {
    Path(Span, Path),
    Trait(Trait),
    Var(ImplVar),
    Type(Rc<Type>),
    Unknown, // A placeholder for an impl that has not been annotated yet.
    Err,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ImplVar(pub u32);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Trait {
    pub x: Name,
    pub ts: Vec<Type>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Send, Sync)]
pub enum Type {
    Path(Path),
    Builtin(Name, Vec<Type>),
    Alias(Name, Vec<Type>),
    Struct(Name, Vec<Type>),
    Enum(Name, Vec<Type>),
    Assoc(Impl, Name, Vec<Type>),
    Var(TypeVar),
    Generic(Name),
    Function(Vec<Type>, Rc<Type>),
    Tuple(Vec<Type>),
    Record(Map<Name, Type>),
    Array(Rc<Type>, Option<usize>),
    Never,
    Unit,
    Paren(Rc<Type>),
    Ref(Vec<Loan>, Rc<Type>),
    RefMut(Vec<Loan>, Rc<Type>),
    Err,
    Unknown, // A placeholder for a type that has not been annotated yet.
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Loan {
    pub place: Place,
    pub mutable: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

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
    pub params: KeyVec<Local>,
    pub ty: Type,
    pub where_clause: Vec<Impl>,
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
    pub params: KeyVec<Local>,
    pub ty: Type,
    pub where_clause: Vec<Impl>,
    pub body: ExprBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtDefBuiltin {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: KeyVec<Local>,
    pub ty: Type,
    pub where_clause: Vec<Impl>,
    pub fun: fn(&mut Context, &[Value]) -> Value,
    pub codegen: Option<Codegen>,
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
    pub body: TypeBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExprBody {
    UserDefined(Rc<Expr>),
    Builtin(BuiltinDef),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeBody {
    UserDefined(Type),
    Builtin(BuiltinType),
}

#[derive(Debug, Clone, Eq)]
pub struct BuiltinDef {
    pub fun: fn(&mut Context, &[Value]) -> Value,
    pub codegen: Option<Codegen>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Copy)]
pub struct Codegen {
    pub rust: &'static str,
    pub java: &'static str,
    pub egglog: Option<Egglog>,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct Egglog {
    pub name: &'static str,
    pub code: for<'a> fn(
        &str,
        &'a mut std::fmt::Formatter<'a>,
    ) -> std::result::Result<(), std::fmt::Error>,
}

impl PartialEq for BuiltinDef {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuiltinType {
    pub codegen: Option<Codegen>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Index {
    pub span: Span,
    pub data: usize,
}

impl From<usize> for Index {
    fn from(data: usize) -> Self {
        Index {
            span: Span::Generated,
            data,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Place {
    pub local: Local,
    pub elems: Vec<PlaceElem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Local {
    pub name: Name,
    pub ty: Type,
    pub mutable: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PlaceElem {
    Index(usize),
    Deref,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Path(Span, Type, Path),
    Ref(Span, Type, Rc<Expr>),
    RefMut(Span, Type, Rc<Expr>),
    Place(Span, Type, Place),
    Deref(Span, Type, Rc<Expr>),
    Int(Span, Type, Symbol),
    Float(Span, Type, Symbol),
    Unit(Span, Type),
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
    Block(Span, Type, Rc<Block>),
    Closure(Span, Type, Map<Name, Type>, Map<Name, Type>, Type, Rc<Expr>),
    Query(Span, Type, Name, Type, Rc<Expr>, Vec<QueryOp>),
    QueryInto(
        Span,
        Type,
        Name,
        Type,
        Rc<Expr>,
        Vec<QueryOp>,
        Name,
        Vec<Type>,
        Vec<Expr>,
    ),
    Assoc(Span, Type, Impl, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Map<Pat, Expr>),
    IfElse(Span, Type, Rc<Expr>, Rc<Expr>, Rc<Expr>),
    Array(Span, Type, Vec<Expr>),
    Assign(Span, Type, Rc<Expr>, Rc<Expr>),
    Return(Span, Type, Rc<Expr>),
    Continue(Span, Type),
    Break(Span, Type),
    While(Span, Type, Rc<Expr>, Rc<Expr>),
    Lambda(Span, Type, Map<Name, Type>, Type, Rc<Expr>),
    For(Span, Type, Name, Rc<Expr>, Rc<Expr>),
    Err(Span, Type),
    InfixBinaryOp(Span, Type, Token, Rc<Expr>, Rc<Expr>),
    PrefixUnaryOp(Span, Type, Token, Rc<Expr>),
    PostfixUnaryOp(Span, Type, Token, Rc<Expr>),
    Annotate(Span, Type, Rc<Expr>),
    Paren(Span, Type, Rc<Expr>),
    Dot(Span, Type, Rc<Expr>, Name, Vec<Type>, Vec<Expr>),
    LetIn(Span, Type, Name, Type, Rc<Expr>, Rc<Expr>),
    Anonymous(Span, Type),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub expr: Option<Expr>,
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
    Unit(Span, Type),
    Wildcard(Span, Type),
    Or(Span, Type, Rc<Pat>, Rc<Pat>),
    Err(Span, Type),
    Annotate(Span, Type, Rc<Pat>),
    Paren(Span, Type, Rc<Pat>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryOp {
    From(Span, Name, Type, Rc<Expr>),
    Var(Span, Name, Type, Rc<Expr>),
    Drop(Span, Name),
    Union(Span, Rc<Expr>),
    Where(Span, Rc<Expr>),
    Select(Span, Map<Name, Expr>),
    Limit(Span, Rc<Expr>),
    OverCompute(Span, Rc<Expr>, Vec<Aggr>),
    GroupOverCompute(Span, Name, Rc<Expr>, Rc<Expr>, Vec<Aggr>),
    JoinOn(Span, Name, Type, Rc<Expr>, Rc<Expr>),
    JoinOverOn(Span, Name, Rc<Expr>, Rc<Expr>, Rc<Expr>),
    // Compute(Span, Name, Rc<Expr>, Rc<Expr>),
    Err(Span),
}

/// An aggregation function.
/// x0 = x1 of e [if e2]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Aggr {
    pub x0: Name,
    pub x1: Name,
    pub e1: Rc<Expr>,
    pub e2: Option<Rc<Expr>>,
}
