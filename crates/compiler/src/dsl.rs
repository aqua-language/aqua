use std::rc::Rc;

use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Query;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::TraitBound;
use crate::ast::TraitDef;
use crate::ast::TraitType;
use crate::ast::Type;

use crate::lexer::Span;

#[macro_export]
macro_rules! check {
    ($a:expr, $b:expr) => {
        assert!($a == $b, "\n(a) {}\n\n(b) {}", $a, $b);
    };
    (@debug; $a:expr, $b:expr) => {
        fn diff(a: impl std::fmt::Debug, b: impl std::fmt::Debug) -> String {
            let a = format!("{:#?}", a);
            let b = format!("{:#?}", b);
            let mut output = String::new();
            let diff = similar::TextDiff::from_lines(&a, &b);
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    similar::ChangeTag::Delete => "A",
                    similar::ChangeTag::Insert => "B",
                    similar::ChangeTag::Equal => " ",
                };
                output.push_str(&format!("{}{}", sign, change));
            }
            output
        }
        assert!($a == $b, "\n{}", diff($a, $b));
    };
    ($a:expr, $b:expr, $b_msg:literal) => {{
        fn diff(a: impl std::fmt::Debug, b: impl std::fmt::Debug) -> String {
            let a = format!("{:#?}", a);
            let b = format!("{:#?}", b);
            let mut output = String::new();
            let diff = similar::TextDiff::from_lines(&a, &b);
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    similar::ChangeTag::Delete => "-",
                    similar::ChangeTag::Insert => "+",
                    similar::ChangeTag::Equal => " ",
                };
                output.push_str(&format!("{}{}", sign, change));
            }
            output
        }
        let b_msg = indoc::indoc!($b_msg);
        assert!($a.val == $b, "{}", diff($a.val, $b));
        assert!($a.msg == b_msg, "{}", diff($a.msg, b_msg));
    }};
}

