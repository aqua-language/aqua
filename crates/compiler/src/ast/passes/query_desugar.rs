use std::rc::Rc;

use util::call_drop;
use util::call_filter;
use util::call_flatmap;
use util::call_keyby;
use util::call_map;
use util::call_merge;
use util::call_sortby;
use util::call_take;
use util::call_uniqueby;
use util::call_window;
use util::relation;
use util::typed_lambda;

use crate::ast::Aggr;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Local;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Path;
use crate::ast::QueryOp;
use crate::ast::Type;
use crate::report::source::Cache;
use crate::report::span::Span;
use crate::report::Report;
use crate::traversal::mapper::Mapper;

use self::util::call;
use self::util::expr_field;
use self::util::expr_var;
use self::util::lambda;
use self::util::record;
use self::util::relation_expr;
use self::util::relation_local;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Vec<Local>>,
    pub report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast, _: &mut Cache) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            report: Report::new(),
        }
    }

    fn bind_relational_var(&mut self, l: Local) {
        self.stack.last_mut().unwrap().push(l);
    }

    fn unbind_relational_vars(&mut self) {
        self.stack.last_mut().unwrap().clear();
    }

    fn unbind_relational_var(&mut self, x: Name) {
        self.stack.last_mut().unwrap().retain(|f| x != f.name);
    }

    fn is_relational_var(&self, x: &Name) -> bool {
        self.stack.iter().any(|s| s.iter().any(|f| *x == f.name))
    }

    fn relational_vars(&self) -> impl Iterator<Item = &Local> {
        self.stack.last().unwrap().iter()
    }

    fn query(&mut self, e0: Expr, q: &QueryOp) -> Expr {
        match q {
            QueryOp::Where(s, e1) => self.where_clause(e0, *s, e1),
            QueryOp::Union(s, e1) => self.union_clause(e0, *s, e1),
            QueryOp::Limit(s, e1) => self.limit_clause(e0, *s, e1),
            QueryOp::From(s, l, e) => self.from_clause(e0, *s, l, e),
            QueryOp::Select(s, xes) => self.select_clause(e0, *s, xes),
            QueryOp::GroupOverCompute(s, l, e1, e2, aggs) => {
                self.group_over_compute_clause(e0, *s, l, e1, e2, aggs)
            }
            QueryOp::JoinOn(s, l, e1, e2) => self.join_on_clause(e0, *s, l, e1, e2),
            QueryOp::Local(s, l, e) => self.var_clause(e0, *s, l, e),
            QueryOp::OverCompute(s, e, aggs) => self.over_compute_clause(e0, *s, e, aggs),
            QueryOp::JoinOverOn(s, l, e1, e2, e3) => {
                self.join_over_on_clause(e0, *s, l, e1, e2, e3)
            }
            QueryOp::Drop(s, x) => self.drop_clause(e0, *s, *x),
            QueryOp::Cross(s, l, e) => self.cross_clause(e0, *s, l, e),
            QueryOp::Order(s, e) => self.order_clause(e0, *s, e),
            QueryOp::Distinct(s, e) => self.distinct_clause(e0, *s, e),
            QueryOp::Compute(s, aggs) => self.compute_clause(e0, *s, aggs),
            QueryOp::GroupCompute(s, l, e, aggs) => self.group_compute_clause(e0, *s, l, e, aggs),
            QueryOp::Err(s) => Expr::Err(*s, Type::Unknown),
            QueryOp::Skip(s, e) => self.skip_clause(e0, *s, e),
        }
    }

    // from x in e
    // =>
    // map(e, x => record(x=x))
    fn first_from_clause(&mut self, s: Span, l: &Local, e: &Expr) -> Expr {
        let e = self.map_expr(e);
        self.bind_relational_var(l.clone());
        let estruct = record(s, vec![(l.clone(), expr_var(l.clone()))]);
        let elam = Expr::Lambda(
            s,
            Type::Unknown,
            vec![l.clone()].into(),
            Type::Unknown,
            Rc::new(estruct),
        );
        call_map(s, e, elam)
    }

    // e0 where e1
    // =>
    // filter(e0, r => e1)
    fn where_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e = self.map_expr(e1);
        let elam = lambda(s, [relation_local(s)], e);
        call_filter(s, e0, elam)
    }

    // e0 union e1
    // =>
    // merge(e0, e1)
    fn union_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        call_merge(s, e0, e1)
    }

    // e0 limit e1
    // =>
    // take(e0, r => e1)
    fn limit_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        call_take(s, e0, e1)
    }

    // e0 skip e1
    fn skip_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        call_drop(s, e0, e1)
    }

    // e0 from x in e
    // =>
    // flatMap(e0, r => e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn from_clause(&mut self, e0: Expr, s: Span, l: &Local, e1: &Expr) -> Expr {
        let e = self.map_expr(e1);
        // record(x=x, x1=r.x1, ..., xn=r.xn)
        let r = Rc::new(relation_expr(s));
        let xt0 = (l.clone(), expr_var(l.clone()));
        let xts = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)));
        let xts = xts.collect::<Map<_, _>>();
        let record = record(s, std::iter::once(xt0).chain(xts).collect::<Vec<_>>());
        self.bind_relational_var(l.clone());
        // x => record(x=x, x1=r.x1, ..., xn=r.xn)
        let elam = typed_lambda(s, [l.clone()], record);
        // e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn))
        let emap = call(s, Name::new(s, "map"), vec![Type::Unknown], vec![e, elam]);
        // flatMap(e0, r => e.map(x => record(x=x, x1=r.x1, ..., xn=r.xn)))
        let elam = lambda(s, [relation_local(s)], emap);
        call_flatmap(s, e0, elam)
    }

    // e0 select x1=e1,...,xn=en
    // =>
    // map(e0, r => record(x1=r.x1, ..., xn=r.xn))
    fn select_clause(&mut self, e0: Expr, s: Span, xes: &[(Local, Expr)]) -> Expr {
        let xts = xes
            .iter()
            .map(|(l, e)| (l.clone(), self.map_expr(e)))
            .collect::<Vec<_>>();
        self.unbind_relational_vars();
        for (l, _) in &xts {
            self.bind_relational_var(l.clone());
        }
        let record = record(s, xts);
        call_map(s, e0, lambda(s, [relation_local(s)], record))
    }

    // e0 drop x
    // =>
    // map(e0, r => record(x1=r.x1, ..., xn=r.xn)) where x is not in {x1, ..., xn}
    fn drop_clause(&mut self, e0: Expr, s: Span, x: Name) -> Expr {
        self.unbind_relational_var(x);
        let r = Rc::new(relation_expr(s));
        let xts = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)))
            .collect::<Vec<_>>();
        let record = record(s, xts);
        call_map(s, e0, lambda(s, [relation_local(s)], record))
    }

    // e0 group xkey = ekey
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
        lkey: &Local,
        ekey: &Expr,
        ewin: &Expr,
        aggs: &[Aggr],
    ) -> Expr {
        // e0.keyBy(r => ekey)
        let ekey = self.map_expr(ekey);
        let ekeyby = call_keyby(s, e0, lambda(s, [relation_local(s)], ekey));
        // (xkey, r) => record(...)
        let erecord = {
            let xts = self.aggs(s, aggs);
            let xts = std::iter::once((lkey.clone(), expr_var(lkey.clone())))
                .chain(xts)
                .collect::<Vec<_>>();
            self.unbind_relational_vars();
            for (l, _) in xts.iter() {
                self.bind_relational_var(l.clone());
            }
            record(s, xts)
        };
        let efun = lambda(s, [lkey.clone(), relation_local(s)], erecord);
        // ekeyby.window(ewin, (xkey, rs) => record(...))
        let ewin = self.map_expr(ewin);
        call_window(s, ekeyby, ewin, efun)
    }

    // x = e1 of e2 [if e3]
    // =>
    // x = e1(r[.filter(r => e3)].map(r => e2))
    fn aggs(&mut self, s: Span, aggs: &[Aggr]) -> Vec<(Local, Expr)> {
        aggs.iter()
            .map(|agg| {
                let l = relation_local(s);
                let v0 = relation_expr(s);
                if let Some(e2) = &agg.filter_expr {
                    let v1 = call_filter(s, v0, lambda(s, [l.clone()], self.map_expr(e2)));
                    let v2 = call_map(
                        s,
                        v1,
                        lambda(s, [l.clone()], self.map_expr(&agg.reduce_expr)),
                    );
                    let v3 = call(s, agg.name, vec![], vec![v2]);
                    (agg.local.clone(), v3)
                } else {
                    let v1 = call_map(s, v0, lambda(s, [l], self.map_expr(&agg.reduce_expr)));
                    let v2 = call(s, agg.name, vec![], vec![v1]);
                    (agg.local.clone(), v2)
                }
            })
            .collect::<Vec<_>>()
    }

    // e0 over e1 compute x1=efun1 of eattr1,...,xn=efunn of eattrn
    // =>
    // e0.window[_](e1,
    //    r =>
    //      record(x1=efun1(r.map[_](r => eattr1)),
    //             ...,
    //             xn=efunn(r.map[_](r => eattrn)))
    fn over_compute_clause(&mut self, e0: Expr, s: Span, e: &Expr, aggs: &[Aggr]) -> Expr {
        let e = self.map_expr(e);
        let les = self.aggs(s, aggs);
        self.unbind_relational_vars();
        for (l, _) in les.iter() {
            self.bind_relational_var(l.clone());
        }
        let record = record(s, les);
        let elam = lambda(s, [relation_local(s)], record);
        call_window(s, e0, e, elam)
    }

    // e0 join x in e1 on e2 == e3
    // =>
    // e0.flatMap[_](r => e1.filter(x => e2 == e3)
    //                   .map[_](x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn join_on_clause(&mut self, e0: Expr, s: Span, l: &Local, e1: &Expr, e2: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        let e2 = self.map_expr(e2);
        let r = Rc::new(relation_expr(s));
        let les = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)))
            .collect::<Vec<_>>();
        self.bind_relational_var(l.clone());
        let record = record(
            s,
            std::iter::once((l.clone(), expr_var(l.clone())))
                .chain(les)
                .collect::<Vec<_>>()
                .into(),
        );
        // e1.filter(x => e2 == e3)
        let efilter = call_filter(s, e1, lambda(s, [l.clone()], e2));
        let emap = call_map(s, efilter, typed_lambda(s, [l.clone()], record));
        call_flatmap(s, e0, lambda(s, [relation_local(s)], emap))
    }

    // e0 var x = e1
    // =>
    // e0.map(r => record(x=e1, x1=r.x1, ..., xn=r.xn))
    fn var_clause(&mut self, e0: Expr, s: Span, l: &Local, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        let r = Rc::new(relation_expr(s));
        let xes = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)));
        let record = record(
            s,
            std::iter::once((l.clone(), e1))
                .chain(xes)
                .collect::<Vec<_>>()
                .into(),
        );
        call_map(s, e0, lambda(s, [relation_local(s)], record))
    }

    // e0 distinct e1
    // =>
    // e0.distinct(r => e1)
    //
    // e0 distinct
    // =>
    // e0.distinct(r => r)
    fn distinct_clause(&mut self, e0: Expr, s: Span, e1: &Option<Rc<Expr>>) -> Expr {
        let e1 = match e1 {
            Some(e) => self.map_expr(e),
            None => relation_expr(s),
        };
        let elam = lambda(s, [relation_local(s)], e1);
        call_uniqueby(s, e0, elam)
    }

    // e0 cross x in e1
    // =>
    // e0.flatMap[_](r => e1.map[_](x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn cross_clause(&mut self, e0: Expr, s: Span, l: &Local, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        let r = Rc::new(relation_expr(s));
        let les = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)))
            .collect::<Vec<_>>();
        self.bind_relational_var(l.clone());
        let record = record(
            s,
            std::iter::once((l.clone(), expr_var(l.clone())))
                .chain(les)
                .collect::<Vec<_>>()
                .into(),
        );
        let emap = call_map(s, e1, typed_lambda(s, [l.clone()], record));
        call_flatmap(s, e0, lambda(s, [relation_local(s)], emap))
    }

    // e0 order e1
    // =>
    // e0.sortBy(r => e1)
    //
    // e0 order
    // =>
    // e0.sortBy(r => r)
    fn order_clause(&mut self, e0: Expr, s: Span, e1: &Option<Rc<Expr>>) -> Expr {
        let e1 = match e1 {
            Some(e) => self.map_expr(e),
            None => relation_expr(s),
        };
        let elam = lambda(s, [relation_local(s)], e1);
        call_sortby(s, e0, elam)
    }

    // e0 compute x1=efun1 of eattr1,...,xn=efunn of eattrn
    // =>
    // e0.map[_](r => record(x1=efun1(r.map[_](r => eattr1)), ..., xn=efunn(r.map[_](r => eattrn))))
    fn compute_clause(&mut self, e0: Expr, s: Span, aggs: &[Aggr]) -> Expr {
        let les = self.aggs(s, aggs);
        self.unbind_relational_vars();
        for (l, _) in les.iter() {
            self.bind_relational_var(l.clone());
        }
        let record = record(s, les);
        call_map(s, e0, lambda(s, [relation_local(s)], record))
    }

    // e0 group x = e1 compute x1=efun1 of eattr1,...,xn=efunn of eattrn
    // =>
    // e0.keyBy[_](r => e1)
    //   .map[_]((x, r) => record(x=x, x1=efun1(r.map[_](r => eattr1)), ..., xn=efunn(r.map[_](r => eattrn))))
    fn group_compute_clause(
        &mut self,
        e0: Expr,
        s: Span,
        l: &Local,
        e1: &Expr,
        aggs: &[Aggr],
    ) -> Expr {
        let e1 = self.map_expr(e1);
        let ekeyby = call_keyby(s, e0, lambda(s, [relation_local(s)], e1));
        let les = self.aggs(s, aggs);
        let les = std::iter::once((l.clone(), expr_var(l.clone())))
            .chain(les)
            .collect::<Vec<_>>();
        self.unbind_relational_vars();
        for (l, _) in les.iter() {
            self.bind_relational_var(l.clone());
        }
        let record = record(s, les);
        let efun = lambda(s, [l.clone(), relation_local(s)], record);
        call_map(s, ekeyby, efun)
    }

    // e0 join x in e1 over e2 on e3
    // =>
    // e0.flatMap[_](r => e1.window[_](e2, x => e3)
    //                   .map[_](x => record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn join_over_on_clause(
        &mut self,
        e0: Expr,
        s: Span,
        l: &Local,
        e1: &Expr,
        e2: &Expr,
        e3: &Expr,
    ) -> Expr {
        let e1 = self.map_expr(e1);
        let e2 = self.map_expr(e2);
        let e3 = self.map_expr(e3);
        let r = Rc::new(relation_expr(s));
        let les = self
            .relational_vars()
            .map(|l| (l.clone(), expr_field(r.clone(), l.name)))
            .collect::<Vec<_>>();
        self.bind_relational_var(l.clone());
        let record = record(
            s,
            std::iter::once((l.clone(), expr_var(l.clone())))
                .chain(les)
                .collect::<Vec<_>>()
                .into(),
        );
        let ewindow = call_window(s, e1, e2, lambda(s, [l.clone()], e3));
        let emap = call_map(s, ewindow, typed_lambda(s, [l.clone()], record));
        call_flatmap(s, e0, lambda(s, [relation_local(s)], emap))
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.push(vec![]);
    }

    fn exit_scope(&mut self) {
        self.stack.pop();
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Query(s, _, l, e, qs) => {
                self.enter_scope();
                let e = self.first_from_clause(*s, l, e);
                let e = qs.iter().fold(e, |e, q| self.query(e, q));
                self.exit_scope();
                e
            }
            Expr::QueryInto(s, _, l, e, qs, x1, ts, es) => {
                self.enter_scope();
                let e = self.first_from_clause(*s, l, e);
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
    use crate::ast::Local;
    use crate::ast::Map;
    use crate::ast::Name;
    use crate::ast::Path;
    use crate::ast::Type;
    use crate::report::span::Span;

    pub(super) fn relation(s: Span) -> Name {
        Name::new(s, "r")
    }

    pub(super) fn relation_local(s: Span) -> Local {
        Local::new(s, Name::new(s, "r"), Type::Unknown, false)
    }

    pub(super) fn relation_expr(s: Span) -> Expr {
        expr_var(relation_local(s))
    }

    pub(super) fn expr_var(l: Local) -> Expr {
        Expr::Path(l.span, l.ty, Path::new_name(l.name))
    }

    pub(super) fn expr_field(e: Rc<Expr>, x: Name) -> Expr {
        Expr::Field(e.span() + x.span, Type::Unknown, e, x)
    }

    // Direct call
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

    pub(super) fn call_drop(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(s, Name::new(s, "drop"), vec![], vec![stream, udf])
    }

    pub(super) fn call_flatmap(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "flatMap"),
            vec![Type::Unknown],
            vec![stream, udf],
        )
    }

    pub(super) fn call_sortby(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "sortBy"),
            vec![Type::Unknown],
            vec![stream, udf],
        )
    }

    pub(super) fn call_uniqueby(s: Span, stream: Expr, udf: Expr) -> Expr {
        call(
            s,
            Name::new(s, "uniqueBy"),
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

    pub(super) fn lambda<const N: usize>(s: Span, ls: [Local; N], e: Expr) -> Expr {
        Expr::Lambda(s, Type::Unknown, ls.to_vec(), Type::Unknown, Rc::new(e))
    }

    pub(super) fn typed_lambda<const N: usize>(s: Span, ls: [Local; N], e: Expr) -> Expr {
        Expr::Lambda(s, Type::Unknown, ls.to_vec(), Type::Unknown, Rc::new(e))
    }

    pub(super) fn record(s: Span, les: Vec<(Local, Expr)>) -> Expr {
        let fes = les
            .into_iter()
            .map(|(l, e)| (l.name, e))
            .collect::<Map<_, _>>();
        Expr::Record(s, Type::Unknown, fes)
    }
}
