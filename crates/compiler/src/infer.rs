use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Candidate;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::diag::Report;
use crate::lexer::Span;
use crate::map::Map;

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Context {
    tvars: usize,
    stack: Vec<Scope>,
    pub impls: Vec<Rc<StmtImpl>>,
    defs: HashMap<Name, Rc<StmtDef>>,
    structs: HashMap<Name, Rc<StmtStruct>>,
    enums: HashMap<Name, Rc<StmtEnum>>,
    traits: HashMap<Name, Rc<StmtTrait>>,
    types: HashMap<Name, Rc<StmtType>>,
    pub report: Report,
}

#[derive(Default, Debug)]
pub struct Scope {
    binds: Map<Name, Binding>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope::default()
    }
}

#[derive(Debug, Clone)]
pub enum Binding {
    Var(Span, Type),
    Type(Span, Vec<Name>),
}

impl Context {
    pub fn new() -> Context {
        Context {
            tvars: 0,
            stack: vec![Scope::new()],
            report: Report::new(),
            impls: vec![],
            defs: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn recover<T>(&mut self, s0: Span, s1: Span, r: Result<T, (Type, Type)>) {
        if let Err((t0, t1)) = r {
            self.report.err2(
                s0,
                s1,
                "Type mismatch",
                format!("Expected {t0}"),
                format!("Found {t1}"),
            );
        }
    }

    pub fn new_tyvar(&mut self) -> Type {
        self.tvars += 1;
        Type::Var(Name::from(format!("?T{}", self.tvars - 1)))
    }

    pub fn get(&self, x1: &Name) -> &Binding {
        self.stack
            .iter()
            .rev()
            .find_map(|s| {
                s.binds
                    .iter()
                    .rev()
                    .find(|(x0, _)| x0 == x1)
                    .map(|(_, b)| b)
            })
            .unwrap_or_else(|| panic!("Unknown variable {x1}"))
    }

    pub fn bind(&mut self, x: Name, b: Binding) {
        self.stack.last_mut().unwrap().binds.insert(x, b);
    }

    pub fn scoped<T>(&mut self, f: impl FnOnce(&mut Context) -> T) -> T {
        self.stack.push(Scope::new());
        let v = f(self);
        self.stack.pop();
        v
    }

    pub fn infer(&mut self, p: &Program) -> Program {
        let mut goals = Vec::new();
        let mut sub = Map::new();
        let program = p.annotate(self);
        program.stmts.iter().for_each(|s| self.declare_stmt(s));
        let stmts = program
            .stmts
            .iter()
            .map(|s| self.stmt(s, &mut sub, &mut goals))
            .collect::<Vec<_>>();
        for goal in goals {
            if let Some(c) = self.solve(&goal, &[], &mut sub) {
                match c {
                    crate::ast::Candidate::Impl(_) => {}
                    crate::ast::Candidate::Bound(_) => {}
                }
            } else {
                self.report
                    .err(goal.span(), "Unsolved goal", "Could not solve goal");
            }
        }
        let p = Program::new(stmts).map_type(&|t| t.apply(&sub));
        println!("{}", p.verbose());
        p
    }

    pub fn declare_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.defs.insert(s.name, s.clone());
            }
            Stmt::Impl(s) => {
                self.impls.push(s.clone());
            }
            Stmt::Expr(_) => {}
            Stmt::Struct(s) => {
                self.structs.insert(s.name, s.clone());
            }
            Stmt::Enum(s) => {
                self.enums.insert(s.name, s.clone());
            }
            Stmt::Type(s) => {
                self.types.insert(s.name, s.clone());
            }
            Stmt::Trait(s) => {
                self.traits.insert(s.name, s.clone());
            }
            Stmt::Err(_) => todo!(),
        }
    }

    pub fn stmt(&mut self, s: &Stmt, sub: &mut Map<Name, Type>, goals: &mut Vec<Bound>) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.stmt_var(s, sub, goals))),
            Stmt::Def(s) => Stmt::Def(Rc::new(self.stmt_def(s))),
            Stmt::Impl(s) => Stmt::Impl(Rc::new(self.stmt_impl(s))),
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.expr(e, sub, goals))),
            Stmt::Struct(s) => Stmt::Struct(s.clone()),
            Stmt::Enum(s) => Stmt::Enum(s.clone()),
            Stmt::Type(s) => Stmt::Type(s.clone()),
            Stmt::Trait(s) => Stmt::Trait(s.clone()),
            Stmt::Err(_) => todo!(),
        }
    }

    pub fn stmt_var(
        &mut self,
        s: &StmtVar,
        sub: &mut Map<Name, Type>,
        goals: &mut Vec<Bound>,
    ) -> StmtVar {
        let e = self.expr(&s.expr, sub, goals);
        self.recover(s.span, e.span(), unify(sub, &s.ty, e.ty()));
        self.bind(s.name, Binding::Var(s.span, s.ty.clone()));
        StmtVar::new(e.span(), s.name, s.ty.clone(), e)
    }

    pub fn stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let span = s.span;
        let generics = s.generics.clone();
        let head = s.head.clone();
        let defs = s
            .defs
            .iter()
            .map(|d| Rc::new(self.stmt_def(d)))
            .collect::<Vec<_>>();
        let types = s.types.clone();
        let where_clause = s.where_clause.clone();
        StmtImpl::new(span, generics, head, where_clause, defs, types)
    }

    pub fn stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        self.scoped(|ctx| {
            match &s.body {
                StmtDefBody::UserDefined(e) => {
                    let mut sub = Map::new();
                    let mut goals = vec![];
                    for (x, t) in &s.params {
                        ctx.bind(*x, Binding::Var(x.span, t.clone()));
                    }
                    let e1 = ctx.expr(e, &mut sub, &mut goals);
                    ctx.recover(s.span, e1.span(), unify(&mut sub, &s.ty, e1.ty()));
                    for goal in goals {
                        // TODO: Try to solve each goal multiple times
                        if ctx.solve(&goal, &s.where_clause, &mut sub).is_none() {
                            // ctx.report
                            //     .err(goal.span, "Unsolved goal", "Could not solve goal");
                        }
                    }
                    StmtDef::new(
                        s.span,
                        s.name,
                        s.generics.clone(),
                        s.params.clone(),
                        s.ty.clone(),
                        s.where_clause.clone(),
                        StmtDefBody::UserDefined(e1),
                    )
                    .map_type(&|t| t.apply(&sub))
                }
                StmtDefBody::Builtin(b) => StmtDef::new(
                    s.span,
                    s.name,
                    s.generics.clone(),
                    s.params.clone(),
                    s.ty.clone(),
                    s.where_clause.clone(),
                    StmtDefBody::Builtin(b.clone()),
                ),
            }
        })
    }

    fn block(&mut self, b: &Block, sub: &mut Map<Name, Type>, goals: &mut Vec<Bound>) -> Block {
        let span = b.span;
        let ss = b.stmts.iter().map(|s| self.stmt(s, sub, goals)).collect();
        let e = self.expr(&b.expr, sub, goals);
        Block::new(span, ss, e)
    }

    pub fn expr(&mut self, e: &Expr, sub: &mut Map<Name, Type>, goals: &mut Vec<Bound>) -> Expr {
        let s = e.span();
        match e {
            Expr::Unresolved(..) => unreachable!(),
            Expr::Int(_, t0, v) => {
                let t1 = Type::i32();
                self.recover(s, s, unify(sub, t0, &t1));
                Expr::Int(s, t0.apply(sub), *v)
            }
            Expr::Float(_, t0, v) => {
                let t1 = Type::f64();
                self.recover(s, s, unify(sub, t0, &t1));
                Expr::Float(s, t0.apply(sub), *v)
            }
            Expr::Bool(_, t0, v) => {
                let t1 = Type::bool();
                let v = *v;
                self.recover(s, s, unify(sub, t0, &t1));
                Expr::Bool(s, t0.apply(sub), v)
            }
            Expr::String(_, t0, v) => {
                let t1 = Type::string();
                self.recover(s, s, unify(sub, t0, &t1));
                Expr::String(s, t0.apply(sub), *v)
            }
            Expr::Struct(_, t0, x, ts, xes) => {
                let stmt = self.structs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts.clone())
                    .collect::<Vec<_>>();
                for (x, e) in xes.clone() {
                    let t2 = stmt
                        .fields
                        .iter()
                        .find_map(|(x0, t)| (*x0 == x).then_some(t))
                        .unwrap()
                        .instantiate(&gsub);
                    let e = self.expr(&e, sub, goals);
                    self.recover(e.span(), stmt.span, unify(sub, e.ty(), &t2));
                }
                let t1 = Type::Cons(*x, ts.clone());
                self.recover(s, stmt.span, unify(sub, t0, &t1));
                Expr::Struct(s, t0.apply(sub), *x, ts.clone(), xes.clone())
            }
            Expr::Enum(_, t0, x, ts, x1, e) => {
                let stmt = self.enums.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts.clone())
                    .collect::<Vec<_>>();
                let t2 = stmt
                    .variants
                    .iter()
                    .find_map(|(x0, t)| (x0 == x1).then_some(t))
                    .unwrap()
                    .instantiate(&gsub);
                let e = self.expr(e, sub, goals);
                self.recover(e.span(), stmt.span, unify(sub, e.ty(), &t2));
                let t1 = Type::Cons(*x, ts.clone());
                self.recover(s, stmt.span, unify(sub, t0, &t1));
                Expr::Enum(s, t0.apply(sub), *x, ts.clone(), *x1, e.into())
            }
            Expr::Tuple(_, t0, es) => {
                let es = es
                    .iter()
                    .map(|e| self.expr(e, sub, goals))
                    .collect::<Vec<_>>();
                let ts = es.iter().map(|e| e.ty().clone()).collect::<Vec<_>>();
                let t1 = Type::Tuple(ts);
                self.recover(s, s, unify(sub, t0, &t1));
                Expr::Tuple(s, t0.apply(sub), es)
            }
            Expr::Var(_, t0, x) => {
                let Binding::Var(s1, t1) = self.get(x).clone() else {
                    unreachable!()
                };
                self.recover(s, s1, unify(sub, t0, &t1));
                Expr::Var(s, t0.apply(sub), *x)
            }
            Expr::Def(_, t0, x, ts0) => {
                let stmt = self.defs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts0.clone())
                    .collect::<Vec<_>>();
                let ts = stmt
                    .params
                    .values()
                    .map(|t| t.instantiate(&gsub))
                    .collect::<Vec<_>>();
                let t1 = stmt.ty.instantiate(&gsub);
                let preds = stmt
                    .where_clause
                    .iter()
                    .map(|p| p.map_type(&|t| t.instantiate(&gsub)))
                    .collect::<Vec<_>>();
                goals.extend(preds);
                let t2 = Type::Fun(ts, Rc::new(t1));
                self.recover(s, stmt.span, unify(sub, t0, &t2));
                Expr::Def(s, t0.apply(sub), *x, ts0.clone())
            }
            Expr::Call(_, t0, e, es) => {
                let e = self.expr(e, sub, goals);
                let es = es
                    .iter()
                    .map(|e| self.expr(e, sub, goals))
                    .collect::<Vec<_>>();
                let ts = es.iter().map(|e| e.ty().clone()).collect::<Vec<_>>();
                let t2 = Type::Fun(ts, Rc::new(t0.clone()));
                self.recover(s, e.span(), unify(sub, e.ty(), &t2));
                Expr::Call(s, t0.apply(sub), Rc::new(e), es)
            }
            Expr::Block(_, t0, b) => {
                let b = self.block(b, sub, goals);
                self.recover(s, e.span(), unify(sub, t0, b.expr.ty()));
                Expr::Block(s, t0.apply(sub), b)
            }
            Expr::Query(..) => todo!(),
            Expr::Field(_, t0, e, x) => {
                let e = self.expr(e, sub, goals);
                let t = e.ty().apply(sub);
                match t {
                    Type::Cons(x0, ts) => {
                        let stmt = self.structs.get(&x0).unwrap().clone();
                        let gsub = stmt
                            .generics
                            .clone()
                            .into_iter()
                            .zip(ts.clone())
                            .collect::<Vec<_>>();
                        let t1 = stmt
                            .fields
                            .iter()
                            .find_map(|(x1, t)| (x1 == x).then_some(t))
                            .unwrap()
                            .instantiate(&gsub);
                        self.recover(s, e.span(), unify(sub, t0, &t1));
                        Expr::Field(s, t0.apply(sub), e.into(), *x)
                    }
                    Type::Record(xts) => {
                        let t1 = xts
                            .iter()
                            .find_map(|(x1, t)| (x1 == x).then_some(t))
                            .unwrap()
                            .clone();
                        self.recover(s, e.span(), unify(sub, t0, &t1));
                        Expr::Field(s, t0.apply(sub), e.into(), *x)
                    }
                    _ => {
                        self.report.err(
                            s,
                            "Unknown type",
                            format!("Type {t} must be known at this point."),
                        );
                        Expr::Err(s, t.clone())
                    }
                }
            }
            Expr::Index(_, t0, e, i) => {
                let e = self.expr(e, sub, goals);
                let t = e.ty().apply(sub);
                let Type::Tuple(ts) = &t else {
                    self.report.err(
                        s,
                        "Unknown type",
                        format!("Type {t} must be known at this point."),
                    );
                    return Expr::Err(s, t.clone());
                };
                let Some(t1) = ts.get(i.data) else {
                    self.report.err(
                        s,
                        format!("Index {i} out of bounds ({i} >= {})", ts.len()),
                        format!("Index {i} out of bounds."),
                    );
                    return Expr::Err(s, t.clone());
                };
                self.recover(s, e.span(), unify(sub, t0, t1));
                Expr::Index(s, t0.apply(sub), e.into(), *i)
            }
            Expr::Assoc(span, t0, b, x1, ts1) => {
                let stmt = self
                    .traits
                    .get(&b.name)
                    .unwrap_or_else(|| panic!("Unknown trait {b}"));
                let d = stmt.defs.iter().find(|d| &d.name == x1).unwrap();
                let gs0 = stmt.generics.clone();
                let gs1 = d.generics.clone();
                let gsub0 = gs0
                    .into_iter()
                    .chain(gs1)
                    .zip(b.ts.clone().into_iter().chain(ts1.clone()))
                    .collect::<Vec<_>>();
                let param_ts = d.params.values().cloned().collect::<Vec<_>>();
                let return_t = d.ty.clone();
                let fun_t = Type::Fun(param_ts, Rc::new(return_t)).instantiate(&gsub0);
                self.recover(s, e.span(), unify(sub, t0, &fun_t));
                let b = b.map_type(&|t| t.apply(sub));
                let ts1 = ts1.iter().map(|t| t.apply(sub)).collect::<Vec<_>>();
                goals.push(Bound::Trait(*span, b.clone()));
                Expr::Assoc(s, t0.apply(sub), b, *x1, ts1)
            }
            Expr::Array(_, t0, es) => {
                let es = es
                    .iter()
                    .map(|e| self.expr(e, sub, goals))
                    .collect::<Vec<_>>();
                let t1 = es
                    .first()
                    .map(|e| e.ty().clone())
                    .unwrap_or_else(|| self.new_tyvar());
                for e in es.iter() {
                    self.recover(s, e.span(), unify(sub, e.ty(), &t1));
                }
                let t2 = Type::Cons(Name::from("Array"), vec![t1]);
                self.recover(s, s, unify(sub, t0, &t2));
                Expr::Array(s, t0.apply(sub), es)
            }
            Expr::Err(s, t) => Expr::Err(*s, t.clone()),
            Expr::Assign(_, t0, e0, e1) => {
                let e0 = self.expr(e0, sub, goals);
                let e1 = self.expr(e1, sub, goals);
                self.recover(s, e0.span(), unify(sub, e0.ty(), e1.ty()));
                self.recover(s, e0.span(), unify(sub, t0, &Type::unit()));
                Expr::Assign(s, t0.apply(sub), Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::Fun(_, _, _, _, _) => todo!(),
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(_, _, _, _) => todo!(),
            Expr::Record(_, _, _) => todo!(),
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
        }
    }
}

