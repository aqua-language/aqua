use std::rc::Rc;

use util::call_filter;
use util::call_flatmap;
use util::call_keyby;
use util::call_map;
use util::call_merge;
use util::call_take;
use util::call_window;
use util::typed_lambda;

use crate::ast::Aggr;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::QueryOp;
use crate::ast::Type;
use crate::diag::Report;
use crate::syntax::span::Span;
use crate::traversal::mapper::Mapper;

use self::util::call;
use self::util::expr_field;
use self::util::expr_var;
use self::util::lambda;
use self::util::record;
use self::util::relation;
use self::util::relation_expr;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Scope>,
    pub report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Program) -> Program {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

#[derive(Debug)]
struct Scope(Vec<Name>);

impl Context {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            report: Report::new(),
        }
    }

    fn bind_relational_var(&mut self, x: Name) {
        self.stack.last_mut().unwrap().0.push(x);
    }

    fn unbind_relational_vars(&mut self) {
        self.stack.last_mut().unwrap().0.clear();
    }

    fn unbind_relational_var(&mut self, x: Name) {
        self.stack.last_mut().unwrap().0.retain(|y| x != *y);
    }

    fn is_relational_var(&self, x: &Name) -> bool {
        self.stack.iter().any(|s| s.0.contains(x))
    }

    fn relational_vars(&self) -> impl Iterator<Item = &Name> {
        self.stack.last().unwrap().0.iter()
    }

    fn query(&mut self, e0: Expr, q: &QueryOp) -> Expr {
        match q {
            QueryOp::Where(s, e1) => self.where_clause(e0, *s, e1),
            QueryOp::Union(s, e1) => self.union_clause(e0, *s, e1),
            QueryOp::Limit(s, e1) => self.limit_clause(e0, *s, e1),
            QueryOp::From(s, x, t, e) => self.from_clause(e0, *s, *x, t, e),
            QueryOp::Select(s, xes) => self.select_clause(e0, *s, xes),
            QueryOp::GroupOverCompute(s, x, e1, e2, aggs) => {
                self.group_over_compute_clause(e0, *s, *x, e1, e2, aggs)
            }
            QueryOp::JoinOn(s, x, t, e1, e2) => self.join_on_clause(e0, *s, *x, t, e1, e2),
            QueryOp::Var(s, x, t, e) => self.var_clause(e0, *s, *x, t, e),
            QueryOp::OverCompute(s, e, aggs) => self.over_compute_clause(e0, *s, e, aggs),
            QueryOp::JoinOverOn(_, _, _, _, _) => todo!(),
            QueryOp::Err(s) => Expr::Err(*s, Type::Unknown),
            QueryOp::Drop(s, x) => self.drop_clause(e0, *s, *x),
        }
    }

    /// from x in e
    /// =>
    /// map(e, x => record(x=x))
    fn first_from_clause(&mut self, s: Span, x: Name, t: Type, e: &Expr) -> Expr {
        let e = self.map_expr(e);
        self.bind_relational_var(x);
        let estruct = Expr::Record(s, Type::Unknown, vec![(x, expr_var(x))].into());
        let elam = Expr::Lambda(
            s,
            Type::Unknown,
            vec![(x, t)].into(),
            Type::Unknown,
            Rc::new(estruct),
        );
        call_map(s, e, elam)
    }

    /// [e0] where e1
    /// =>
    /// filter(e0, r => e1)
    fn where_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e = self.map_expr(e1);
        let elam = lambda(s, [relation(s)], e);
        call_filter(s, e0, elam)
    }

    /// [e0] union e1
    /// =>
    /// merge(e0, e1)
    fn union_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        call_merge(s, e0, e1)
    }

    /// [e0] limit e1
    /// =>
    /// take(e0, r => e1)
    fn limit_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        call_take(s, e0, e1)
    }

    /// [e0] from x in e
    /// =>
    /// flatMap(e0, r => e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn from_clause(&mut self, e0: Expr, s: Span, x: Name, t: &Type, e1: &Expr) -> Expr {
        let e = self.map_expr(e1);
        // record(x=x, x1=r.x1, ..., xn=r.xn)
        let r = Rc::new(relation_expr(s));
        let xt0 = (x, expr_var(x));
        let xts = self
            .relational_vars()
            .map(|x| (*x, expr_field(r.clone(), *x)));
        let xts = xts.collect::<Map<_, _>>();
        let record = record(
            s,
            std::iter::once(xt0).chain(xts).collect::<Vec<_>>().into(),
        );
        self.bind_relational_var(x);
        // x => record(x=x, x1=r.x1, ..., xn=r.xn)
        let elam = typed_lambda(s, [(x, t.clone())], record);
        // e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn))
        let emap = call(s, Name::new(s, "map"), vec![Type::Unknown], vec![e, elam]);
        // flatMap(e0, r => e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn)))
        let elam = lambda(s, [relation(s)], emap);
        call_flatmap(s, e0, elam)
    }

    // [e0] select x1=e1,...,xn=en
    // =>
    // map(e0, r => record(x1=r.x1, ..., xn=r.xn))
    fn select_clause(&mut self, e0: Expr, s: Span, xes: &Map<Name, Expr>) -> Expr {
        let xts = xes
            .iter()
            .map(|(x, e)| (*x, self.map_expr(e)))
            .collect::<Map<_, _>>();
        self.unbind_relational_vars();
        xts.keys().for_each(|x| self.bind_relational_var(*x));
        let record = record(s, xts);
        call_map(s, e0, lambda(s, [relation(s)], record))
    }

    // [e0] drop x
    // =>
    // map(e0, r => record(x1=r.x1, ..., xn=r.xn)) where x is not in {x1, ..., xn}
    fn drop_clause(&mut self, e0: Expr, s: Span, x: Name) -> Expr {
        self.unbind_relational_var(x);
        let r = Rc::new(relation_expr(s));
        let xts = self
            .relational_vars()
            .map(|x| (*x, expr_field(r.clone(), *x)))
            .collect::<Map<_, _>>();
        let record = record(s, xts);
        call_map(s, e0, lambda(s, [relation(s)], record))
    }

    // [e0] group xkey = ekey
    //       over ewin
    //       compute xagg1=efun1 of eattr1,...,xaggn=efunn of eattrn
    // =>
    // e0.keyBy[_](r => ekey)
    //   .window[_](
    //     ewin,
    //     (xkey, r) => record(
    //       xkey = xkey,
    //       xagg1 = efun1(r.map[_](r => eattr1))
    //       ...,
    //       xaggn = efunn(r.map[_](r => eattrn))
    //     )
    //   )
    fn group_over_compute_clause(
        &mut self,
        e0: Expr,
        s: Span,
        xkey: Name,
        ekey: &Expr,
        ewin: &Expr,
        aggs: &[Aggr],
    ) -> Expr {
        // e0.keyBy(r => ekey)
        let ekey = self.map_expr(ekey);
        let ekeyby = call_keyby(s, e0, lambda(s, [relation(s)], ekey));
        // (xkey, r) => record(...)
        let erecord = {
            let xts = self.aggs(s, aggs);
            let xts = std::iter::once((xkey, Expr::Path(s, Type::Unknown, Path::new_name(xkey))))
                .chain(xts)
                .collect::<Map<_, _>>();
            self.unbind_relational_vars();
            xts.keys().for_each(|x| self.bind_relational_var(*x));
            record(s, xts)
        };
        let efun = lambda(s, [xkey, relation(s)], erecord);
        // ekeyby.window(ewin, (xkey, rs) => record(...))
        let ewin = self.map_expr(ewin);
        call_window(s, ekeyby, ewin, efun)
    }

    // x = e1 of e2 [if e3]
    // =>
    // x = e1(r[.filter(r => e3)].map(r => e2))
    fn aggs(&mut self, s: Span, aggs: &[Aggr]) -> Map<Name, Expr> {
        aggs.iter()
            .map(|agg| {
                let x = relation(s);
                if let Some(e2) = &agg.e2 {
                    let v0 = relation_expr(s);
                    let v1 = call_filter(s, v0, lambda(s, [x], self.map_expr(e2)));
                    let v2 = call_map(s, v1, lambda(s, [x], self.map_expr(&agg.e1)));
                    let v3 = call(s, agg.x1, vec![], vec![v2]);
                    (agg.x0, v3)
                } else {
                    let v0 = relation_expr(s);
                    let v1 = call_map(s, v0, lambda(s, [x], self.map_expr(&agg.e1)));
                    let v2 = call(s, agg.x1, vec![], vec![v1]);
                    (agg.x0, v2)
                }
            })
            .collect::<Map<_, _>>()
    }

    // [e0] over e1 compute x1=efun1 of eattr1,...,xn=efunn of eattrn
    // =>
    // e0.window[_](e1,
    //    r =>
    //      record(x1=efun1(r.map[_](r => eattr1)),
    //             ...,
    //             xn=efunn(r.map[_](r => eattrn)))
    fn over_compute_clause(&mut self, e0: Expr, s: Span, e: &Expr, aggs: &[Aggr]) -> Expr {
        let e = self.map_expr(e);
        let xes = self.aggs(s, aggs);
        self.unbind_relational_vars();
        xes.keys().for_each(|x| self.bind_relational_var(*x));
        let record = record(s, xes);
        let elam = lambda(s, [relation(s)], record);
        call_window(s, e0, e, elam)
    }

    // [e0] join x in e1 on e2 == e3
    // =>
    // e0.flatMap[_](r => e1.filter(x => e2 == e3)
    //                   .map[_](x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn join_on_clause(
        &mut self,
        e0: Expr,
        s: Span,
        x: Name,
        t: &Type,
        e1: &Expr,
        e2: &Expr,
    ) -> Expr {
        let e1 = self.map_expr(e1);
        let e2 = self.map_expr(e2);
        let r = Rc::new(relation_expr(s));
        let xts = self
            .relational_vars()
            .map(|x| (*x, expr_field(r.clone(), *x)))
            .collect::<Map<_, _>>();
        self.bind_relational_var(x);
        let record = record(
            s,
            std::iter::once((x, expr_var(x)))
                .chain(xts)
                .collect::<Vec<_>>()
                .into(),
        );
        // e1.filter(x => e2 == e3)
        let efilter = call_filter(s, e1, lambda(s, [x], e2));
        let emap = call_map(s, efilter, typed_lambda(s, [(x, t.clone())], record));
        call_flatmap(s, e0, lambda(s, [relation(s)], emap))
    }

    // [e0] var x = e1
    // =>
    // e0.map(r => record(x=e1, x1=r.x1, ..., xn=r.xn))
    fn var_clause(&mut self, e0: Expr, s: Span, x: Name, t: &Type, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        let r = Rc::new(relation_expr(s));
        let xes = self
            .relational_vars()
            .map(|x| (*x, expr_field(r.clone(), *x)));
        let xes = xes.collect::<Map<_, _>>();
        let record = record(
            s,
            std::iter::once((x, e1.with_type(t.clone())))
                .chain(xes)
                .collect::<Vec<_>>()
                .into(),
        );
        call_map(s, e0, lambda(s, [relation(s)], record))
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.push(Scope(vec![]));
    }

    fn exit_scope(&mut self) {
        self.stack.pop();
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Query(s, _, x, t, e, qs) => {
                self.enter_scope();
                let e = self.first_from_clause(*s, *x, t.clone(), e);
                let e = qs.iter().fold(e, |e, q| self.query(e, q));
                self.exit_scope();
                e
            }
            Expr::QueryInto(s, _, x0, t0, e, qs, x1, ts, es) => {
                self.enter_scope();
                let e = self.first_from_clause(*s, *x0, t0.clone(), e);
                let e = qs.iter().fold(e, |e, q| self.query(e, q));
                let es = self.map_exprs(es);
                let es = std::iter::once(e).chain(es).collect::<Vec<_>>();
                let e = call(*s, *x1, ts.clone(), es);
                self.exit_scope();
                e
            }
            Expr::Path(s, t, p) => {
                if let Some(x) = p.as_name() {
                    if self.is_relational_var(&x) {
                        // r.x
                        let e0 = Expr::Path(*s, Type::Unknown, Path::new_name(relation(*s)));
                        return Expr::Field(*s, Type::Unknown, Rc::new(e0), *x);
                    }
                }
                let p = self.map_path(p);
                Expr::Path(*s, t.clone(), p)
            }
            _ => self._map_expr(e),
        }
    }
}

