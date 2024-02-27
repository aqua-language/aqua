use std::rc::Rc;

use serde::Deserialize;
use serde::Serialize;
use smol_str::SmolStr;

use crate::builtins::Value;
use crate::lexer::Span;
use crate::lexer::Token;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnresolvedPath {
    pub segments: Vec<(Name, Vec<Type>)>,
}

impl UnresolvedPath {
    pub fn new(segments: Vec<(Name, Vec<Type>)>) -> Self {
        Self { segments }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub span: Span,
    pub data: SmolStr,
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|data| Self {
            span: Span::default(),
            data: SmolStr::from(data),
        })
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.data.cmp(&other.data))
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name {
            span: Span::default(),
            data: SmolStr::from(s),
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name {
            span: Span::default(),
            data: SmolStr::from(s),
        }
    }
}

impl Name {
    pub fn new(span: Span, data: impl Into<SmolStr>) -> Name {
        Name {
            span,
            data: data.into(),
        }
    }
}

impl Index {
    pub fn new(span: Span, index: usize) -> Index {
        Index { span, data: index }
    }
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
    pub defs: Vec<StmtDef>,
    pub types: Vec<StmtType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtTrait {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub bounds: Vec<Bound>,
    pub defs: Vec<TraitDef>,
    pub types: Vec<TraitType>,
}

// A trait is like a predicate
// <name>(<types>)
// e.g., Clone(i32)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Bound {
    Unresolved(Span, UnresolvedPath),
    Trait(Span, Name, Vec<Type>),
    Err(Span),
}

