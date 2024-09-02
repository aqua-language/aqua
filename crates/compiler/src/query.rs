use std::rc::Rc;

use crate::ast::Aggr;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Query;
use crate::ast::Segment;
use crate::ast::Type;
use crate::span::Span;
use crate::traversal::mapper::Mapper;

use self::util::call;
use self::util::direct_call;
use self::util::expr_field;
use self::util::expr_var;
use self::util::lambda;
use self::util::record;
use self::util::relation;
use self::util::relation_expr;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Scope>,
}

#[derive(Debug)]
struct Scope(Vec<Name>);

impl Context {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn querycomp(&mut self, program: &Program) -> Program {
        self.map_program(program)
    }

    fn bind_relational_var(&mut self, x: Name) {
        self.stack.last_mut().unwrap().0.push(x);
    }

    fn unbind_relational_vars(&mut self) {
        self.stack.last_mut().unwrap().0.clear();
    }

    fn is_relational_var(&self, x: &Name) -> bool {
        self.stack.iter().any(|s| s.0.contains(x))
    }

    fn relational_vars(&self) -> impl Iterator<Item = &Name> {
        self.stack.last().unwrap().0.iter()
    }

    fn query(&mut self, e0: Expr, q: &Query) -> Expr {
        match q {
            Query::Where(s, e1) => self.where_clause(e0, *s, e1),
            Query::From(s, x, e) => self.from_clause(e0, *s, *x, e),
            Query::Select(s, xes) => self.select_clause(e0, *s, xes),
            Query::GroupOverCompute(s, x, e1, e2, aggs) => {
                self.group_over_compute_clause(e0, *s, *x, e1, e2, aggs)
            }
            Query::JoinOn(s, x, e1, e2) => self.join_on_clause(e0, *s, *x, e1, e2),
            Query::Var(s, x, e) => self.var_clause(e0, *s, *x, e),
            Query::OverCompute(s, e, aggs) => self.over_compute_clause(e0, *s, e, aggs),
            Query::JoinOverOn(_, _, _, _, _) => todo!(),
            Query::Err(_) => todo!(),
        }
    }

    /// from x in e
    /// =>
    /// map(e, fun(x) = record(x=x))
    fn first_from_clause(&mut self, s: Span, x: Name, t: Type, e: &Expr) -> Expr {
        let e = self.map_expr(e);
        self.bind_relational_var(x);
        let estruct = Expr::Record(s, Type::Unknown, vec![(x, expr_var(x))].into());
        let elam = Expr::Fun(
            s,
            Type::Unknown,
            vec![(x, t)].into(),
            Type::Unknown,
            Rc::new(estruct),
        );
        direct_call(s, "map", vec![e, elam])
    }

    /// [e0] where e1
    /// =>
    /// filter(e0, fun(r) = e1)
    fn where_clause(&mut self, e0: Expr, s: Span, e1: &Expr) -> Expr {
        let e = self.map_expr(e1);
        let elam = lambda(s, [relation(s)], e);
        direct_call(s, "filter", vec![e0, elam])
    }