pub fn trim(s: &str) -> String {
    // Trim space right before \n on each line
    s.trim_end()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn program<const N: usize>(ss: [Stmt; N]) -> Program {
    Program::new(vec(ss))
}

pub fn stmt_trait<const N: usize, const M: usize, const K: usize, const L: usize>(
    x: &'static str,
    gs: [&'static str; N],
    bounds: [Bound; M],
    defs: [TraitDef; K],
    types: [TraitType; L],
) -> Stmt {
    StmtTrait::new(
        span(),
        name(x),
        app(gs, name),
        vec(bounds),
        vec(defs),
        vec(types),
    )
    .into()
}

fn vec<const N: usize, T: Clone>(xs: [T; N]) -> Vec<T> {
    xs.into_iter().collect()
}

fn map<const N: usize, K, V>(kvs: [(K, V); N]) -> Map<K, V> {
    Map::from(kvs.into_iter().collect::<Vec<_>>())
}

fn name_map<const N: usize, V>(xs: [(&'static str, V); N]) -> Map<Name, V> {
    Map::from(
        xs.into_iter()
            .map(|(x, v)| (name(x), v))
            .collect::<Vec<_>>(),
    )
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
    head: Bound,
    body: [Bound; M],
    defs: [Stmt; K],
    types: [Stmt; L],
) -> Stmt {
    StmtImpl::new(
        span(),
        app(gs, name),
        head,
        vec(body),
        app(defs, |s| {
            if let Stmt::Def(s) = s {
                s
            } else {
                unreachable!()
            }
        }),
        app(types, |s| {
            if let Stmt::Type(s) = s {
                s
            } else {
                unreachable!()
            }
        }),
    )
    .into()
}

// Create a Fact:
// impl<{gs}> {head}
pub fn fact<const N: usize, const M: usize, const K: usize>(
    gs: [&'static str; N],
    head: Bound,
    defs: [StmtDef; M],
    types: [StmtType; K],
) -> StmtImpl {
    StmtImpl::new(span(), app(gs, name), head, vec([]), vec(defs), vec(types))
}

// Create a predicate:
// {t0}: {name}<{t1},..,{tn},{assoc}>
pub fn bound<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xts: [(&'static str, Type); M],
) -> Bound {
    Bound::Trait(span(), trait_bound(x, ts, xts))
}

pub fn trait_bound<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xts: [(&'static str, Type); M],
) -> TraitBound {
    TraitBound::new(name(x), vec(ts), name_map(xts))
}

pub fn bound_err() -> Bound {
    Bound::Err(span())
}

pub fn ty_alias<const N: usize>(x: &'static str, ts: [Type; N]) -> Type {
    Type::Alias(name(x), vec(ts))
}

pub fn ty(x: &'static str) -> Type {
    debug_assert!(!x.starts_with('?'));
    ty_con(x, [])
}

pub fn ty_err() -> Type {
    Type::Err
}

pub fn ty_hole() -> Type {
    Type::Hole
}

pub fn name(x: &'static str) -> Name {
    Name::from(x)
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

pub fn ty_var(x: &'static str) -> Type {
    debug_assert!(x.starts_with('?'));
    Type::Var(name(x))
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

pub fn pat_record<const N: usize>(xps: [(&'static str, Pat); N]) -> Pat {
    Pat::Record(span(), ty_hole(), name_map(xps))
}

pub mod unresolved {
    use std::rc::Rc;

    use super::expr_assign;
    use super::map;
    use super::map3;
    use super::name_map;
    use super::vec;
    use crate::ast::Bound;
    use crate::ast::Expr;
    use crate::ast::Map;
    use crate::ast::Name;
    use crate::ast::Pat;
    use crate::ast::Path;
    use crate::ast::Segment;
    use crate::ast::Type;
    use crate::ast::UnresolvedPatField;

    use super::app;
    use super::expr_call;
    use super::name;
    use super::span;
    use super::ty_hole;

    // x
    pub fn expr_var(x: &'static str) -> Expr {
        Expr::Unresolved(
            span(),
            ty_hole(),
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
            ty_hole(),
            Rc::new(name_expr(x, ts)),
            app(xes, |(x, e)| expr_assign(expr_var(x), e)),
        )
    }

    pub fn expr_def<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
        name_expr(x, ts)
    }

    fn name_expr<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
        Expr::Unresolved(span(), ty_hole(), name_path(x, ts, []))
    }

    pub fn expr_variant<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        x1: &'static str,
        es: [Expr; M],
    ) -> Expr {
        Expr::Call(
            span(),
            ty_hole(),
            Rc::new(Expr::Unresolved(
                span(),
                ty_hole(),
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
        Expr::Unresolved(
            span(),
            ty_hole(),
            path([(x0, vec(ts), map([])), (x1, vec([]), map([]))]),
        )
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
        Expr::Unresolved(
            span(),
            ty_hole(),
            path([(x0, vec(ts), name_map(xts)), (x1, vec([]), map([]))]),
        )
    }

    pub fn expr_binop(x0: &'static str, x1: &'static str, e0: Expr, e1: Expr) -> Expr {
        Expr::Call(
            span(),
            ty_hole(),
            Rc::new(expr_assoc(x0, [], [], x1)),
            vec([e0, e1]),
        )
    }

    pub fn expr_enum(x0: &'static str, x1: &'static str) -> Expr {
        Expr::Unresolved(
            span(),
            ty_hole(),
            Path::new(vec([
                segment(x0, vec([]), map([])),
                segment(x1, vec([]), map([])),
            ])),
        )
    }

    pub fn expr_unop(x0: &'static str, x1: &'static str, e: Expr) -> Expr {
        Expr::Call(
            span(),
            ty_hole(),
            Rc::new(expr_assoc(x0, [], [], x1)),
            vec([e]),
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

    pub fn pat_var(x: &'static str) -> Pat {
        Pat::Unresolved(span(), ty_hole(), name_path(x, [], []), None)
    }

    pub fn pat_enum<const N: usize>(
        x0: &'static str,
        ts: [Type; N],
        x1: &'static str,
        p: Pat,
    ) -> Pat {
        Pat::Unresolved(
            span(),
            ty_hole(),
            path([(x0, vec(ts), map([])), (x1, vec([]), map([]))]),
            Some(vec([UnresolvedPatField::Unnamed(p)])),
        )
    }

    pub fn pat_struct<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        xps: [(&'static str, Pat); M],
    ) -> Pat {
        Pat::Unresolved(
            span(),
            ty_hole(),
            path([(x0, vec(ts), map([]))]),
            Some(app(xps, |(x, p)| UnresolvedPatField::Named(name(x), p))),
        )
    }

    pub fn pat_unit_struct<const N: usize>(x: &'static str, ts: [Type; N]) -> Pat {
        Pat::Unresolved(span(), ty_hole(), path([(x, vec(ts), map([]))]), None)
    }

    pub fn ty_con<const N: usize>(x: &'static str, ts: [Type; N]) -> Type {
        Type::Unresolved(name_path(x, ts, []))
    }

    pub fn ty(x: &'static str) -> Type {
        Type::Unresolved(name_path(x, [], []))
    }

    pub fn ty_assoc<const N: usize, const M: usize>(
        x0: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
        x1: &'static str,
    ) -> Type {
        Type::Unresolved(path([(x0, vec(ts), name_map(xts)), (x1, vec([]), map([]))]))
    }

    pub fn bound<const N: usize, const M: usize>(
        x: &'static str,
        ts: [Type; N],
        xts: [(&'static str, Type); M],
    ) -> Bound {
        Bound::Unresolved(span(), name_path(x, ts, xts))
    }

    pub fn head<const N: usize>(x: &'static str, ts: [Type; N]) -> Bound {
        Bound::Unresolved(span(), name_path(x, ts, []))
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

pub fn expr_assoc<const N: usize, const M: usize>(
    x0: &'static str,
    ts0: [Type; N],
    x1: &'static str,
    ts1: [Type; M],
) -> Expr {
    let b = trait_bound(x0, ts0, []);
    Expr::Assoc(span(), ty_hole(), b, name(x1), vec(ts1))
}

pub fn expr_assign(e0: Expr, e1: Expr) -> Expr {
    Expr::Assign(span(), ty_hole(), Rc::new(e0), Rc::new(e1))
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

pub fn stmt_type<const N: usize>(
    x: &'static str,
    generics: [&'static str; N],
    t: impl Into<StmtTypeBody>,
) -> Stmt {
    StmtType::new(span(), name(x), app(generics, name), t.into()).into()
}

pub fn stmt_expr(e: Expr) -> Stmt {
    Stmt::Expr(e)
}

pub fn expr_unit() -> Expr {
    Expr::Tuple(span(), ty_hole(), vec([]))
}

pub fn ty_unit() -> Type {
    Type::Tuple(vec([]))
}

pub fn expr_struct<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xes: [(&'static str, Expr); M],
) -> Expr {
    Expr::Struct(span(), ty_hole(), name(x), vec(ts), name_map(xes))
}

pub fn expr_tuple<const N: usize>(es: [Expr; N]) -> Expr {
    Expr::Tuple(span(), ty_hole(), vec(es))
}

pub fn expr_array<const N: usize>(es: [Expr; N]) -> Expr {
    Expr::Array(span(), ty_hole(), vec(es))
}

pub fn expr_index(e1: Expr, i: Index) -> Expr {
    Expr::Index(span(), ty_hole(), Rc::new(e1), i)
}

pub fn index(i: &'static str) -> Index {
    Index::new(span(), i.parse().unwrap())
}

pub fn expr_field(e: Expr, x: &'static str) -> Expr {
    Expr::Field(span(), ty_hole(), Rc::new(e), name(x))
}

pub fn expr_enum<const N: usize>(
    x0: &'static str,
    ts: [Type; N],
    x1: &'static str,
    e: Expr,
) -> Expr {
    Expr::Enum(span(), ty_hole(), name(x0), vec(ts), name(x1), Rc::new(e))
}

pub fn expr_body(e: Expr) -> StmtDefBody {
    StmtDefBody::UserDefined(e)
}

pub fn stmt_def<const N: usize, const M: usize, const K: usize>(
    x: &'static str,
    gs: [&'static str; N],
    ps: [(&'static str, Type); K],
    t: Type,
    qs: [Bound; M],
    b: impl Into<StmtDefBody>,
) -> Stmt {
    StmtDef::new(
        span(),
        name(x),
        app(gs, name),
        app(ps, |(s, t)| Param::new(span(), name(s), t)),
        t,
        vec(qs),
        b.into(),
    )
    .into()
}

pub fn stmt_err() -> Stmt {
    Stmt::Err(span())
}

pub fn tr_def<const N: usize, const M: usize, const K: usize>(
    x: &'static str,
    gs: [&'static str; N],
    xts: [(&'static str, Type); K],
    t: Type,
    qs: [Bound; M],
) -> TraitDef {
    TraitDef::new(span(), name(x), app(gs, name), app(xts, param), t, vec(qs))
}

pub fn tr_type<const N: usize>(x: &'static str, gs: [&'static str; N]) -> TraitType {
    TraitType::new(span(), name(x), app(gs, name))
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
    Expr::Call(span(), ty_hole(), Rc::new(e), vec(es))
}

pub fn expr_and(e0: Expr, e1: Expr) -> Expr {
    Expr::Match(
        span(),
        ty_hole(),
        Rc::new(e0),
        arms([(pat_bool(true), e1), (pat_wild(), expr_bool(false))]),
    )
}

pub fn expr_or(e0: Expr, e1: Expr) -> Expr {
    Expr::Match(
        span(),
        ty_hole(),
        Rc::new(e0),
        arms([(pat_bool(true), expr_bool(true)), (pat_wild(), e1)]),
    )
}

pub fn expr_var(x: &'static str) -> Expr {
    Expr::Var(span(), ty_hole(), name(x))
}

pub fn pat_var(x: &'static str) -> Pat {
    Pat::Var(span(), ty_hole(), name(x))
}

pub fn pat_int(i: &'static str) -> Pat {
    Pat::Int(span(), ty_hole(), i.to_owned())
}

pub fn pat_string(s: &'static str) -> Pat {
    Pat::String(span(), ty_hole(), s.to_owned())
}

pub fn pat_unit() -> Pat {
    Pat::Tuple(span(), ty_hole(), vec([]))
}

pub fn pat_bool(b: bool) -> Pat {
    Pat::Bool(span(), ty_hole(), b)
}

pub fn pat_char(c: char) -> Pat {
    Pat::Char(span(), ty_hole(), c)
}

pub fn pat_wild() -> Pat {
    Pat::Wildcard(span(), ty_hole())
}

pub fn pat_tuple<const N: usize>(ps: [Pat; N]) -> Pat {
    Pat::Tuple(span(), ty_hole(), vec(ps))
}

pub fn pat_enum<const N: usize>(x0: &'static str, ts: [Type; N], x1: &'static str, p: Pat) -> Pat {
    Pat::Enum(span(), ty_hole(), name(x0), vec(ts), name(x1), Rc::new(p))
}

pub fn pat_struct<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    xps: [(&'static str, Pat); M],
) -> Pat {
    Pat::Struct(span(), ty_hole(), name(x), vec(ts), name_map(xps))
}

pub fn pat_annot(t: Type, p: Pat) -> Pat {
    p.with_ty(t)
}

pub fn arms<const N: usize>(arms: [(Pat, Expr); N]) -> Vec<Arm> {
    arms.into_iter()
        .map(|(p, e)| Arm::new(span(), p, e))
        .collect()
}

pub fn expr_match<const N: usize>(e: Expr, pes: [(Pat, Expr); N]) -> Expr {
    Expr::Match(span(), ty_hole(), Rc::new(e), arms(pes))
}

pub fn expr_if(e0: Expr, b1: Block) -> Expr {
    Expr::Match(
        span(),
        ty_hole(),
        Rc::new(e0),
        arms([
            (pat_bool(true), Expr::Block(span(), Type::Hole, b1)),
            (pat_wild(), expr_unit()),
        ]),
    )
}

pub fn expr_if_else(e0: Expr, b1: Block, b2: Block) -> Expr {
    Expr::Match(
        span(),
        ty_hole(),
        Rc::new(e0),
        arms([
            (pat_bool(true), Expr::Block(span(), Type::Hole, b1)),
            (pat_wild(), Expr::Block(span(), Type::Hole, b2)),
        ]),
    )
}

pub fn expr_def<const N: usize>(x: &'static str, ts: [Type; N]) -> Expr {
    Expr::Def(span(), ty_hole(), name(x), vec(ts))
}

pub fn expr_int(i: &'static str) -> Expr {
    Expr::Int(span(), ty_hole(), i.to_owned())
}

pub fn expr_float(f: &'static str) -> Expr {
    Expr::Float(span(), ty_hole(), f.to_owned())
}

pub fn expr_bool(b: bool) -> Expr {
    Expr::Bool(span(), ty_hole(), b)
}

pub fn expr_string(s: &'static str) -> Expr {
    Expr::String(span(), ty_hole(), s.to_owned())
}

pub fn expr_char(c: char) -> Expr {
    Expr::Char(span(), ty_hole(), c)
}

pub fn block<const N: usize>(ss: [Stmt; N], e: Expr) -> Block {
    Block::new(span(), vec(ss), e)
}

pub fn expr_block<const N: usize>(ss: [Stmt; N], e: Expr) -> Expr {
    Expr::Block(span(), ty_hole(), block(ss, e))
}

pub fn spanned_expr_block<const N: usize>(span: Span, ss: [Stmt; N], e: Expr) -> Expr {
    Expr::Block(span, ty_hole(), block(ss, e))
}

pub fn expr_err() -> Expr {
    Expr::Err(span(), ty_hole())
}

pub fn expr_fun<const N: usize>(ps: [&'static str; N], e: Expr) -> Expr {
    Expr::Fun(
        span(),
        ty_hole(),
        app(ps, |s| Param::new(span(), name(s), ty_hole())),
        ty_hole(),
        Rc::new(e),
    )
}

fn param((x, t): (&'static str, Type)) -> Param {
    Param::new(span(), name(x), t)
}

pub fn expr_fun_typed<const N: usize>(ps: [(&'static str, Type); N], t: Type, e: Expr) -> Expr {
    Expr::Fun(
        span(),
        ty_hole(),
        app(ps, |(s, t)| Param::new(span(), name(s), t)),
        t,
        Rc::new(e),
    )
}

pub fn expr_return(e: Expr) -> Expr {
    Expr::Return(span(), ty_hole(), Rc::new(e))
}

pub fn expr_continue() -> Expr {
    Expr::Continue(span(), ty_hole())
}

pub fn expr_break() -> Expr {
    Expr::Break(span(), ty_hole())
}

pub fn expr_query<const N: usize>(qs: [Query; N]) -> Expr {
    Expr::Query(span(), ty_hole(), vec(qs))
}

pub fn query_select<const N: usize>(xes: [(&'static str, Expr); N]) -> Query {
    Query::Select(span(), ty_hole(), name_map(xes))
}

pub fn query_where(e: Expr) -> Query {
    Query::Where(span(), ty_hole(), Rc::new(e))
}

pub fn query_from(x: &'static str, e: Expr) -> Query {
    Query::From(span(), ty_hole(), name(x), Rc::new(e))
}

pub fn query_into<const N: usize, const M: usize>(
    x: &'static str,
    ts: [Type; N],
    es: [Expr; M],
) -> Query {
    Query::Into(span(), ty_hole(), name(x), vec(ts), vec(es))
}

pub fn query_var(x: &'static str, e: Expr) -> Query {
    Query::Var(span(), ty_hole(), name(x), Rc::new(e))
}

pub fn query_join(x: &'static str, e0: Expr, e1: Expr) -> Query {
    Query::Join(span(), ty_hole(), name(x), Rc::new(e0), Rc::new(e1))
}

pub fn query_group<const N: usize>(e: Expr, qs: [Query; N]) -> Query {
    Query::Group(span(), ty_hole(), Rc::new(e), vec(qs))
}

pub fn query_over<const N: usize>(e: Expr, qs: [Query; N]) -> Query {
    Query::Over(span(), ty_hole(), Rc::new(e), vec(qs))
}

pub fn query_compute(x: &'static str, e0: Expr, e1: Expr) -> Query {
    Query::Compute(span(), ty_hole(), name(x), Rc::new(e0), Rc::new(e1))
}

pub fn expr_while(e: Expr, b: Block) -> Expr {
    Expr::While(span(), ty_hole(), Rc::new(e), b)
}

pub fn span() -> Span {
    Span::default()
}

pub fn seg<const N: usize>(x: &'static str, ts: [Type; N]) -> (Name, Vec<Type>) {
    (name(x), vec(ts))
}

pub mod types {
    use crate::ast::Type;

    use super::ty;
    use super::ty_con;

    pub fn ty_vec(t: Type) -> Type {
        ty_con("Vec", [t])
    }

    pub fn ty_stream(t: Type) -> Type {
        ty_con("Stream", [t])
    }

    pub fn ty_i32() -> Type {
        ty("i32")
    }

    pub fn ty_i64() -> Type {
        ty("i64")
    }

    pub fn ty_bool() -> Type {
        ty("bool")
    }

    pub fn ty_f32() -> Type {
        ty("f32")
    }
}

pub mod traits {
    use crate::ast::Bound;
    use crate::ast::Stmt;
    use crate::ast::StmtImpl;
    use crate::ast::Type;

    use super::app;
    use super::bound;
    use super::name;
    use super::span;
    use super::stmt_type;
    use super::ty_assoc;
    use super::vec;

    pub fn imp<const N: usize, const M: usize, const K: usize, const L: usize>(
        gs: [&'static str; N],
        head: Bound,
        where_clause: [Bound; M],
        defs: [Stmt; K],
        types: [Stmt; L],
    ) -> StmtImpl {
        StmtImpl::new(
            span(),
            app(gs, name),
            head,
            vec(where_clause),
            app(defs, |s| s.as_def().clone()),
            app(types, |s| s.as_type().clone()),
        )
    }

    pub fn impl_clone<const N: usize, const M: usize>(
        gs: [&'static str; N],
        t: Type,
        where_clause: [Bound; M],
    ) -> StmtImpl {
        imp(gs, tr_clone(t), where_clause, [], [])
    }

    pub fn tr_clone(t: Type) -> Bound {
        bound("Clone", [t], [])
    }

    pub fn impl_iterator<const N: usize, const M: usize>(
        gs: [&'static str; N],
        t: Type,
        t1: Type,
        where_clause: [Bound; M],
    ) -> StmtImpl {
        imp(
            gs,
            tr_iterator(t),
            where_clause,
            [],
            [stmt_type("Item", [], t1)],
        )
    }

    pub fn tr_iterator(t0: Type) -> Bound {
        bound("Iterator", [t0], [])
    }

    pub fn ty_iterator_item(t: Type) -> Type {
        ty_assoc("Iterator", [t], [], "Item", [])
    }

    pub fn impl_add<const N: usize, const M: usize>(
        gs: [&'static str; N],
        ts: [Type; 2],
        t: Type,
        where_clause: [Bound; M],
    ) -> StmtImpl {
        imp(
            gs,
            tr_add(ts),
            where_clause,
            [],
            [stmt_type("Output", [], t)],
        )
    }

    pub fn tr_add(ts: [Type; 2]) -> Bound {
        bound("Add", ts, [])
    }

    pub fn ty_add_output(ts: [Type; 2]) -> Type {
        ty_assoc("Add", ts, [], "Output", [])
    }

    pub fn impl_into_iterator<const N: usize, const M: usize>(
        gs: [&'static str; N],
        self_ty: Type,
        item_ty: Type,
        intoiter_ty: Type,
        where_clause: [Bound; M],
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

    // TODO: Need to be able to apply associated types
    pub fn tr_into_iterator(t0: Type) -> Bound {
        bound("IntoIterator", [t0], [])
    }

    pub fn ty_intoiterator_item(t0: Type) -> Type {
        ty_assoc("IntoIterator", [t0], [], "Item", [])
    }

    pub fn ty_into_iterator_into_iter(t0: Type) -> Type {
        ty_assoc("IntoIterator", [t0], [], "IntoIter", [])
    }
}