// A type is like a proposition
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    Unresolved(UnresolvedPath),
    Cons(Name, Vec<Type>),
    Alias(Name, Vec<Type>),
    Assoc(Name, Vec<Type>, Name, Vec<Type>),
    Var(Name),
    Generic(Name),
    Fun(Vec<Type>, Rc<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(Name, Type)>),
    Array(Rc<Type>, Option<usize>),
    Never,
    Hole,
    Err,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Param {
    pub span: Span,
    pub name: Name,
    pub ty: Type,
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
    Var(StmtVar),
    Def(StmtDef),
    Trait(StmtTrait),
    Impl(StmtImpl),
    Struct(StmtStruct),
    Enum(StmtEnum),
    Type(StmtType),
    Expr(Expr),
    Err(Span),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraitType {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraitDef {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub bounds: Vec<Bound>,
    pub params: Vec<Param>,
    pub ty: Type,
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
    pub where_clause: Vec<Bound>,
    pub params: Vec<Param>,
    pub ty: Type,
    pub body: Body,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Body {
    Expr(Expr),
    Builtin,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtStruct {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub fields: Vec<(Name, Type)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtEnum {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub variants: Vec<(Name, Type)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StmtType {
    pub span: Span,
    pub name: Name,
    pub generics: Vec<Name>,
    pub ty: Type,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Index {
    pub span: Span,
    pub data: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Unresolved(Span, Type, UnresolvedPath),
    Prefix(Span, Type, Token, Rc<Expr>),
    Infix(Span, Type, Token, Rc<Expr>, Rc<Expr>),
    Postfix(Span, Type, Token, Rc<Expr>),
    Int(Span, Type, String),
    Float(Span, Type, String),
    Bool(Span, Type, bool),
    String(Span, Type, String),
    Struct(Span, Type, Name, Vec<Type>, Vec<(Name, Expr)>),
    Tuple(Span, Type, Vec<Expr>),
    Record(Span, Type, Vec<(Name, Expr)>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Expr>),
    Field(Span, Type, Rc<Expr>, Name),
    Index(Span, Type, Rc<Expr>, Index),
    Var(Span, Type, Name),
    Def(Span, Type, Name, Vec<Type>),
    Call(Span, Type, Rc<Expr>, Vec<Expr>),
    Block(Span, Type, Vec<Stmt>, Rc<Expr>),
    Query(Span, Type, Vec<Query>),
    Assoc(Span, Type, Name, Vec<Type>, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Vec<(Pat, Expr)>),
    Array(Span, Type, Vec<Expr>),
    Assign(Span, Type, Rc<Expr>, Rc<Expr>),
    Return(Span, Type, Rc<Expr>),
    Continue(Span, Type),
    Break(Span, Type),
    While(Span, Type, Rc<Expr>, Rc<Expr>),
    Fun(Span, Type, Vec<Param>, Type, Rc<Expr>),
    Err(Span, Type),
    Value(Type, Value),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatArg {
    Named(Name, Pat),
    Unnamed(Pat),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Pat {
    Unresolved(Span, Type, UnresolvedPath, Option<Vec<PatArg>>),
    Var(Span, Type, Name),
    Tuple(Span, Type, Vec<Pat>),
    Struct(Span, Type, Name, Vec<Type>, Vec<(Name, Pat)>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Pat>),
    Int(Span, Type, String),
    String(Span, Type, String),
    Wildcard(Span, Type),
    Bool(Span, Type, bool),
    Err(Span, Type),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Query {
    From(Span, Type, Vec<(Name, Expr)>),
    Where(Span, Type, Rc<Expr>),
    Select(Span, Type, Vec<(Name, Expr)>),
    Join(Span, Type, Vec<(Name, Expr)>, Rc<Expr>),
    Group(Span, Type, Vec<Name>, Vec<Query>),
    Over(Span, Type, Rc<Expr>, Vec<Query>),
    Order(Span, Type, Vec<(Name, bool)>),
    With(Span, Type, Vec<(Name, Expr)>),
    Into(Span, Type, Vec<Expr>),
    Compute(Span, Type, Vec<(Name, Expr, Expr)>),
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Program {
        Program { stmts }
    }
}

impl StmtImpl {
    pub fn new(
        span: Span,
        generics: Vec<Name>,
        head: Bound,
        where_clause: Vec<Bound>,
        defs: Vec<StmtDef>,
        types: Vec<StmtType>,
    ) -> StmtImpl {
        StmtImpl {
            span,
            generics,
            where_clause,
            types,
            head,
            defs,
        }
    }
}

impl StmtType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, ty: Type) -> StmtType {
        StmtType {
            span,
            name,
            generics,
            ty,
        }
    }
}

impl StmtTrait {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        body: Vec<Bound>,
        defs: Vec<TraitDef>,
        types: Vec<TraitType>,
    ) -> StmtTrait {
        StmtTrait {
            span,
            name,
            generics,
            bounds: body,
            defs,
            types,
        }
    }
}

impl TraitType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>) -> Self {
        Self {
            span,
            name,
            generics,
        }
    }
}

impl Param {
    pub fn new(span: Span, name: Name, ty: Type) -> Param {
        Param { span, name, ty }
    }
}

impl StmtVar {
    pub fn new(span: Span, name: Name, ty: Type, expr: Expr) -> StmtVar {
        StmtVar {
            span,
            name,
            ty,
            expr,
        }
    }
}

impl StmtDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        where_clause: Vec<Bound>,
        params: Vec<Param>,
        ty: Type,
        body: Body,
    ) -> StmtDef {
        StmtDef {
            span,
            name,
            generics,
            where_clause,
            params,
            ty,
            body,
        }
    }
}

impl StmtStruct {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        fields: Vec<(Name, Type)>,
    ) -> StmtStruct {
        StmtStruct {
            span,
            name,
            generics,
            fields,
        }
    }
}

impl StmtEnum {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        variants: Vec<(Name, Type)>,
    ) -> StmtEnum {
        StmtEnum {
            span,
            name,
            generics,
            variants,
        }
    }
}

impl TraitDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        where_clause: Vec<Bound>,
        params: Vec<Param>,
        ty: Type,
    ) -> Self {
        Self {
            span,
            name,
            generics,
            bounds: where_clause,
            params,
            ty,
        }
    }
}

impl Bound {
    pub fn span(&self) -> Span {
        match self {
            Bound::Unresolved(s, _) => *s,
            Bound::Trait(s, _, _) => *s,
            Bound::Err(s) => *s,
        }
    }
}

impl From<StmtVar> for Stmt {
    fn from(v: StmtVar) -> Stmt {
        Stmt::Var(v)
    }
}

impl From<StmtDef> for Stmt {
    fn from(d: StmtDef) -> Stmt {
        Stmt::Def(d)
    }
}
impl From<StmtImpl> for Stmt {
    fn from(i: StmtImpl) -> Stmt {
        Stmt::Impl(i)
    }
}

impl From<Expr> for Stmt {
    fn from(e: Expr) -> Stmt {
        Stmt::Expr(e)
    }
}

impl From<StmtStruct> for Stmt {
    fn from(s: StmtStruct) -> Stmt {
        Stmt::Struct(s)
    }
}

