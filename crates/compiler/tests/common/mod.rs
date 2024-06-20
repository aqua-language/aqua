#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Aggr;
use compiler::ast::Block;
use compiler::ast::Expr;
use compiler::ast::Index;
use compiler::ast::Map;
use compiler::ast::Name;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Query;
use compiler::ast::Stmt;
use compiler::ast::StmtDef;
use compiler::ast::StmtDefBody;
use compiler::ast::StmtEnum;
use compiler::ast::StmtImpl;
use compiler::ast::StmtStruct;
use compiler::ast::StmtTrait;
use compiler::ast::StmtTraitDef;
use compiler::ast::StmtTraitType;
use compiler::ast::StmtType;
use compiler::ast::StmtTypeBody;
use compiler::ast::StmtVar;
use compiler::ast::Trait;
use compiler::ast::Type;

use compiler::ast::TypeVar;
use compiler::builtins::Value;
use compiler::lexer::Lexer;
use compiler::lexer::Span;
use compiler::parser::Parser;
use compiler::Compiler;
use compiler::Recovered;

#[macro_export]
macro_rules! check {
    ($a:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        assert!($a.msg == msg, "{}", common::diff($a.msg, msg.to_string()));
    }};
    ($a:expr, $b:expr) => {
        assert!($a == $b, "{}", {
            let a_str = format!("{}", $a);
            let b_str = format!("{}", $b);
            if a_str != b_str {
                common::diff(a_str, b_str)
            } else {
                let a_str = format!("{}", $a.verbose());
                let b_str = format!("{}", $b.verbose());
                if a_str != b_str {
                    common::diff(a_str, b_str)
                } else {
                    let a_str = format!("{:#?}", $a);
                    let b_str = format!("{:#?}", $b);
                    common::diff(a_str, b_str)
                }
            }
        });
    };
    ($a:expr, $b:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        check!($a.val, $b);
        assert!($a.msg == msg, "{}", common::diff($a.msg, msg.to_string()));
    }};
}