impl Context {
    pub fn solve(
        &mut self,
        goal: &Bound,
        where_clause: &[Bound],
        sub0: &mut Map<Name, Type>,
    ) -> Option<Candidate> {
        let mut solutions = vec![];
        for tr in where_clause {
            let mut sub1 = sub0.clone();
            if matches(&mut sub1, goal, tr) {
                *sub0 = sub1;
                solutions.push(Candidate::Bound(tr.clone()));
            }
        }
        for i in self.impls.clone() {
            let i = self.instantiate_impl(&i);
            let mut sub1 = sub0.clone();
            if matches(&mut sub1, goal, &i.head)
                && i.where_clause
                    .iter()
                    .all(|subgoal| self.solve(subgoal, where_clause, &mut sub1).is_some())
            {
                *sub0 = sub1;
                solutions.push(Candidate::Impl(i));
            }
        }
        // TODO: Return the current solutions
        match solutions.len() {
            0 => None,
            1 => Some(solutions.pop().unwrap()),
            _ => None,
        }
    }
}

// Checks if i0 and i1 match, and if so, returns a substitution.
fn matches(s: &mut Map<Name, Type>, b0: &Bound, b1: &Bound) -> bool {
    match (b0, b1) {
        (Bound::Trait(_, b0), Bound::Trait(_, b1)) => {
            b0.name == b1.name
                && b0.ts.len() == b1.ts.len()
                && b1
                    .ts
                    .iter()
                    .zip(b0.ts.iter())
                    .all(|(t0, t1)| unify(s, t0, t1).is_ok())
                && b0
                    .xts
                    .iter()
                    .zip(b1.xts.iter())
                    .all(|((x0, t0), (x1, t1))| {
                        assert!(x0 == x1);
                        unify(s, t0, t1).is_ok()
                    })
        }
        (Bound::Unresolved(..), _) | (_, Bound::Unresolved(..)) => unreachable!(),
        (Bound::Err(..), _) | (_, Bound::Err(..)) => true,
    }
    // && i1
    //     .assocs
    //     .iter()
    //     .zip(i0.assocs.iter())
    //     .all(|((_, t0), (_, t1))| unify(s, &t0, &t1).is_ok())
}

