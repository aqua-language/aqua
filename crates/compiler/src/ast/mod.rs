pub mod display;
pub mod parse;
pub mod passes;
pub mod utils;

use std::rc::Rc;

use runtime::prelude::Send;
use runtime::prelude::Sync;

use crate::builtins::value::Value;

pub use crate::collections::map::Map;
use crate::interpret::Context;
use crate::report::span::Span;
use crate::report::symbol::Symbol;
use parse::token::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ast {
    pub span: Span,
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
    Function(Vec<Type>, Rc<Type>, Effect),
    Tuple(Vec<Type>),
    Record(Map<Name, Type>),
    Array(Rc<Type>, Option<usize>),
    Never,
    Unit,
    Paren(Rc<Type>),
    Ref(Vec<Loan>, Rc<Type>, bool),
    Err,
    Unknown, // A placeholder for a type that has not been annotated yet.
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Effect {
    Var(EffectVar),
    Cons(Name, Rc<Effect>),
    Nil,
    Err,
    Unknown, // A placeholder for an effect that has not been annotated yet.
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EffectVar(pub u32);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Loan {
    pub place: Place,
    pub mutable: bool,
}

impl Loan {
    pub fn new(place: Place, mutable: bool) -> Loan {
        Loan { place, mutable }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stmt {
    Local(Rc<StmtLocal>),
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
    pub params: Vec<Local>,
    pub ty: Type,
    pub effect: Effect,
    pub where_clause: Vec<Impl>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtLocal {
    pub span: Span,
    pub local: Local,
    pub expr: Option<Expr>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtDef {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Local>,
    pub ty: Type,
    pub effect: Effect,
    pub where_clause: Vec<Impl>,
    pub body: ExprBody,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtDefBuiltin {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Local>,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
    pub span: Span,
    pub local: Local,
    pub elems: Vec<PlaceElem>,
}

impl Place {
    pub fn new(span: Span, local: Local, elems: Vec<PlaceElem>) -> Place {
        Place { span, local, elems }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Local {
    pub span: Span,
    pub name: Name,
    pub ty: Type,
    pub mutable: bool,
}

impl Local {
    pub fn new(span: Span, name: Name, ty: Type, mutable: bool) -> Local {
        Local {
            span,
            name,
            ty,
            mutable,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PlaceElem {
    Index(Span, Type, Index),
    Field(Span, Type, Name),
    Deref(Span, Type),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Path(Span, Type, Path),
    Ref(Span, Type, Rc<Expr>, bool),
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
    Index(Span, Type, Rc<Expr>, Index),
    Local(Span, Type, Name, bool),
    Def(Span, Type, Name, Vec<Type>),
    Call(Span, Type, Rc<Expr>, Vec<Expr>),
    Block(Span, Type, Rc<Block>),
    Query(Span, Type, Local, Rc<Expr>, Vec<QueryOp>),
    QueryInto(
        Span,
        Type,
        Local,
        Rc<Expr>,
        Vec<QueryOp>,
        Name,
        Vec<Type>,
        Vec<Expr>,
    ),
    Assoc(Span, Type, Impl, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Vec<(Pat, Expr)>),
    IfElse(Span, Type, Rc<Expr>, Rc<Block>, Rc<Block>),
    Array(Span, Type, Vec<Expr>),
    Assign(Span, Type, Rc<Expr>, Rc<Expr>),
    Return(Span, Type, Rc<Expr>),
    Continue(Span, Type, Option<Name>),
    Break(Span, Type, Option<Name>),
    While(Span, Type, Option<Name>, Rc<Expr>, Rc<Block>),
    Lambda(Span, Type, Vec<Local>, Type, Rc<Expr>),
    Closure(Span, Type, usize, Vec<Local>, Vec<Place>, Type, Rc<Expr>),
    For(Span, Type, Option<Name>, Local, Rc<Expr>, Rc<Block>),
    Loop(Span, Type, Option<Name>, Rc<Block>),
    Err(Span, Type),
    InfixBinaryOp(Span, Type, Token, Rc<Expr>, Rc<Expr>),
    PrefixUnaryOp(Span, Type, Token, Rc<Expr>),
    PostfixUnaryOp(Span, Type, Token, Rc<Expr>),
    Annotate(Span, Type, Rc<Expr>),
    Paren(Span, Type, Rc<Expr>),
    Dot(Span, Type, Rc<Expr>, Name, Vec<Type>, Vec<Expr>),
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
    Local(Span, Type, Name, bool),
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
    From(Span, Local, Rc<Expr>),
    Distinct(Span),
    Local(Span, Local, Rc<Expr>),
    Drop(Span, Name),
    Union(Span, Rc<Expr>),
    Where(Span, Rc<Expr>),
    Select(Span, Vec<(Local, Expr)>),
    Limit(Span, Rc<Expr>),
    OverCompute(Span, Rc<Expr>, Vec<Aggr>),
    GroupOverCompute(Span, Local, Rc<Expr>, Rc<Expr>, Vec<Aggr>),
    JoinOn(Span, Local, Rc<Expr>, Rc<Expr>),
    JoinOverOn(Span, Local, Rc<Expr>, Rc<Expr>, Rc<Expr>),
    // Compute(Span, Name, Rc<Expr>, Rc<Expr>),
    Err(Span),
}

/// An aggregation function.
/// l = x of e1 [if e2]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Aggr {
    pub local: Local,
    pub name: Name,
    pub reduce_expr: Rc<Expr>,
    pub filter_expr: Option<Rc<Expr>>,
}