    /// [e0] from x in e
    /// =>
    /// flatMap(e0, fun(r) = e.map(fun(x) = record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn from_clause(&mut self, e0: Expr, s: Span, x: Name, e1: &Expr) -> Expr {
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
        // fun(x) = record(x=x, x1=r.x1, ..., xn=r.xn)
        let elam = lambda(s, [x], record);
        // e.map(fun(x) = record(x=x, x1=r.x1, ..., xn=r.xn))
        let emap = direct_call(s, "map", vec![e, elam]);
        // flatMap(e0, fun(r) = e.map(fun(x) = record(x=x, x1=r.x1, ..., xn=r.xn)))
        let elam = lambda(s, [relation(s)], emap);
        direct_call(s, "flatMap", vec![e0, elam])
    }

    // [e0] select x1=e1,...,xn=en
    // =>
    // map(e0, fun(r) = record(x1=r.x1, ..., xn=r.xn))
    fn select_clause(&mut self, e0: Expr, s: Span, xes: &Map<Name, Expr>) -> Expr {
        let xts = xes
            .iter()
            .map(|(x, e)| (*x, self.map_expr(e)))
            .collect::<Map<_, _>>();
        self.unbind_relational_vars();
        xts.keys().for_each(|x| self.bind_relational_var(*x));
        let record = record(s, xts);
        direct_call(s, "map", vec![e0, lambda(s, [relation(s)], record)])
    }

    // [e0] group xkey = ekey over ewin compute xagg1=efun1 of eattr1,...,xaggn=efunn of eattrn
    // =>
    // e0.keyBy(fun(r) = ekey)
    //   .window(
    //     ewin,
    //     fun(xkey, r) = record(
    //       xkey = xkey,
    //       xagg1 = efun1(r.map(fun(r) = eattr1))
    //       ...,
    //       xaggn = efunn(r.map(fun(r) = eattrn))
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
        // e0.keyBy(fun(r) = ekey)
        let ekey = self.map_expr(ekey);
        let ekeyby = direct_call(s, "keyBy", vec![e0, lambda(s, [relation(s)], ekey)]);
        // fun(xkey, r) = record(...)
        let erecord = {
            let r = relation_expr(s);
            let xts = aggs.iter().map(|agg| {
                let efun = self.map_expr(&agg.e0);
                let eattr = self.map_expr(&agg.e1);
                let elam = lambda(s, [relation(s)], eattr);
                let emap = direct_call(s, "map", vec![r.clone(), elam]);
                let eagg = call(s, efun, vec![emap.clone()]);
                (agg.x, eagg)
            });
            let xts = std::iter::once((xkey, Expr::Path(s, Type::Unknown, Path::new_name(xkey))))
                .chain(xts)
                .collect::<Map<_, _>>();
            self.unbind_relational_vars();
            xts.keys().for_each(|x| self.bind_relational_var(*x));
            record(s, xts)
        };
        let efun = lambda(s, [xkey, relation(s)], erecord);
        // ekeyby.window(ewin, fun(xkey, rs) = record(...))
        let ewin = self.map_expr(ewin);
        direct_call(s, "window", vec![ekeyby, ewin, efun])
    }

    // [e0] over e1 compute x1=efun1 of eattr1,...,xn=efunn of eattrn
    // =>
    // e0.window(e1,
    //    fun(r) =
    //      record(x1=efun1(r.map(fun(r) = eattr1)),
    //             ...,
    //             xn=efunn(r.map(fun(r) = eattrn)))
    fn over_compute_clause(&mut self, e0: Expr, s: Span, e: &Expr, aggs: &[Aggr]) -> Expr {
        let e = self.map_expr(e);
        let r = relation_expr(s);
        let xts = aggs.iter().map(|agg| {
            let efun = self.map_expr(&agg.e0);
            let eattr = self.map_expr(&agg.e1);
            let elam = lambda(s, [relation(s)], eattr);
            let emap = direct_call(s, "map", vec![r.clone(), elam]);
            let eagg = call(s, efun, vec![emap.clone()]);
            (agg.x, eagg)
        });
        let xts = xts.collect::<Map<_, _>>();
        self.unbind_relational_vars();
        xts.keys().for_each(|x| self.bind_relational_var(*x));
        let record = record(s, xts);
        let elam = lambda(s, [relation(s)], record);
        direct_call(s, "window", vec![e0, e, elam])
    }

    // [e0] join x in e1 on e2 == e3
    // =>
    // e0.flatMap(fun(r) = e1.filter(fun(x) = e2 == e3)
    //                       .map(fun(x) = record(x=x, x1=r.x1, ..., xn=r.xn)))
    fn join_on_clause(&mut self, e0: Expr, s: Span, x: Name, e1: &Expr, e2: &Expr) -> Expr {
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
        // e1.filter(fun(x) = e2 == e3)
        let efilter = direct_call(s, "filter", vec![e1, lambda(s, [x], e2)]);
        let emap = direct_call(s, "map", vec![efilter, lambda(s, [x], record)]);
        direct_call(s, "flatMap", vec![e0, lambda(s, [relation(s)], emap)])
    }

    // [e0] var x = e1
    // =>
    // e0.map(fun(r) = record(x=e1, x1=r.x1, ..., xn=r.xn))
    fn var_clause(&mut self, e0: Expr, s: Span, x: Name, e1: &Expr) -> Expr {
        let e1 = self.map_expr(e1);
        let r = Rc::new(relation_expr(s));
        let xts = self
            .relational_vars()
            .map(|x| (*x, expr_field(r.clone(), *x)));
        let xts = xts.collect::<Map<_, _>>();
        let record = record(
            s,
            std::iter::once((x, e1))
                .chain(xts)
                .collect::<Vec<_>>()
                .into(),
        );
        direct_call(s, "map", vec![e0, lambda(s, [relation(s)], record)])
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
            Expr::QueryInto(s, t, x0, t0, e, qs, x1, ts, es) => {
                self.enter_scope();
                let e = self.first_from_clause(*s, *x0, t0.clone(), e);
                let e = qs.iter().fold(e, |e, q| self.query(e, q));
                let es = self.map_exprs(es);
                let es = std::iter::once(e).chain(es).collect();
                let path = Path::new(vec![Segment::new(*s, *x1, ts.clone(), vec![].into())]);
                let e = Expr::Path(*s, Type::Unknown, path);
                let e = Expr::Call(*s, t.clone(), Rc::new(e), es);
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
    use crate::ast::Map;
    use crate::ast::Name;
    use crate::ast::Path;
    use crate::ast::Type;
    use crate::span::Span;

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

    pub(super) fn direct_call(s: Span, x: &'static str, es: Vec<Expr>) -> Expr {
        let path = Path::new_name(Name::new(s, x));
        Expr::Call(
            s,
            Type::Unknown,
            Rc::new(Expr::Path(s, Type::Unknown, path)),
            es,
        )
    }

    pub(super) fn call(s: Span, e: Expr, es: Vec<Expr>) -> Expr {
        Expr::Call(s, Type::Unknown, Rc::new(e), es)
    }

    pub(super) fn lambda<const N: usize>(s: Span, x: [Name; N], e: Expr) -> Expr {
        Expr::Fun(
            s,
            Type::Unknown,
            x.iter().map(|x| (*x, Type::Unknown)).collect::<Map<_, _>>(),
            Type::Unknown,
            Rc::new(e),
        )
    }

    pub(super) fn record(s: Span, xts: Map<Name, Expr>) -> Expr {
        Expr::Record(s, Type::Unknown, xts)
    }
}