pub fn diff(a: String, b: String) -> String {
    let mut output = String::new();
    let diff = similar::TextDiff::from_lines(&a, &b);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "A ",
            similar::ChangeTag::Insert => "B ",
            similar::ChangeTag::Equal => "  ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

pub fn program<const N: usize>(ss: [Stmt; N]) -> Program {
    Program::new(span(), vec(ss))
}

pub fn stmt_trait<const N: usize, const M: usize, const K: usize, const L: usize>(
    x: &'static str,
    gs: [&'static str; N],
    bounds: [Trait; M],
    defs: [StmtTraitDef; K],
    types: [StmtTraitType; L],
) -> Stmt {
    StmtTrait::new(
        span(),
        name(x),
        app(gs, name),
        vec(bounds),
        app(defs, Rc::new),
        app(types, Rc::new),
    )
    .into()
}

fn vec<const N: usize, T: Clone>(xs: [T; N]) -> Vec<T> {
    xs.into_iter().collect()
}

fn map<const N: usize, K, V>(kvs: [(K, V); N]) -> Map<K, V> {
    kvs.into_iter().collect()
}

fn name_map<const N: usize, V>(xvs: [(&'static str, V); N]) -> Map<Name, V> {
    xvs.into_iter().map(|(x, v)| (name(x), v)).collect()
}

fn app<const N: usize, T, U: Clone>(xs: [T; N], f: impl Fn(T) -> U) -> Vec<U> {
    xs.into_iter().map(f).collect()
}

#[allow(unused)]
fn map2<const N: usize, T, U, V: Clone>(xs: [(T, U); N], f: impl Fn(T, U) -> V) -> Vec<V> {
    xs.into_iter().map(|(x, y)| f(x, y)).collect()
}

fn map3<const N: usize, T, U, S, V: Clone>(xs: [(T, U, S); N], f: impl Fn(T, U, S) -> V) -> Vec<V> {
    xs.into_iter().map(|(x, y, z)| f(x, y, z)).collect()
}

// Create a Rule:
// impl<{gs}> {head} where {body}
pub fn stmt_impl<const N: usize, const M: usize, const K: usize, const L: usize>(
    gs: [&'static str; N],
    head: Trait,
    body: [Trait; M],
    defs: [Stmt; K],
    types: [Stmt; L],
) -> Stmt {
    StmtImpl::new(
        span(),
        app(gs, name),
        head,
        vec(body),
        app(defs, |s| Rc::new(s.as_def().unwrap().clone())),
        app(types, |s| Rc::new(s.as_type().unwrap().clone())),
    )
    .into()
}

// Create a Fact:
// impl<{gs}> {head}
pub fn fact<const N: usize, const M: usize, const K: usize>(
    gs: [&'static str; N],
    head: Trait,
    defs: [StmtDef; M],
    types: [StmtType; K],
) -> StmtImpl {
    StmtImpl::new(
        span(),
        app(gs, name),
        head,
        vec([]),
        app(defs, Rc::new),
        app(types, Rc::new),
    )
}

pub fn type_bound(t: Type) -> Trait {
    Trait::Type(Rc::new(t))
}

pub fn trait_bound<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xts: [(&'static str, Type); M],
) -> Trait {
    Trait::Cons(name(x), vec(ts), name_map(xts))
}

pub fn bound_err() -> Trait {
    Trait::Err
}

pub fn ty_alias<const N: usize>(x: &'static str, ts: [Type; N]) -> Type {
    Type::Alias(name(x), vec(ts))
}

pub fn ty(x: &'static str) -> Type {
    debug_assert!(!x.starts_with('?'));
    ty_con(x, [])
}

pub fn name(x: impl Into<Name>) -> Name {
    x.into()
}

pub fn ty_con<const N: usize>(x: &'static str, types: [Type; N]) -> Type {
    Type::Cons(name(x), vec(types))
}

pub fn ty_tuple<const N: usize>(ts: [Type; N]) -> Type {
    Type::Tuple(vec(ts))
}

pub fn ty_fun<const N: usize>(ts: [Type; N], t: Type) -> Type {
    Type::Fun(vec(ts), Rc::new(t))
}

pub fn ty_var(id: u32) -> Type {
    Type::Var(TypeVar(id))
}

pub fn ty_gen(x: &'static str) -> Type {
    Type::Generic(name(x))
}

pub fn ty_assoc<const N: usize, const M: usize, const K: usize>(
    x0: &'static str,
    ts0: [Type; N],
    xts0: [(&'static str, Type); M],
    x1: &'static str,
    ts1: [Type; K],
) -> Type {
    Type::Assoc(trait_bound(x0, ts0, xts0), name(x1), vec(ts1))
}

pub fn ty_record<const N: usize>(xts: [(&'static str, Type); N]) -> Type {
    Type::Record(name_map(xts))
}

pub fn ty_unit() -> Type {
    Type::Tuple(vec([]))
}

pub fn pat_record<const N: usize>(xps: [(&'static str, Pat); N]) -> Pat {
    Pat::Record(span(), Type::Unknown, name_map(xps))
}

pub mod sugared {
    use std::rc::Rc;

    use compiler::ast::Expr;
    use compiler::ast::Pat;
    use compiler::ast::Type;
    use compiler::lexer::Token;

    use super::span;

    fn expr_binop(t: Token, e0: Expr, e1: Expr) -> Expr {
        Expr::InfixBinaryOp(span(), Type::Unknown, t, Rc::new(e0), Rc::new(e1))
    }

    fn expr_unop(t: Token, e: Expr) -> Expr {
        Expr::PrefixUnaryOp(span(), Type::Unknown, t, Rc::new(e))
    }

    pub fn expr_add(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Plus, e0, e1)
    }

    pub fn expr_sub(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Minus, e0, e1)
    }

    pub fn expr_mul(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Star, e0, e1)
    }

    pub fn expr_div(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Slash, e0, e1)
    }

    pub fn expr_eq(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::EqEq, e0, e1)
    }

    pub fn expr_ne(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::NotEq, e0, e1)
    }

    pub fn expr_lt(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Lt, e0, e1)
    }

    pub fn expr_le(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Le, e0, e1)
    }

    pub fn expr_gt(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Gt, e0, e1)
    }

    pub fn expr_ge(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Ge, e0, e1)
    }

    pub fn expr_and(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::And, e0, e1)
    }

    pub fn expr_or(e0: Expr, e1: Expr) -> Expr {
        expr_binop(Token::Or, e0, e1)
    }

    pub fn expr_not(e: Expr) -> Expr {
        expr_unop(Token::Not, e)
    }

    pub fn expr_neg(e: Expr) -> Expr {
        expr_unop(Token::Minus, e)
    }

    pub fn expr_paren(e: Expr) -> Expr {
        Expr::Paren(span(), Type::Unknown, Rc::new(e))
    }

    pub fn pat_paren(p: Pat) -> Pat {
        Pat::Paren(span(), Type::Unknown, Rc::new(p))
    }
}

pub mod desugared {
    use std::rc::Rc;

    use compiler::ast::Expr;
    use compiler::ast::Type;

    use super::arms;
    use super::expr_bool;
    use super::pat_bool;
    use super::pat_wild;
    use super::span;
    use super::unresolved::expr_assoc;
    use super::vec;

    pub fn expr_unop(x0: &'static str, x1: &'static str, e: Expr) -> Expr {
        let vec = vec([e]);
        Expr::Call(
            span(),
            Type::Unknown,
            Rc::new(expr_assoc(x0, [], [], x1)),
            vec,
        )
    }

    pub fn expr_binop(x0: &'static str, x1: &'static str, e0: Expr, e1: Expr) -> Expr {
        Expr::Call(
            span(),
            Type::Unknown,
            Rc::new(expr_assoc(x0, [], [], x1)),
            vec([e0, e1]),
        )
    }

    pub fn expr_add(e0: Expr, e1: Expr) -> Expr {
        expr_binop("Add", "add", e0, e1)
    }

    pub fn expr_sub(e0: Expr, e1: Expr) -> Expr {
        expr_binop("Sub", "sub", e0, e1)
    }

    pub fn expr_mul(e0: Expr, e1: Expr) -> Expr {
        expr_binop("Mul", "mul", e0, e1)
    }

    pub fn expr_div(e0: Expr, e1: Expr) -> Expr {
        expr_binop("Div", "div", e0, e1)
    }

    pub fn expr_eq(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialEq", "eq", e0, e1)
    }

    pub fn expr_ne(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialEq", "ne", e0, e1)
    }

    pub fn expr_lt(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialOrd", "lt", e0, e1)
    }

    pub fn expr_le(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialOrd", "le", e0, e1)
    }

    pub fn expr_gt(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialOrd", "gt", e0, e1)
    }

    pub fn expr_ge(e0: Expr, e1: Expr) -> Expr {
        expr_binop("PartialOrd", "ge", e0, e1)
    }
    pub fn expr_not(e: Expr) -> Expr {
        expr_unop("Not", "not", e)
    }

    pub fn expr_neg(e: Expr) -> Expr {
        expr_unop("Neg", "neg", e)
    }

    pub fn expr_and(e0: Expr, e1: Expr) -> Expr {
        Expr::Match(
            span(),
            Type::Unknown,
            Rc::new(e0),
            arms([(pat_bool(true), e1), (pat_wild(), expr_bool(false))]),
        )
    }

    pub fn expr_or(e0: Expr, e1: Expr) -> Expr {
        Expr::Match(
            span(),
            Type::Unknown,
            Rc::new(e0),
            arms([(pat_bool(true), expr_bool(true)), (pat_wild(), e1)]),
        )
    }
}

pub mod unresolved {
    use std::rc::Rc;

    use super::expr_assign;
    use super::map;
    use super::map3;
    use super::name_map;
    use super::vec;
    use compiler::ast::Expr;
    use compiler::ast::Map;
    use compiler::ast::Name;
    use compiler::ast::Pat;
    use compiler::ast::Path;
    use compiler::ast::PathPatField;
    use compiler::ast::Segment;
    use compiler::ast::Trait;
    use compiler::ast::Type;

    use super::app;
    use super::expr_call;
    use super::name;
    use super::span;

    // x
    pub fn expr_var(x: &'static str) -> Expr {
        Expr::Path(
            span(),
            Type::Unknown,
            Path::new(vec([segment(x, vec([]), map([]))])),
        )
    }

    // x[ts](x0 = e0, x1 = e1, ..)
    pub fn expr_struct<const N: usize, const M: usize>(
        x: &'static str,
        ts: [Type; N],
        xes: [(&'static str, Expr); M],
    ) -> Expr {
        Expr::Call(
            span(),
            Type::Unknown,
            Rc::new(name_expr(x, ts)),
            app(xes, |(x, e)| expr_assign(expr_var(x), e)),
        )
    }

    pub fn expr_def<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
        name_expr(x, ts)
    }

    fn name_expr<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
        Expr::Path(span(), Type::Unknown, name_path(x, ts, []))
    }

    pub fn expr_variant<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        x1: &'static str,
        es: [Expr; M],
    ) -> Expr {
        Expr::Call(
            span(),
            Type::Unknown,
            Rc::new(Expr::Path(
                span(),
                Type::Unknown,
                path([(x0, vec(ts), map([])), (x1, vec([]), map([]))]),
            )),
            vec(es),
        )
    }

    pub fn expr_unit_variant<const N: usize>(
        x0: &'static str,
        ts: [Type; N],
        x1: &'static str,
    ) -> Expr {
        Expr::Path(
            span(),
            Type::Unknown,
            path([(x0, vec(ts), map([])), (x1, vec([]), map([]))]),
        )
    }

    pub fn expr_call_method<const N: usize, const M: usize>(
        e: Expr,
        x: &'static str,
        ts: [Type; N],
        es: [Expr; M],
    ) -> Expr {
        Expr::Dot(span(), Type::Unknown, Rc::new(e), name(x), vec(ts), vec(es))
    }

    pub fn expr_call_direct<const N: usize, const M: usize>(
        x: &'static str,
        ts: [Type; N],
        es: [Expr; M],
    ) -> Expr {
        expr_call(expr_def(x, ts), es)
    }

    pub fn expr_assoc<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
        x1: &'static str,
    ) -> Expr {
        Expr::Path(
            span(),
            Type::Unknown,
            path([(x0, vec(ts), name_map(xts)), (x1, vec([]), map([]))]),
        )
    }

    pub fn expr_enum(x0: &'static str, x1: &'static str) -> Expr {
        Expr::Path(
            span(),
            Type::Unknown,
            Path::new(vec([
                segment(x0, vec([]), map([])),
                segment(x1, vec([]), map([])),
            ])),
        )
    }

    pub fn pat_var(x: &'static str) -> Pat {
        Pat::Path(span(), Type::Unknown, name_path(x, [], []), None)
    }

    pub fn pat_enum<const N: usize>(
        x0: &'static str,
        ts: [Type; N],
        x1: &'static str,
        p: Pat,
    ) -> Pat {
        Pat::Path(
            span(),
            Type::Unknown,
            path([(x0, vec(ts), map([])), (x1, vec([]), map([]))]),
            Some(vec([PathPatField::Unnamed(p)])),
        )
    }

    pub fn pat_struct<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        xps: [(&'static str, Pat); M],
    ) -> Pat {
        Pat::Path(
            span(),
            Type::Unknown,
            path([(x0, vec(ts), map([]))]),
            Some(app(xps, |(x, p)| PathPatField::Named(name(x), p))),
        )
    }

    pub fn pat_unit_struct<const N: usize>(x: &'static str, ts: [Type; N]) -> Pat {
        Pat::Path(span(), Type::Unknown, path([(x, vec(ts), map([]))]), None)
    }

    pub fn ty_con<const N: usize>(x: &'static str, ts: [Type; N]) -> Type {
        Type::Path(name_path(x, ts, []))
    }

    pub fn ty(x: &'static str) -> Type {
        Type::Path(name_path(x, [], []))
    }

    pub fn ty_assoc<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
        x1: &'static str,
    ) -> Type {
        Type::Path(path([(x0, vec(ts), name_map(xts)), (x1, vec([]), map([]))]))
    }

    pub fn bound<const N: usize, const M: usize>(
        x: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
    ) -> Trait {
        Trait::Path(span(), name_path(x, ts, xts))
    }

    pub fn head<const N: usize>(x: &'static str, ts: [Type; N]) -> Trait {
        Trait::Path(span(), name_path(x, ts, []))
    }

    pub fn path<const N: usize>(segments: [(&'static str, Vec<Type>, Map<Name, Type>); N]) -> Path {
        Path::new(map3(segments, segment))
    }

    pub fn segment(x: &'static str, ts: Vec<Type>, xts: Map<Name, Type>) -> Segment {
        Segment::new(span(), name(x), ts, xts)
    }

    pub fn name_path<const N: usize, const M: usize>(
        x: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
    ) -> Path {
        let tys = vec(ts);
        let xts = name_map(xts);
        Path::new(vec([segment(x, tys, xts)]))
    }
}

pub fn expr_assoc<const N: usize>(b: Trait, x1: &'static str, ts1: [Type; N]) -> Expr {
    Expr::TraitMethod(span(), Type::Unknown, b, name(x1), vec(ts1))
}

pub fn expr_assign(e0: Expr, e1: Expr) -> Expr {
    Expr::Assign(span(), Type::Unknown, Rc::new(e0), Rc::new(e1))
}

// pub fn stmt_mod<const N: usize>(x: &'static str, ss: [Stmt; N]) -> Stmt {
//     StmtMod::new(span(), name(x), Vec(ss)).into()
// }
//
// pub fn stmt_use<const N: usize>(x: [&'static str; N]) -> Stmt {
//     StmtUse::new(span(), Path::new(map(x, name))).into()
// }

pub fn stmt_var(x: &'static str, t: Type, e: Expr) -> Stmt {
    StmtVar::new(span(), name(x), t, e).into()
}

pub fn type_body(t: Type) -> StmtTypeBody {
    StmtTypeBody::UserDefined(t)
}

pub fn expr_record<const N: usize>(xes: [(&'static str, Expr); N]) -> Expr {
    Expr::Record(span(), Type::Unknown, name_map(xes))
}

pub fn stmt_type<const N: usize>(
    x: &'static str,
    generics: [&'static str; N],
    t: impl Into<StmtTypeBody>,
) -> Stmt {
    StmtType::new(span(), name(x), app(generics, name), t.into()).into()
}

pub fn stmt_expr(e: Expr) -> Stmt {
    Stmt::Expr(Rc::new(e))
}

pub fn expr_unit() -> Expr {
    Expr::Tuple(span(), Type::Unknown, vec([]))
}

pub fn expr_struct<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xes: [(&'static str, Expr); M],
) -> Expr {
    Expr::Struct(span(), Type::Unknown, name(x), vec(ts), name_map(xes))
}

pub fn expr_tuple<const N: usize>(es: [Expr; N]) -> Expr {
    Expr::Tuple(span(), Type::Unknown, vec(es))
}

pub fn expr_array<const N: usize>(es: [Expr; N]) -> Expr {
    Expr::Array(span(), Type::Unknown, vec(es))
}

pub fn expr_index(e1: Expr, i: Index) -> Expr {
    Expr::Index(span(), Type::Unknown, Rc::new(e1), i)
}

pub fn index(i: &'static str) -> Index {
    Index::new(span(), i.parse().unwrap())
}

pub fn expr_field(e: Expr, x: &'static str) -> Expr {
    Expr::Field(span(), Type::Unknown, Rc::new(e), name(x))
}

pub fn expr_enum<const N: usize>(
    x0: &'static str,
    ts: [Type; N],
    x1: &'static str,
    e: Expr,
) -> Expr {
    Expr::Enum(
        span(),
        Type::Unknown,
        name(x0),
        vec(ts),
        name(x1),
        Rc::new(e),
    )
}

pub fn expr_body(e: Expr) -> StmtDefBody {
    StmtDefBody::UserDefined(e)
}

pub fn stmt_def<const N: usize, const M: usize, const K: usize>(
    x: impl Into<Name>,
    gs: [&'static str; N],
    ps: [(&'static str, Type); K],
    t: Type,
    qs: [Trait; M],
    b: impl Into<StmtDefBody>,
) -> Stmt {
    StmtDef::new(
        span(),
        name(x),
        app(gs, name),
        params(ps),
        t,
        vec(qs),
        b.into(),
    )
    .into()
}

pub fn stmt_err() -> Stmt {
    Stmt::Err(span())
}

fn params<const N: usize>(xts: [(&'static str, Type); N]) -> Map<Name, Type> {
    xts.into_iter().map(|(x, t)| (name(x), t)).collect()
}

pub fn tr_def<const N: usize, const M: usize, const K: usize>(
    x: &'static str,
    gs: [&'static str; N],
    xts: [(&'static str, Type); K],
    t: Type,
    qs: [Trait; M],
) -> StmtTraitDef {
    StmtTraitDef::new(span(), name(x), app(gs, name), params(xts), t, vec(qs))
}

pub fn tr_type<const N: usize>(x: &'static str, gs: [&'static str; N]) -> StmtTraitType {
    StmtTraitType::new(span(), name(x), app(gs, name))
}

pub fn stmt_struct<const N: usize, const M: usize>(
    x: &'static str,
    gs: [&'static str; N],
    xts: [(&'static str, Type); M],
) -> Stmt {
    StmtStruct::new(span(), name(x), app(gs, name), name_map(xts)).into()
}

pub fn stmt_enum<const N: usize, const M: usize>(
    x: &'static str,
    gs: [&'static str; N],
    xts: [(&'static str, Type); M],
) -> Stmt {
    StmtEnum::new(span(), name(x), app(gs, name), name_map(xts)).into()
}

pub fn expr_call<const N: usize>(e: Expr, es: [Expr; N]) -> Expr {
    Expr::Call(span(), Type::Unknown, Rc::new(e), vec(es))
}

pub fn expr_unresolved<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
    Expr::Unresolved(span(), Type::Unknown, name(x), vec(ts))
}

pub fn expr_call_direct<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    es: [Expr; M],
) -> Expr {
    expr_call(expr_def(x, ts), es)
}

pub fn expr_var(x: &'static str) -> Expr {
    Expr::Var(span(), Type::Unknown, name(x))
}

pub fn expr_annotate(e: Expr, t: Type) -> Expr {
    Expr::Annotate(span(), t, Rc::new(e))
}

pub fn pat_annotate(p: Pat, t: Type) -> Pat {
    Pat::Annotate(span(), t, Rc::new(p))
}

pub fn pat_var(x: &'static str) -> Pat {
    Pat::Var(span(), Type::Unknown, name(x))
}

pub fn pat_int(v: &'static str) -> Pat {
    Pat::Int(span(), Type::Unknown, v.into())
}

pub fn pat_string(s: &'static str) -> Pat {
    Pat::String(span(), Type::Unknown, s.into())
}

pub fn pat_unit() -> Pat {
    Pat::Tuple(span(), Type::Unknown, vec([]))
}

pub fn pat_bool(b: bool) -> Pat {
    Pat::Bool(span(), Type::Unknown, b)
}

pub fn pat_char(c: char) -> Pat {
    Pat::Char(span(), Type::Unknown, c)
}

pub fn pat_wild() -> Pat {
    Pat::Wildcard(span(), Type::Unknown)
}

pub fn pat_tuple<const N: usize>(ps: [Pat; N]) -> Pat {
    Pat::Tuple(span(), Type::Unknown, vec(ps))
}

pub fn pat_enum<const N: usize>(x0: &'static str, ts: [Type; N], x1: &'static str, p: Pat) -> Pat {
    Pat::Enum(
        span(),
        Type::Unknown,
        name(x0),
        vec(ts),
        name(x1),
        Rc::new(p),
    )
}

pub fn pat_struct<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xps: [(&'static str, Pat); M],
) -> Pat {
    Pat::Struct(span(), Type::Unknown, name(x), vec(ts), name_map(xps))
}

pub fn pat_annot(t: Type, p: Pat) -> Pat {
    p.with_type(t)
}

pub fn arms<const N: usize>(arms: [(Pat, Expr); N]) -> Map<Pat, Expr> {
    arms.into_iter().map(|(p, e)| (p, e)).collect()
}

pub fn expr_match<const N: usize>(e: Expr, pes: [(Pat, Expr); N]) -> Expr {
    Expr::Match(span(), Type::Unknown, Rc::new(e), arms(pes))
}

pub fn expr_if(e0: Expr, b0: Block) -> Expr {
    Expr::IfElse(
        span(),
        Type::Unknown,
        Rc::new(e0),
        b0,
        block([], expr_unit()),
    )
}

pub fn expr_if_else(e0: Expr, b0: Block, b1: Block) -> Expr {
    Expr::IfElse(span(), Type::Unknown, Rc::new(e0), b0, b1)
}

pub fn expr_def<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
    Expr::Def(span(), Type::Unknown, name(x), vec(ts))
}

pub fn expr_int(i: &'static str) -> Expr {
    Expr::Int(span(), Type::Unknown, i.into())
}

pub fn expr_float(f: &'static str) -> Expr {
    Expr::Float(span(), Type::Unknown, f.into())
}

pub fn expr_bool(b: bool) -> Expr {
    Expr::Bool(span(), Type::Unknown, b)
}

pub fn expr_string(s: &'static str) -> Expr {
    Expr::String(span(), Type::Unknown, s.into())
}

pub fn expr_char(c: char) -> Expr {
    Expr::Char(span(), Type::Unknown, c)
}

pub fn block<const N: usize>(ss: [Stmt; N], e: Expr) -> Block {
    Block::new(span(), vec(ss), e)
}

pub fn expr_block<const N: usize>(ss: [Stmt; N], e: Expr) -> Expr {
    Expr::Block(span(), Type::Unknown, block(ss, e))
}

pub fn spanned_expr_block<const N: usize>(span: Span, ss: [Stmt; N], e: Expr) -> Expr {
    Expr::Block(span, Type::Unknown, block(ss, e))
}

pub fn expr_err() -> Expr {
    Expr::Err(span(), Type::Unknown)
}

pub fn expr_fun<const N: usize>(ps: [&'static str; N], e: Expr) -> Expr {
    Expr::Fun(
        span(),
        Type::Unknown,
        app(ps, |s| (name(s), Type::Unknown)).into(),
        Type::Unknown,
        Rc::new(e),
    )
}

fn param((x, t): (&'static str, Type)) -> (Name, Type) {
    (name(x), t)
}

pub fn expr_fun_typed<const N: usize>(ps: [(&'static str, Type); N], t: Type, e: Expr) -> Expr {
    Expr::Fun(span(), Type::Unknown, app(ps, param).into(), t, Rc::new(e))
}

pub fn expr_return(e: Expr) -> Expr {
    Expr::Return(span(), Type::Unknown, Rc::new(e))
}

pub fn expr_continue() -> Expr {
    Expr::Continue(span(), Type::Unknown)
}

pub fn expr_break() -> Expr {
    Expr::Break(span(), Type::Unknown)
}

pub fn expr_query<const N: usize>(x: &'static str, e: Expr, qs: [Query; N]) -> Expr {
    Expr::Query(span(), Type::Unknown, name(x), Rc::new(e), vec(qs))
}

pub fn expr_query_into<const N: usize, const M: usize, const K: usize>(
    x0: &'static str,
    e: Expr,
    qs: [Query; N],
    x1: &'static str,
    ts: [Type; M],
    es: [Expr; K],
) -> Expr {
    Expr::QueryInto(
        span(),
        Type::Unknown,
        name(x0),
        Rc::new(e),
        vec(qs),
        name(x1),
        vec(ts),
        vec(es),
    )
}

pub fn query_join_on(x: &'static str, e0: Expr, e1: Expr) -> Query {
    Query::JoinOn(span(), name(x), Rc::new(e0), Rc::new(e1))
}

pub fn query_join_over_on(x: &'static str, e0: Expr, e1: Expr, e2: Expr) -> Query {
    Query::JoinOverOn(span(), name(x), Rc::new(e0), Rc::new(e1), Rc::new(e2))
}

pub fn query_select<const N: usize>(xes: [(&'static str, Expr); N]) -> Query {
    Query::Select(span(), name_map(xes))
}

pub fn query_where(e: Expr) -> Query {
    Query::Where(span(), Rc::new(e))
}

pub fn query_from(x: &'static str, e: Expr) -> Query {
    Query::From(span(), name(x), Rc::new(e))
}

pub fn query_var(x: &'static str, e: Expr) -> Query {
    Query::Var(span(), name(x), Rc::new(e))
}

pub fn query_join(x: &'static str, e0: Expr, e1: Expr, e2: Expr) -> Query {
    Query::JoinOn(span(), name(x), Rc::new(e0), Rc::new(e1))
}

pub fn query_join_over(x: &'static str, e0: Expr, e1: Expr, e2: Expr, e3: Expr) -> Query {
    Query::JoinOverOn(span(), name(x), Rc::new(e0), Rc::new(e1), Rc::new(e2))
}

pub fn query_group_over_compute<const N: usize>(
    x: &'static str,
    e0: Expr,
    e1: Expr,
    aggs: [Aggr; N],
) -> Query {
    Query::GroupOverCompute(span(), name(x), Rc::new(e0), Rc::new(e1), vec(aggs))
}

pub fn query_over_compute<const N: usize>(e: Expr, aggs: [Aggr; N]) -> Query {
    Query::OverCompute(span(), Rc::new(e), vec(aggs))
}

pub fn aggr(x: &'static str, e0: Expr, e1: Expr) -> Aggr {
    Aggr::new(name(x), e0, e1)
}

pub fn expr_while(e: Expr, b: Block) -> Expr {
    Expr::While(span(), Type::Unknown, Rc::new(e), b)
}

pub fn span() -> Span {
    Span::default()
}

pub fn seg<const N: usize>(x: &'static str, ts: [Type; N]) -> (Name, Vec<Type>) {
    (name(x), vec(ts))
}

pub mod types {
    use compiler::ast::Type;

    use super::ty;
    use super::ty_con;

    pub fn ty_i32() -> Type {
        ty("i32")
    }

    pub fn ty_i64() -> Type {
        ty("i64")
    }

    pub fn ty_bool() -> Type {
        ty("bool")
    }

    pub fn ty_string() -> Type {
        ty("String")
    }

    pub fn ty_vec(t: Type) -> Type {
        ty_con("Vec", [t])
    }

    pub fn ty_option(t: Type) -> Type {
        ty_con("Option", [t])
    }

    pub fn ty_stream(t: Type) -> Type {
        ty_con("Stream", [t])
    }
}

pub mod traits {
    use std::rc::Rc;

    use compiler::ast::Stmt;
    use compiler::ast::StmtImpl;
    use compiler::ast::Trait;
    use compiler::ast::Type;

    use super::app;
    use super::name;
    use super::span;
    use super::stmt_type;
    use super::trait_bound;
    use super::ty_assoc;
    use super::vec;

    pub fn imp<const N: usize, const M: usize, const K: usize, const L: usize>(
        gs: [&'static str; N],
        head: Trait,
        where_clause: [Trait; M],
        defs: [Stmt; K],
        types: [Stmt; L],
    ) -> StmtImpl {
        StmtImpl::new(
            span(),
            app(gs, name),
            head,
            vec(where_clause),
            app(defs, |s| Rc::new(s.as_def().unwrap().clone())),
            app(types, |s| Rc::new(s.as_type().unwrap().clone())),
        )
    }

    pub fn impls<const N: usize>(impls: [StmtImpl; N]) -> Vec<Rc<StmtImpl>> {
        impls.into_iter().map(Rc::new).collect()
    }

    pub fn impl_clone<const N: usize, const M: usize>(
        gs: [&'static str; N],
        t: Type,
        where_clause: [Trait; M],
    ) -> StmtImpl {
        imp(gs, tr_clone(t), where_clause, [], [])
    }

    pub fn tr_clone(t: Type) -> Trait {
        trait_bound("Clone", [t], [])
    }

    pub fn impl_iterator<const N: usize, const M: usize>(
        gs: [&'static str; N],
        t: Type,
        t1: Type,
        where_clause: [Trait; M],
    ) -> StmtImpl {
        imp(
            gs,
            tr_iterator(t),
            where_clause,
            [],
            [stmt_type("Item", [], t1)],
        )
    }

    pub fn tr_iterator(t0: Type) -> Trait {
        trait_bound("Iterator", [t0], [])
    }

    pub fn ty_iterator_item(t: Type) -> Type {
        ty_assoc("Iterator", [t], [], "Item", [])
    }

    pub fn impl_add<const N: usize, const M: usize>(
        gs: [&'static str; N],
        ts: [Type; 2],
        t: Type,
        where_clause: [Trait; M],
    ) -> StmtImpl {
        imp(
            gs,
            tr_add(ts),
            where_clause,
            [],
            [stmt_type("Output", [], t)],
        )
    }

    pub fn tr_add(ts: [Type; 2]) -> Trait {
        trait_bound("Add", ts, [])
    }

    pub fn ty_add_output(ts: [Type; 2]) -> Type {
        ty_assoc("Add", ts, [], "Output", [])
    }

    pub fn impl_into_iterator<const N: usize, const M: usize>(
        gs: [&'static str; N],
        self_ty: Type,
        item_ty: Type,
        intoiter_ty: Type,
        where_clause: [Trait; M],
    ) -> StmtImpl {
        imp(
            gs,
            tr_into_iterator(self_ty),
            where_clause,
            [],
            [
                stmt_type("Item", [], item_ty),
                stmt_type("IntoIter", [], intoiter_ty),
            ],
        )
    }

    pub fn tr_into_iterator(t0: Type) -> Trait {
        trait_bound("IntoIterator", [t0], [])
    }

    pub fn ty_intoiterator_item(t0: Type) -> Type {
        ty_assoc("IntoIterator", [t0], [], "Item", [])
    }

    pub fn ty_into_iterator_into_iter(t0: Type) -> Type {
        ty_assoc("IntoIterator", [t0], [], "IntoIter", [])
    }
}

pub fn parse<T>(
    comp: &mut Compiler,
    name: impl ToString,
    input: &str,
    f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>) -> T,
) -> Result<T, Recovered<T>> {
    let input: Rc<str> = Rc::from(input);
    let id = comp.sources.add(name, input.clone());
    let mut lexer = Lexer::new(id, input.as_ref());
    let mut parser = Parser::new(&input, &mut lexer);
    let result = f(&mut parser);
    comp.add_report(&mut parser.report);
    comp.add_report(&mut lexer.report);
    comp.recover(result)
}

pub fn desugar(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.parse(name, input, |parser| parser.parse(Parser::program).unwrap())?;
    let result = comp.desugar.desugar(&program);
    comp.recover(result)
}

pub fn resolve(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.desugar(name, input)?;
    let result = comp.resolve.resolve(&program);
    comp.report.merge(&mut comp.resolve.report);
    comp.recover(result)
}

pub fn flatten(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.resolve(name, input)?;
    let program = comp.flatten.flatten(&program);
    comp.recover(program)
}

pub fn lift(comp: &mut Compiler, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
    let program = comp.resolve(name, input)?;
    let program = comp.lift.lift(&program);
    comp.recover(program)
}

pub fn infer(comp: &mut Compiler, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
    let result = comp.resolve(name, input)?;
    let result = comp.infer.infer(&result);
    comp.report.merge(&mut comp.infer.report);
    comp.recover(result)
}

pub fn monomorphise(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let result = comp.infer(name, input)?;
    let result = comp.monomorphise.monomorphise(&result);
    comp.recover(result)
}

pub fn interpret(comp: &mut Compiler, name: &str, input: &str) -> Result<Value, Recovered<Value>> {
    let mut result = comp.infer(name, input).unwrap();
    let stmt = result.stmts.pop().unwrap();
    let expr = stmt.as_expr().unwrap();
    comp.interpret.interpret(&result);
    let value = comp.interpret.expr(expr);
    comp.report.merge(&mut comp.interpret.report);
    comp.recover(value)
}

pub fn parse_expr(input: &str) -> Result<Expr, Recovered<Expr>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::expr).unwrap())
}

pub fn parse_stmt(input: &str) -> Result<Stmt, Recovered<Stmt>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::stmt).unwrap())
}

pub fn parse_type(input: &str) -> Result<Type, Recovered<Type>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::ty).unwrap())
}

pub fn parse_pat(input: &str) -> Result<Pat, Recovered<Pat>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::pat).unwrap())
}

pub fn parse_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::program).unwrap())
}

pub fn desugar_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().desugar("test", input)
}

pub fn querycomp_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().querycomp("test", input)
}

pub fn resolve_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().resolve("test", input)
}

pub fn flatten_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().flatten("test", input)
}

pub fn lift_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().lift("test", input)
}

pub fn infer_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().infer("test", input)
}

pub fn monomorphise_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().monomorphise("test", input)
}

pub fn interpret_program(input: &str) -> Result<Value, Recovered<Value>> {
    Compiler::default().init().interpret("test", input)
}

macro_rules! parse_program {
    ($code:literal) => {
        common::parse_program(indoc::indoc!($code))
    };
}

macro_rules! parse_expr {
    ($code:literal) => {
        common::parse_expr(indoc::indoc!($code))
    };
    ($code:expr) => {
        common::parse_stmt($code)
    };
}

macro_rules! parse_type {
    ($code:literal) => {
        common::parse_type(indoc::indoc!($code))
    };
}

macro_rules! parse_stmt {
    ($code:literal) => {
        common::parse_stmt(indoc::indoc!($code))
    };
}

macro_rules! parse_pat {
    ($code:literal) => {
        common::parse_pat(indoc::indoc!($code))
    };
}

macro_rules! desugar_program {
    ($code:literal) => {
        common::desugar_program(indoc::indoc!($code))
    };
}

macro_rules! resolve_program {
    ($code:literal) => {
        common::resolve_program(indoc::indoc!($code))
    };
}

macro_rules! lift_program {
    ($code:literal) => {
        common::lift_program(indoc::indoc!($code))
    };
}

macro_rules! infer_program {
    ($code:literal) => {
        common::infer_program(indoc::indoc!($code))
    };
}

macro_rules! monomorphise_program {
    ($code:literal) => {
        common::monomorphise_program(indoc::indoc!($code))
    };
}

macro_rules! interpret_program {
    ($code:literal) => {
        common::interpret_program(indoc::indoc!($code))
    };
}

macro_rules! flatten_program {
    ($code:literal) => {
        common::flatten_program(indoc::indoc!($code))
    };
}

macro_rules! querycomp_program {
    ($code:literal) => {
        common::querycomp_program(indoc::indoc!($code))
    };
}