#[track_caller]
pub fn unify(s0: &mut Map<Name, Type>, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
    let t0 = t0.apply(s0);
    let t1 = t1.apply(s0);
    let Some(s1) = mgu(&t0, &t1) else {
        return Err((t0, t1));
    };
    *s0 = compose(s0.clone(), s1);
    Ok(())
}

fn compose(s0: Map<Name, Type>, s1: Map<Name, Type>) -> Map<Name, Type> {
    s1.into_iter()
        .map(|(x, t)| (x, t.apply(&s0)))
        .chain(s0.clone())
        .collect()
}

fn mgu_fold<'a>(
    ts0: impl IntoIterator<Item = &'a Type>,
    ts1: impl IntoIterator<Item = &'a Type>,
) -> Option<Map<Name, Type>> {
    ts0.into_iter()
        .zip(ts1)
        .try_fold(Map::new(), |s0, (t0, t1)| {
            let t0 = t0.apply(&s0);
            let t1 = t1.apply(&s0);
            let s1 = mgu(&t0, &t1)?;
            Some(compose(s1, s0))
        })
}

fn mgu(t0: &Type, t1: &Type) -> Option<Map<Name, Type>> {
    match (t0, t1) {
        (Type::Cons(x0, ts0), Type::Cons(x1, ts1)) => {
            (x0 == x1 && ts0.len() == ts1.len()).then(|| mgu_fold(ts0, ts1))?
        }
        (Type::Var(x), t) | (t, Type::Var(x)) => (t0 != t1)
            .then(|| Map::singleton(*x, t.clone()))
            .or_else(|| Some(Map::new())),
        (Type::Fun(ts0, t0), Type::Fun(ts1, t1)) => (ts0.len() == ts1.len()).then(|| {
            mgu_fold(
                ts0.iter().chain([t0.as_ref()]),
                ts1.iter().chain([t1.as_ref()]),
            )
        })?,
        (Type::Tuple(ts0), Type::Tuple(ts1)) => (ts0.len() == ts1.len())
            .then(|| mgu_fold(ts0.iter().chain(ts1), ts1.iter().chain(ts0)))?,
        (Type::Err, _) | (_, Type::Err) => Some(Map::new()),
        (Type::Generic(x0), Type::Generic(x1)) => (x0 == x1).then(Map::new),
        (Type::Assoc(tb, x, _), t2) | (t2, Type::Assoc(tb, x, _)) => {
            let t3 = tb.xts.get(x).unwrap();
            mgu(t2, t3)
        }
        (Type::Hole, _) | (_, Type::Hole) => unreachable!(),
        (Type::Unresolved(_), _) | (_, Type::Unresolved(_)) => unreachable!(),
        _ => None,
    }
}