impl From<StmtEnum> for Stmt {
    fn from(e: StmtEnum) -> Stmt {
        Stmt::Enum(e)
    }
}

impl From<StmtType> for Stmt {
    fn from(t: StmtType) -> Stmt {
        Stmt::Type(t)
    }
}

impl From<StmtTrait> for Stmt {
    fn from(t: StmtTrait) -> Stmt {
        Stmt::Trait(t)
    }
}

impl Expr {
    pub fn ty(&self) -> &Type {
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
            Expr::Unresolved(_, t, _) => t,
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => todo!(),
            Expr::Postfix(_, _, _, _) => todo!(),
            Expr::Prefix(_, _, _, _) => todo!(),
        }
    }

    #[inline(always)]
    pub fn of(self, t: Type) -> Expr {
        match self {
            Expr::Int(s, _, v) => Expr::Int(s, t, v),
            Expr::Float(s, _, v) => Expr::Float(s, t, v),
            Expr::Bool(s, _, v) => Expr::Bool(s, t, v),
            Expr::String(s, _, v) => Expr::String(s, t, v),
            Expr::Var(s, _, x) => Expr::Var(s, t, x),
            Expr::Def(s, _, x, ts) => Expr::Def(s, t, x, ts),
            Expr::Call(s, _, e, es) => Expr::Call(s, t, e, es),
            Expr::Block(s, _, ss, e) => Expr::Block(s, t, ss, e),
            Expr::Query(s, _, qs) => Expr::Query(s, t, qs),
            Expr::Struct(s, _, x, ts, xes) => Expr::Struct(s, t, x, ts, xes),
            Expr::Enum(s, _, x0, ts, x1, e) => Expr::Enum(s, t, x0, ts, x1, e),
            Expr::Field(s, _, e, x) => Expr::Field(s, t, e, x),
            Expr::Tuple(s, _, es) => Expr::Tuple(s, t, es),
            Expr::Assoc(s, _, x0, ts0, x1, ts1) => Expr::Assoc(s, t, x0, ts0, x1, ts1),
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
            Expr::Record(_, _, _) => todo!(),
            Expr::Unresolved(s, _, p) => Expr::Unresolved(s, t, p),
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => todo!(),
            Expr::Postfix(_, _, _, _) => todo!(),
            Expr::Prefix(_, _, _, _) => todo!(),
        }
    }

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
            Expr::Unresolved(s, _, _) => *s,
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => todo!(),
            Expr::Postfix(_, _, _, _) => todo!(),
            Expr::Prefix(_, _, _, _) => todo!(),
        }
    }

    #[inline(always)]
    pub fn with_span(self, span: Span) -> Expr {
        match self {
            Expr::Unresolved(_, t, p) => Expr::Unresolved(span, t, p),
            Expr::Record(_, t, xes) => Expr::Record(span, t, xes),
            Expr::While(_, t, e0, e1) => Expr::While(span, t, e0, e1),
            Expr::Int(_, t, v) => Expr::Int(span, t, v),
            Expr::Float(_, t, v) => Expr::Float(span, t, v),
            Expr::Bool(_, t, v) => Expr::Bool(span, t, v),
            Expr::String(_, t, v) => Expr::String(span, t, v),
            Expr::Struct(_, t, x, ts, xes) => Expr::Struct(span, t, x, ts, xes),
            Expr::Tuple(_, t, es) => Expr::Tuple(span, t, es),
            Expr::Enum(_, t, x0, ts, x1, e) => Expr::Enum(span, t, x0, ts, x1, e),
            Expr::Var(_, t, x) => Expr::Var(span, t, x),
            Expr::Def(_, t, x, ts) => Expr::Def(span, t, x, ts),
            Expr::Call(_, t, e, es) => Expr::Call(span, t, e, es),
            Expr::Block(_, t, ss, e) => Expr::Block(span, t, ss, e),
            Expr::Query(_, t, qs) => Expr::Query(span, t, qs),
            Expr::Field(_, t, e, x) => Expr::Field(span, t, e, x),
            Expr::Assoc(_, t, x0, ts0, x1, ts1) => Expr::Assoc(span, t, x0, ts0, x1, ts1),
            Expr::Index(_, t, e, i) => Expr::Index(span, t, e, i),
            Expr::Array(_, t, es) => Expr::Array(span, t, es),
            Expr::Assign(_, t, e0, e1) => Expr::Assign(span, t, e0, e1),
            Expr::Return(_, t, e) => Expr::Return(span, t, e),
            Expr::Continue(_, t) => Expr::Continue(span, t),
            Expr::Break(_, t) => Expr::Break(span, t),
            Expr::Fun(_, t, ps, t1, e) => Expr::Fun(span, t, ps, t1, e),
            Expr::Match(_, t, e, pes) => Expr::Match(span, t, e, pes),
            Expr::Err(_, t) => Expr::Err(span, t),
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => todo!(),
            Expr::Postfix(_, _, _, _) => todo!(),
            Expr::Prefix(_, _, _, _) => todo!(),
        }
    }
}