mod util {
    use std::rc::Rc;

    use crate::ast::Expr;
    use crate::ast::Impl;
    use crate::ast::Map;
    use crate::ast::Name;
    use crate::ast::Path;
    use crate::ast::Type;
    use crate::syntax::span::Span;

    pub(super) fn relation(s: Span) -> Name {
        Name::new(s, "r")
    }

    pub(super) fn relation_expr(s: Span) -> Expr {
        expr_var(relation(s))
    }

    pub(super) fn expr_var(x: Name) -> Expr {
        Expr::Path(x.span, Type::Unknown, Path::new_name(x))
    }

    pub(super) fn expr_field(e: Rc<Expr>, x: Name) -> Expr {
        Expr::Field(e.span_of() + x.span, Type::Unknown, e, x)
    }

    /// Direct call
    pub(super) fn call(s: Span, x: Name, ts: Vec<Type>, es: Vec<Expr>) -> Expr {
        Expr::Call(
            s,
            Type::Unknown,
            Rc::new(Expr::Assoc(s, Type::Unknown, Impl::Unknown, x, ts)),
            es,
        )
    }

    pub(super) fn call_map(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "map"),
            vec![Type::Unknown],
            vec![stream, udf],
        )
    }

    pub(super) fn call_filter(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(s, Name::new(s, "filter"), vec![], vec![stream, udf])
    }

    pub(super) fn call_merge(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(s, Name::new(s, "merge"), vec![], vec![stream, udf])
    }

    pub(super) fn call_take(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(s, Name::new(s, "take"), vec![], vec![stream, udf])
    }

    pub(super) fn call_flatmap(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "flatMap"),
            vec![Type::Unknown],
            vec![stream, udf],
        )
    }

    pub(super) fn call_keyby(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "keyBy"),
            vec![Type::Unknown],
            vec![stream, udf],
        )
    }

    pub(super) fn call_window(s: Span, stream: Expr, window: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "window"),
            vec![Type::Unknown],
            vec![stream, window, udf],
        )
    }

    pub(super) fn lambda<const N: usize>(s: Span, x: [Name; N], e: Expr) -> Expr {
        Expr::Lambda(
            s,
            Type::Unknown,
            x.iter().map(|x| (*x, Type::Unknown)).collect::<Map<_, _>>(),
            Type::Unknown,
            Rc::new(e),
        )
    }

    pub(super) fn typed_lambda<const N: usize>(s: Span, xt: [(Name, Type); N], e: Expr) -> Expr {
        Expr::Lambda(
            s,
            Type::Unknown,
            xt.iter().cloned().collect::<Map<_, _>>(),
            Type::Unknown,
            Rc::new(e),
        )
    }

    pub(super) fn record(s: Span, xts: Map<Name, Expr>) -> Expr {
        Expr::Record(s, Type::Unknown, xts)
    }
}