impl Context {
    pub fn instantiate_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let sub = s
            .generics
            .iter()
            .map(|x| (*x, self.new_tyvar()))
            .collect::<Vec<_>>();

        let head = s.head.map_type(&|t| t.instantiate(&sub));
        let body = s
            .where_clause
            .iter()
            .map(|i| i.map_type(&|t| t.instantiate(&sub)))
            .collect::<Vec<_>>();
        let stmt_defs = s
            .defs
            .iter()
            .map(|s| Rc::new(s.map_type(&|t| t.apply(&sub))))
            .collect::<Vec<_>>();
        let stmt_types = s
            .types
            .iter()
            .map(|s| Rc::new(s.map_type(&|t| t.apply(&sub))))
            .collect::<Vec<_>>();

        StmtImpl::new(s.span, vec![], head, body, stmt_defs, stmt_types)
    }
}

// pub fn instantiate_def(s: &StmtDef, ctx: &mut Context) -> StmtDef {
//     let sub = s
//         .generics
//         .iter()
//         .map(|q| (q.clone(), ctx.new_tyvar()))
//         .collect::<Vec<_>>();
//
//     let body = s
//         .where_clause
//         .iter()
//         .map(|p| p.apply(&sub))
//         .collect::<Vec<_>>();
//     let params = s.params.iter().map(|p| p.apply(&sub)).collect::<Vec<_>>();
//     let ty = s.ty.apply(&sub);
//     let expr = s.expr.apply(&sub);
//
//     StmtDef::new(s.span, s.name.clone(), vec![], body, params, ty, expr)
// }