impl Pat {
    pub fn ty(&self) -> &Type {
        match self {
            Pat::Unresolved(_, t, _, _) => t,
            Pat::Var(_, t, ..) => t,
            Pat::Tuple(_, t, ..) => t,
            Pat::Struct(_, t, ..) => t,
            Pat::Enum(_, t, ..) => t,
            Pat::Int(_, t, ..) => t,
            Pat::Wildcard(_, t, ..) => t,
            Pat::String(_, t, ..) => t,
            Pat::Bool(_, t, ..) => t,
            Pat::Err(_, t) => t,
        }
    }

    #[inline(always)]
    pub fn with_ty(self, t: Type) -> Pat {
        match self {
            Pat::Unresolved(s, _, p, a) => Pat::Unresolved(s, t, p, a),
            Pat::Var(s, _, x) => Pat::Var(s, t, x),
            Pat::Tuple(s, _, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(s, _, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(s, _, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(s, _, v) => Pat::Int(s, t, v),
            Pat::String(s, _, v) => Pat::String(s, t, v),
            Pat::Wildcard(s, _) => Pat::Wildcard(s, t),
            Pat::Bool(s, _, v) => Pat::Bool(s, t, v),
            Pat::Err(s, _) => Pat::Err(s, t),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Pat::Unresolved(s, _, _, _) => *s,
            Pat::Var(s, ..) => *s,
            Pat::Tuple(s, ..) => *s,
            Pat::Struct(s, ..) => *s,
            Pat::Enum(s, ..) => *s,
            Pat::Int(s, ..) => *s,
            Pat::Wildcard(s, ..) => *s,
            Pat::String(s, ..) => *s,
            Pat::Bool(s, ..) => *s,
            Pat::Err(s, _) => *s,
        }
    }

    #[inline(always)]
    pub fn with_span(self, s: Span) -> Pat {
        match self {
            Pat::Unresolved(_, t, p, a) => Pat::Unresolved(s, t, p, a),
            Pat::Var(_, t, x) => Pat::Var(s, t, x),
            Pat::Tuple(_, t, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(_, t, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(_, t, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(_, t, v) => Pat::Int(s, t, v),
            Pat::Wildcard(_, t) => Pat::Wildcard(s, t),
            Pat::String(_, t, v) => Pat::String(s, t, v),
            Pat::Bool(_, t, v) => Pat::Bool(s, t, v),
            Pat::Err(_, t) => Pat::Err(s, t),
        }
    }
}

impl Query {
    pub fn span(&self) -> Span {
        match self {
            Query::From(s, ..) => *s,
            Query::Where(s, ..) => *s,
            Query::Select(s, ..) => *s,
            Query::Into(s, ..) => *s,
            Query::Join(s, ..) => *s,
            Query::Group(s, ..) => *s,
            Query::Over(s, ..) => *s,
            Query::Order(s, ..) => *s,
            Query::With(s, ..) => *s,
            Query::Compute(s, ..) => *s,
        }
    }

    #[inline(always)]
    pub fn with_span(self, s: Span) -> Query {
        match self {
            Query::From(_, t, xes) => Query::From(s, t, xes),
            Query::Where(_, t, e) => Query::Where(s, t, e),
            Query::Select(_, t, xes) => Query::Select(s, t, xes),
            Query::Into(_, t, es) => Query::Into(s, t, es),
            Query::Join(_, t, xes, e) => Query::Join(s, t, xes, e),
            Query::Group(_, t, xs, qs) => Query::Group(s, t, xs, qs),
            Query::Over(_, t, e, qs) => Query::Over(s, t, e, qs),
            Query::Order(_, t, xs) => Query::Order(s, t, xs),
            Query::With(_, t, xes) => Query::With(s, t, xes),
            Query::Compute(_, t, xes) => Query::Compute(s, t, xes),
        }
    }
}
