use std::ops::Deref;
use std::rc::Rc;

use smol_str::SmolStr;

use crate::builtins::Value;
use crate::interpret::Context;
use crate::lexer::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Path {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Segment {
    pub span: Span,
    pub name: Name,
    pub types: Vec<Type>,
    pub named_types: Map<Name, Type>,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Map<K, V>(Vec<(K, V)>);

impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.0.push((k, v))
    }
    pub fn get(&self, k: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        self.0.iter().find(|(k1, _)| k1 == k).map(|(_, v)| v)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.iter()
    }
}

impl<K, V> AsRef<[(K, V)]> for Map<K, V> {
    fn as_ref(&self) -> &[(K, V)] {
        &self.0
    }
}

impl<K, V> Deref for Map<K, V> {
    type Target = [(K, V)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> From<Vec<(K, V)>> for Map<K, V> {
    fn from(v: Vec<(K, V)>) -> Self {
        Self(v)
    }
}

impl<const N: usize, K, V> From<[(K, V); N]> for Map<K, V> {
    fn from(v: [(K, V); N]) -> Self {
        Self(v.into_iter().collect())
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Type {
    pub fn name(&self) -> Option<&Name> {
        if let Type::Unresolved(p) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Pat {
    pub fn name(&self) -> Option<&Name> {
        if let Pat::Unresolved(_, _, p, None) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Expr {
    pub fn name(&self) -> Option<&Name> {
        if let Expr::Unresolved(_, _, p) = self {
            p.name()
        } else {
            None
        }
    }
}

impl Segment {
    pub fn new(span: Span, name: Name, types: Vec<Type>, named_types: Map<Name, Type>) -> Self {
        Self {
            span,
            name,
            types,
            named_types,
        }
    }

    pub fn new_name(name: Name) -> Self {
        Self::new(name.span, name, Vec::new(), Map::new())
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct Uid(pub u16);

impl Uid {
    pub fn increment(self) -> Uid {
        Uid(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Name {
    pub uid: Uid,
    pub span: Span,
    pub data: SmolStr,
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name {
            span: Span::default(),
            data: SmolStr::from(s),
            uid: Uid::default(),
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name {
            uid: Uid::default(),
            span: Span::default(),
            data: SmolStr::from(s),
        }
    }
}

impl Name {
    pub fn new(span: Span, data: impl Into<SmolStr>) -> Name {
        Name {
            uid: Uid(0),
            span,
            data: data.into(),
        }
    }
    pub fn new_unique(span: Span, uid: Uid, data: impl Into<SmolStr>) -> Name {
        Name {
            uid,
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
    pub where_clause: Vec<Bound>,
    pub defs: Vec<TraitDef>,
    pub types: Vec<TraitType>,
}

// A trait is like a predicate
// <name>(<types>)
// e.g., Clone(i32)
//       Iterator(Item=i32)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Bound {
    Unresolved(Span, Path),
    Trait(Span, TraitBound),
    Err(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraitBound {
    pub name: Name,
    pub ts: Vec<Type>,
    pub xts: Map<Name, Type>,
}

impl TraitBound {
    pub fn new(name: Name, types: Vec<Type>, named_types: Map<Name, Type>) -> Self {
        Self {
            name,
            ts: types,
            xts: named_types,
        }
    }
}

// A type is like a proposition
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Unresolved(Path),
    Cons(Name, Vec<Type>),
    Alias(Name, Vec<Type>),
    Assoc(TraitBound, Name, Vec<Type>),
    Var(Name),
    Generic(Name),
    Fun(Vec<Type>, Rc<Type>),
    Tuple(Vec<Type>),
    Record(Map<Name, Type>),
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

impl Stmt {
    pub fn as_var(&self) -> &StmtVar {
        let Stmt::Var(v) = self else { unreachable!() };
        v
    }
    pub fn as_def(&self) -> &StmtDef {
        let Stmt::Def(d) = self else { unreachable!() };
        d
    }
    pub fn as_trait(&self) -> &StmtTrait {
        let Stmt::Trait(t) = self else { unreachable!() };
        t
    }
    pub fn as_impl(&self) -> &StmtImpl {
        let Stmt::Impl(i) = self else { unreachable!() };
        i
    }
    pub fn as_struct(&self) -> &StmtStruct {
        let Stmt::Struct(s) = self else {
            unreachable!()
        };
        s
    }
    pub fn as_enum(&self) -> &StmtEnum {
        let Stmt::Enum(e) = self else { unreachable!() };
        e
    }
    pub fn as_type(&self) -> &StmtType {
        let Stmt::Type(t) = self else { unreachable!() };
        t
    }
    pub fn as_expr(&self) -> &Expr {
        let Stmt::Expr(e) = self else { unreachable!() };
        e
    }
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
    pub params: Vec<Param>,
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
    pub params: Vec<Param>,
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
    Unresolved(Span, Type, Path),
    Int(Span, Type, String),
    Float(Span, Type, String),
    Bool(Span, Type, bool),
    String(Span, Type, String),
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
    Assoc(Span, Type, TraitBound, Name, Vec<Type>),
    Match(Span, Type, Rc<Expr>, Vec<Arm>),
    Array(Span, Type, Vec<Expr>),
    Assign(Span, Type, Rc<Expr>, Rc<Expr>),
    Return(Span, Type, Rc<Expr>),
    Continue(Span, Type),
    Break(Span, Type),
    While(Span, Type, Rc<Expr>, Block),
    Fun(Span, Type, Vec<Param>, Type, Rc<Expr>),
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
pub enum UnresolvedPatField {
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
    Unresolved(Span, Type, Path, Option<Vec<UnresolvedPatField>>),
    Var(Span, Type, Name),
    Tuple(Span, Type, Vec<Pat>),
    Struct(Span, Type, Name, Vec<Type>, Map<Name, Pat>),
    Record(Span, Type, Map<Name, Pat>),
    Enum(Span, Type, Name, Vec<Type>, Name, Rc<Pat>),
    Int(Span, Type, String),
    String(Span, Type, String),
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
        defs: Vec<TraitDef>,
        types: Vec<TraitType>,
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
        params: Vec<Param>,
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

impl TraitDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        params: Vec<Param>,
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
            Bound::Unresolved(s, _) => *s,
            Bound::Trait(s, _) => *s,
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
            Expr::Unresolved(_, t, _) => t,
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
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
            Expr::Record(_, _, _) => todo!(),
            Expr::Unresolved(s, _, p) => Expr::Unresolved(s, t, p),
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
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
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
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
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
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
            Pat::Record(_, t, _) => t,
            Pat::Or(_, t, _, _) => t,
            Pat::Char(_, t, _) => t,
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
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
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
            Pat::Record(_, _, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
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
