use std::rc::Rc;

use crate::builtins::Value;
use crate::interpret::Context;
use crate::lexer::Span;
use crate::symbol::Symbol;

pub use crate::map::Map;

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

impl Path {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
    pub fn new_name(name: Name) -> Self {
        Self::new(vec![Segment::new_name(name)])
    }
    pub fn is_name(&self) -> bool {
        self.segments.len() == 1 && self.segments[0].is_empty()
    }
    pub fn name(&self) -> Option<&Name> {
        if self.is_name() {
            Some(&self.segments[0].name)
        } else {
            None
        }
    }
}

impl Type {
    pub fn name(&self) -> Option<&Name> {
        if let Type::Path(p) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Pat {
    pub fn name(&self) -> Option<&Name> {
        if let Pat::Path(_, _, p, None) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Expr {
    pub fn name(&self) -> Option<&Name> {
        if let Expr::Path(_, _, p) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Segment {
    pub fn new(span: Span, name: Name, ts: Vec<Type>, xts: Map<Name, Type>) -> Self {
        Self {
            span,
            name,
            ts,
            xts,
        }
    }

    pub fn new_name(name: Name) -> Self {
        Self::new(name.span, name, Vec::new(), Map::new())
    }

    pub fn is_empty(&self) -> bool {
        self.ts.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Name {
    pub span: Span,
    pub data: Symbol,
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name {
            span: Span::default(),
            data: Symbol::from(s),
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name {
            span: Span::default(),
            data: Symbol::from(s),
        }
    }
}

impl Name {
    pub fn new(span: Span, data: impl Into<Symbol>) -> Name {
        Name {
            span,
            data: data.into(),
        }
    }
    pub fn suffix(self, suffix: impl std::fmt::Display) -> Name {
        Name::new(self.span, self.data.suffix(suffix))
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Uid(usize);

impl Uid {
    pub fn incr(&mut self) {
        self.0 += 1;
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

impl Type {
    pub fn err() -> Type {
        Type::Err
    }

    pub fn hole() -> Type {
        Type::Hole
    }

    pub fn unit() -> Type {
        Type::Tuple(Vec::new())
    }

    pub fn bool() -> Type {
        Type::Cons("bool".into(), Vec::new())
    }

    pub fn i32() -> Type {
        Type::Cons("i32".into(), Vec::new())
    }

    pub fn i64() -> Type {
        Type::Cons("i64".into(), Vec::new())
    }

    pub fn f64() -> Type {
        Type::Cons("f64".into(), Vec::new())
    }

    pub fn string() -> Type {
        Type::Cons("string".into(), Vec::new())
    }

    pub fn char() -> Type {
        Type::Cons("char".into(), Vec::new())
    }

    pub fn vec(t: Type) -> Type {
        Type::Cons("Vec".into(), vec![t])
    }

    pub fn stream(t: Type) -> Type {
        Type::Cons("Stream".into(), vec![t])
    }
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

impl Stmt {
    pub fn as_var(&self) -> Option<&StmtVar> {
        if let Stmt::Var(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_def(&self) -> Option<&StmtDef> {
        if let Stmt::Def(d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_trait(&self) -> Option<&StmtTrait> {
        if let Stmt::Trait(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn as_impl(&self) -> Option<&StmtImpl> {
        if let Stmt::Impl(i) = self {
            Some(i)
        } else {
            None
        }
    }
    pub fn as_struct(&self) -> Option<&StmtStruct> {
        if let Stmt::Struct(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_enum(&self) -> Option<&StmtEnum> {
        if let Stmt::Enum(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_type(&self) -> Option<&StmtType> {
        if let Stmt::Type(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn as_expr(&self) -> Option<&Expr> {
        if let Stmt::Expr(e) = self {
            Some(e)
        } else {
            None
        }
    }
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

impl StmtTypeBody {
    pub fn unwrap_ty(&self) -> &Type {
        match self {
            StmtTypeBody::UserDefined(t) => t,
            StmtTypeBody::Builtin(_) => unreachable!(),
        }
    }
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

impl StmtDefBody {
    pub fn as_expr(&self) -> &Expr {
        let StmtDefBody::UserDefined(e) = self else {
            unreachable!()
        };
        e
    }
    pub fn as_builtin(&self) -> &BuiltinDef {
        let StmtDefBody::Builtin(b) = self else {
            unreachable!()
        };
        b
    }
}

impl StmtTypeBody {
    pub fn as_ty(&self) -> &Type {
        let StmtTypeBody::UserDefined(t) = self else {
            unreachable!()
        };
        t
    }
    pub fn as_builtin(&self) -> &BuiltinType {
        let StmtTypeBody::Builtin(b) = self else {
            unreachable!()
        };
        b
    }
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
        defs: Vec<Rc<StmtDef>>,
        types: Vec<Rc<StmtType>>,
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
    pub fn new(span: Span, name: Name, generics: Vec<Name>, body: StmtTypeBody) -> StmtType {
        StmtType {
            span,
            name,
            generics,
            body,
        }
    }
}

impl StmtTrait {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        bounds: Vec<Bound>,
        defs: Vec<Rc<StmtTraitDef>>,
        types: Vec<Rc<StmtTraitType>>,
    ) -> StmtTrait {
        StmtTrait {
            span,
            name,
            generics,
            where_clause: bounds,
            defs,
            types,
        }
    }
}

impl StmtTraitType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>) -> Self {
        Self {
            span,
            name,
            generics,
        }
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
        params: Map<Name, Type>,
        ty: Type,
        where_clause: Vec<Bound>,
        body: StmtDefBody,
    ) -> StmtDef {
        StmtDef {
            span,
            name,
            generics,
            params,
            ty,
            where_clause,
            body,
        }
    }
}

impl StmtStruct {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, fields: Map<Name, Type>) -> StmtStruct {
        StmtStruct {
            span,
            name,
            generics,
            fields,
        }
    }
}

impl StmtEnum {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, variants: Map<Name, Type>) -> StmtEnum {
        StmtEnum {
            span,
            name,
            generics,
            variants,
        }
    }
}

impl StmtTraitDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        params: Map<Name, Type>,
        ty: Type,
        where_clause: Vec<Bound>,
    ) -> Self {
        Self {
            span,
            name,
            generics,
            params,
            ty,
            where_clause,
        }
    }
}

impl Block {
    pub fn new(span: Span, stmts: Vec<Stmt>, expr: Expr) -> Block {
        Block {
            span,
            stmts,
            expr: Rc::new(expr),
        }
    }
}

impl Bound {
    pub fn span(&self) -> Span {
        match self {
            Bound::Path(s, ..) => *s,
            Bound::Trait(s, ..) => *s,
            Bound::Type(s, ..) => *s,
            Bound::Err(s) => *s,
        }
    }
    pub fn get_type(&self, x: &Name) -> Option<&Type> {
        match self {
            Bound::Path(_, _) => None,
            Bound::Trait(_, _, _, xts) => xts.get(x),
            Bound::Type(_, _) => None,
            Bound::Err(_) => None,
        }
    }
}

impl From<StmtVar> for Stmt {
    fn from(v: StmtVar) -> Stmt {
        Stmt::Var(Rc::new(v))
    }
}

impl From<StmtDef> for Stmt {
    fn from(d: StmtDef) -> Stmt {
        Stmt::Def(Rc::new(d))
    }
}
impl From<StmtImpl> for Stmt {
    fn from(i: StmtImpl) -> Stmt {
        Stmt::Impl(Rc::new(i))
    }
}

impl From<Expr> for Stmt {
    fn from(e: Expr) -> Stmt {
        Stmt::Expr(Rc::new(e))
    }
}

impl From<StmtStruct> for Stmt {
    fn from(s: StmtStruct) -> Stmt {
        Stmt::Struct(Rc::new(s))
    }
}

impl From<StmtEnum> for Stmt {
    fn from(e: StmtEnum) -> Stmt {
        Stmt::Enum(Rc::new(e))
    }
}

impl From<StmtType> for Stmt {
    fn from(t: StmtType) -> Stmt {
        Stmt::Type(Rc::new(t))
    }
}

impl From<StmtTrait> for Stmt {
    fn from(t: StmtTrait) -> Stmt {
        Stmt::Trait(Rc::new(t))
    }
}

impl From<Expr> for StmtDefBody {
    fn from(e: Expr) -> StmtDefBody {
        StmtDefBody::UserDefined(e)
    }
}

impl From<Type> for StmtTypeBody {
    fn from(t: Type) -> StmtTypeBody {
        StmtTypeBody::UserDefined(t)
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
            Expr::Path(_, t, _) => t,
            Expr::Value(t, _) => t,
            Expr::For(_, t, _, _, _) => t,
            Expr::Char(_, t, _) => t,
            Expr::Unresolved(_, t, _, _) => t,
        }
    }

    #[inline(always)]
    pub fn with_ty(self, t: Type) -> Expr {
        match self {
            Expr::Int(s, _, v) => Expr::Int(s, t, v),
            Expr::Float(s, _, v) => Expr::Float(s, t, v),
            Expr::Bool(s, _, v) => Expr::Bool(s, t, v),
            Expr::String(s, _, v) => Expr::String(s, t, v),
            Expr::Var(s, _, x) => Expr::Var(s, t, x),
            Expr::Def(s, _, x, ts) => Expr::Def(s, t, x, ts),
            Expr::Call(s, _, e, es) => Expr::Call(s, t, e, es),
            Expr::Block(s, _, b) => Expr::Block(s, t, b),
            Expr::Query(s, _, qs) => Expr::Query(s, t, qs),
            Expr::Struct(s, _, x, ts, xes) => Expr::Struct(s, t, x, ts, xes),
            Expr::Enum(s, _, x0, ts, x1, e) => Expr::Enum(s, t, x0, ts, x1, e),
            Expr::Field(s, _, e, x) => Expr::Field(s, t, e, x),
            Expr::Tuple(s, _, es) => Expr::Tuple(s, t, es),
            Expr::Assoc(s, _, b, x1, ts1) => Expr::Assoc(s, t, b, x1, ts1),
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
            Expr::Record(s, _, xes) => Expr::Record(s, t, xes),
            Expr::Path(s, _, p) => Expr::Path(s, t, p),
            Expr::Value(_, v) => Expr::Value(t, v),
            Expr::For(s, _, x, e, b) => Expr::For(s, t, x, e, b),
            Expr::Char(s, _, v) => Expr::Char(s, t, v),
            Expr::Unresolved(s, _, x, ts) => Expr::Unresolved(s, t, x, ts),
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
            Expr::Path(s, _, _) => *s,
            Expr::Value(_, _) => unreachable!(),
            Expr::For(s, _, _, _, _) => *s,
            Expr::Char(s, _, _) => *s,
            Expr::Unresolved(s, _, _, _) => *s,
        }
    }

    #[inline(always)]
    pub fn with_span(self, span: Span) -> Expr {
        match self {
            Expr::Path(_, t, p) => Expr::Path(span, t, p),
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
            Expr::Block(_, t, b) => Expr::Block(span, t, b),
            Expr::Query(_, t, qs) => Expr::Query(span, t, qs),
            Expr::Field(_, t, e, x) => Expr::Field(span, t, e, x),
            Expr::Assoc(_, t, b, x1, ts1) => Expr::Assoc(span, t, b, x1, ts1),
            Expr::Index(_, t, e, i) => Expr::Index(span, t, e, i),
            Expr::Array(_, t, es) => Expr::Array(span, t, es),
            Expr::Assign(_, t, e0, e1) => Expr::Assign(span, t, e0, e1),
            Expr::Return(_, t, e) => Expr::Return(span, t, e),
            Expr::Continue(_, t) => Expr::Continue(span, t),
            Expr::Break(_, t) => Expr::Break(span, t),
            Expr::Fun(_, t, ps, t1, e) => Expr::Fun(span, t, ps, t1, e),
            Expr::Match(_, t, e, pes) => Expr::Match(span, t, e, pes),
            Expr::Err(_, t) => Expr::Err(span, t),
            Expr::Value(t, v) => Expr::Value(t, v),
            Expr::For(_, t, x, e, b) => Expr::For(span, t, x, e, b),
            Expr::Char(_, t, v) => Expr::Char(span, t, v),
            Expr::Unresolved(_, t, x, ts) => Expr::Unresolved(span, t, x, ts),
        }
    }
}

impl Pat {
    pub fn ty(&self) -> &Type {
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

    #[inline(always)]
    pub fn with_ty(self, t: Type) -> Pat {
        match self {
            Pat::Path(s, _, p, a) => Pat::Path(s, t, p, a),
            Pat::Var(s, _, x) => Pat::Var(s, t, x),
            Pat::Tuple(s, _, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(s, _, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(s, _, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(s, _, v) => Pat::Int(s, t, v),
            Pat::String(s, _, v) => Pat::String(s, t, v),
            Pat::Wildcard(s, _) => Pat::Wildcard(s, t),
            Pat::Bool(s, _, v) => Pat::Bool(s, t, v),
            Pat::Err(s, _) => Pat::Err(s, t),
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
        }
    }

    pub fn span(&self) -> Span {
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

    #[inline(always)]
    pub fn with_span(self, s: Span) -> Pat {
        match self {
            Pat::Path(_, t, p, a) => Pat::Path(s, t, p, a),
            Pat::Var(_, t, x) => Pat::Var(s, t, x),
            Pat::Tuple(_, t, ps) => Pat::Tuple(s, t, ps),
            Pat::Struct(_, t, x, ts, xps) => Pat::Struct(s, t, x, ts, xps),
            Pat::Enum(_, t, x0, ts, x1, p) => Pat::Enum(s, t, x0, ts, x1, p),
            Pat::Int(_, t, v) => Pat::Int(s, t, v),
            Pat::Wildcard(_, t) => Pat::Wildcard(s, t),
            Pat::String(_, t, v) => Pat::String(s, t, v),
            Pat::Bool(_, t, v) => Pat::Bool(s, t, v),
            Pat::Err(_, t) => Pat::Err(s, t),
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
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
            Query::Var(s, ..) => *s,
            Query::Compute(s, ..) => *s,
            Query::Err(s, _) => *s,
        }
    }

    #[inline(always)]
    pub fn with_span(self, s: Span) -> Query {
        match self {
            Query::From(_, t, x, e) => Query::From(s, t, x, e),
            Query::Where(_, t, e) => Query::Where(s, t, e),
            Query::Select(_, t, xes) => Query::Select(s, t, xes),
            Query::Into(_, t, x, ts, es) => Query::Into(s, t, x, ts, es),
            Query::Join(_, t, x, e0, e1) => Query::Join(s, t, x, e0, e1),
            Query::Group(_, t, xs, qs) => Query::Group(s, t, xs, qs),
            Query::Over(_, t, e, qs) => Query::Over(s, t, e, qs),
            Query::Order(_, t, x, o) => Query::Order(s, t, x, o),
            Query::Var(_, t, x, e) => Query::Var(s, t, x, e),
            Query::Compute(_, t, x, e0, e1) => Query::Compute(s, t, x, e0, e1),
            Query::Err(_, t) => Query::Err(s, t),
        }
    }
}

impl Arm {
    pub fn new(span: Span, p: Pat, e: Expr) -> Arm {
        Arm { span, p, e }
    }
}
