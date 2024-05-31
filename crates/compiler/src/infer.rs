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
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::collections::map::Map;
use crate::diag::Report;
use crate::lexer::Span;

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
    ints: Vec<Name>,
    int_default: Type,
    floats: Vec<Name>,
    float_default: Type,
    string: Type,
    char: Type,
    bool: Type,
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
            ints: vec![
                "i8".into(),
                "i32".into(),
                "i64".into(),
                "i128".into(),
                "u8".into(),
                "u32".into(),
                "u64".into(),
                "u128".into(),
            ],
            int_default: Type::Cons("i32".into(), vec![]),
            floats: vec!["f32".into(), "f64".into()],
            float_default: Type::Cons("f64".into(), vec![]),
            string: Type::Cons("String".into(), vec![]),
            char: Type::Cons("char".into(), vec![]),
            bool: Type::Cons("bool".into(), vec![]),
        }
    }

    pub fn unify(&mut self, s: &mut Map<Name, Type>, s0: Span, s1: Span, t0: &Type, t1: &Type) {
        if let Err((t0, t1)) = self.try_unify(s, t0, t1) {
            self.report.err2(
                s0,
                s1,
                "Type mismatch",
                format!("Expected {t0}"),
                format!("Found {t1}"),
            );
        }
    }

    pub fn new_tyvar(&mut self, kind: TypeVar) -> Type {
        self.tvars += 1;
        Type::Var(Name::from(format!("?T{}", self.tvars - 1)), kind)
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
        let program = p.map_type(&mut |t| t.annotate(self));
        println!("{}", program.verbose());
        program.stmts.iter().for_each(|s| self.declare_stmt(s));
        let stmts = program
            .stmts
            .iter()
            .map(|s| self.stmt(s, &mut sub, &mut goals))
            .collect::<Vec<_>>();
        let program = Program::new(stmts);
        for goal in goals {
            if let Some(c) = self.solve(&goal, &[], &mut sub) {
                match c {
                    crate::ast::Candidate::Impl(_) => {}
                    crate::ast::Candidate::Bound(_) => unreachable!(),
                }
            } else {
                self.report.err(
                    goal.span(),
                    "Trait is not implemented",
                    "Found no implementation for trait",
                );
            }
        }
        let program = program.map_type(&mut |t| {
            t.expand_assoc(&mut |b, x, _| {
                if let Some(c) = self.solve(b, &[], &mut sub) {
                    match c {
                        crate::ast::Candidate::Impl(s) => s
                            .types
                            .iter()
                            .find_map(|s| {
                                if &s.name == x {
                                    match &s.body {
                                        StmtTypeBody::UserDefined(t) => Some(t),
                                        StmtTypeBody::Builtin(_) => unreachable!(),
                                    }
                                } else {
                                    None
                                }
                            })
                            .map(|t| t.clone())
                            .unwrap_or_else(|| {
                                self.report.err(
                                    b.span(),
                                    "Type not found",
                                    format!("Type {x} not found"),
                                );
                                Type::Err
                            }),
                        crate::ast::Candidate::Bound(_) => unreachable!(),
                    }
                } else {
                    self.report.err(
                        b.span(),
                        "Trait is not implemented",
                        "Found no implementation for trait",
                    );
                    Type::Err
                }
            })
        });
        let program = program.map_type(&mut |t| t.apply(&sub));
        let program = program.map_type(&mut |t| t.default(&self.int_default, &self.float_default));
        program
    }

    #[allow(unused)]
    fn debug(sub: &Map<Name, Type>) {
        if sub.is_empty() {
            println!("Substitutions: {{}}");
        } else {
            println!("Substitutions: {{");
            for (x, t) in sub {
                println!("    {x} = {t},");
            }
            println!("}}");
        }
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
        self.unify(sub, s.span, e.span_of(), &s.ty, e.type_of());
        self.bind(s.name, Binding::Var(s.span, s.ty.clone()));
        StmtVar::new(e.span_of(), s.name, s.ty.clone(), e)
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
                    ctx.unify(&mut sub, s.span, e1.span_of(), &s.ty, e1.type_of());
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
                    .map_type(&mut |t| t.apply(&sub).default(&ctx.int_default, &ctx.float_default))
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

    pub fn exprs(
        &mut self,
        es: &[Expr],
        sub: &mut Map<Name, Type>,
        goals: &mut Vec<Bound>,
    ) -> Vec<Expr> {
        es.iter().map(|e| self.expr(e, sub, goals)).collect()
    }

    pub fn expr(&mut self, e: &Expr, sub: &mut Map<Name, Type>, goals: &mut Vec<Bound>) -> Expr {
        match e {
            Expr::Path(..) => unreachable!(),
            Expr::Int(s, t0, v) => {
                let t1 = self.new_tyvar(TypeVar::Int);
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Int(*s, t0.apply(sub), *v)
            }
            Expr::Float(s, t0, v) => {
                let t1 = self.new_tyvar(TypeVar::Float);
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Float(*s, t0.apply(sub), *v)
            }
            Expr::Bool(s, t0, v) => {
                let t1 = self.bool.clone();
                let v = *v;
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Bool(*s, t0.apply(sub), v)
            }
            Expr::Char(s, t0, v) => {
                let t1 = self.char.clone();
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Char(*s, t0.apply(sub), *v)
            }
            Expr::String(s, t0, v) => {
                let t1 = self.string.clone();
                self.unify(sub, *s, *s, t0, &t1);
                Expr::String(*s, t0.apply(sub), *v)
            }
            Expr::Struct(s, t0, x, ts, xes) => {
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
                    self.unify(sub, e.span_of(), stmt.span, e.type_of(), &t2);
                }
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(sub, *s, stmt.span, t0, &t1);
                Expr::Struct(*s, t0.apply(sub), *x, ts.clone(), xes.clone())
            }
            Expr::Enum(s, t0, x, ts, x1, e) => {
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
                self.unify(sub, e.span_of(), stmt.span, e.type_of(), &t2);
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(sub, *s, stmt.span, t0, &t1);
                Expr::Enum(*s, t0.apply(sub), *x, ts.clone(), *x1, e.into())
            }
            Expr::Tuple(s, t0, es) => {
                let es = self.exprs(es, sub, goals);
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t1 = Type::Tuple(ts);
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Tuple(*s, t0.apply(sub), es)
            }
            Expr::Var(s, t0, x) => {
                let Binding::Var(s1, t1) = self.get(x).clone() else {
                    unreachable!()
                };
                self.unify(sub, *s, s1, t0, &t1);
                Expr::Var(*s, t0.apply(sub), *x)
            }
            Expr::Def(s, t0, x, ts0) => {
                let stmt = self.defs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts0.clone())
                    .collect::<Vec<_>>();
                let preds = stmt
                    .where_clause
                    .iter()
                    .map(|p| p.map_type(&mut |t| t.instantiate(&gsub)))
                    .collect::<Vec<_>>();
                goals.extend(preds);
                let t2 = Type::Fun(
                    stmt.params.values().cloned().collect::<Vec<_>>(),
                    Rc::new(stmt.ty.clone()),
                )
                .instantiate(&gsub);
                self.unify(sub, *s, stmt.span, t0, &t2);
                Expr::Def(*s, t0.apply(sub), *x, ts0.clone())
            }
            Expr::Call(s, t0, e, es) => {
                let e = self.expr(e, sub, goals);
                let es = self.exprs(es, sub, goals);
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t2 = Type::Fun(ts, Rc::new(t0.clone()));
                self.unify(sub, *s, e.span_of(), e.type_of(), &t2);
                Expr::Call(*s, t0.apply(sub), Rc::new(e), es)
            }
            Expr::Block(s, t0, b) => {
                let b = self.block(b, sub, goals);
                self.unify(sub, *s, e.span_of(), t0, b.expr.type_of());
                Expr::Block(*s, t0.apply(sub), b)
            }
            Expr::Query(..) => todo!(),
            Expr::Field(s, t0, e, x) => {
                let e = self.expr(e, sub, goals);
                let t = e.type_of().apply(sub);
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
                        self.unify(sub, *s, e.span_of(), t0, &t1);
                        Expr::Field(*s, t0.apply(sub), e.into(), *x)
                    }
                    Type::Record(xts) => {
                        let t1 = xts
                            .iter()
                            .find_map(|(x1, t)| (x1 == x).then_some(t))
                            .unwrap()
                            .clone();
                        self.unify(sub, *s, e.span_of(), t0, &t1);
                        Expr::Field(*s, t0.apply(sub), e.into(), *x)
                    }
                    _ => {
                        self.report.err(
                            *s,
                            "Unknown type",
                            format!("Type {t} must be known at this point."),
                        );
                        Expr::Err(*s, t.clone())
                    }
                }
            }
            Expr::Index(s, t0, e, i) => {
                let e = self.expr(e, sub, goals);
                let t = e.type_of().apply(sub);
                let Type::Tuple(ts) = &t else {
                    self.report.err(
                        *s,
                        "Unknown type",
                        format!("Type {t} must be known at this point."),
                    );
                    return Expr::Err(*s, t.clone());
                };
                let Some(t1) = ts.get(i.data) else {
                    self.report.err(
                        *s,
                        format!("Index {i} out of bounds ({i} >= {})", ts.len()),
                        format!("Index {i} out of bounds."),
                    );
                    return Expr::Err(*s, t.clone());
                };
                self.unify(sub, *s, e.span_of(), t0, t1);
                Expr::Index(*s, t0.apply(sub), e.into(), *i)
            }
            Expr::Array(s, t0, es) => {
                let es = self.exprs(es, sub, goals);
                let t1 = es
                    .first()
                    .map(|e| e.type_of().clone())
                    .unwrap_or_else(|| self.new_tyvar(TypeVar::General));
                es.iter()
                    .for_each(|e| self.unify(sub, *s, e.span_of(), e.type_of(), &t1));
                let t2 = Type::Cons(Name::from("Array"), vec![t1]);
                self.unify(sub, *s, *s, t0, &t2);
                Expr::Array(*s, t0.apply(sub), es)
            }
            Expr::Err(s, t0) => Expr::Err(*s, t0.clone()),
            Expr::Assign(s, t0, e0, e1) => {
                let e0 = self.expr(e0, sub, goals);
                let e1 = self.expr(e1, sub, goals);
                self.unify(sub, *s, e0.span_of(), e0.type_of(), e1.type_of());
                self.unify(sub, *s, e0.span_of(), t0, &Type::Tuple(vec![]));
                Expr::Assign(*s, t0.apply(sub), Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::Fun(_, _, _, _, _) => todo!(),
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(_, _, _, _) => todo!(),
            Expr::Record(s, t0, xes) => {
                let xts = xes
                    .iter()
                    .map(|(x, e)| {
                        let e = self.expr(e, sub, goals);
                        (x.clone(), e.type_of().clone())
                    })
                    .collect::<Map<_, _>>();
                let t1 = Type::Record(xts);
                self.unify(sub, *s, *s, t0, &t1);
                Expr::Record(*s, t0.apply(sub), xes.clone())
            }
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Assoc(s, t0, b, x1, ts1) => match b {
                Bound::Path(_, _) => unreachable!(),
                Bound::Trait(_, x, ts, _) => {
                    let stmt = self.traits.get(x).unwrap();
                    let def_stmt = stmt.defs.iter().find(|d| &d.name == x1).unwrap();
                    let gs0 = stmt.generics.clone();
                    let gs1 = def_stmt.generics.clone();
                    let gsub0 = gs0
                        .into_iter()
                        .chain(gs1)
                        .zip(ts.clone().into_iter().chain(ts1.clone()))
                        .collect::<Vec<_>>();
                    let param_ts = def_stmt.params.values().cloned().collect::<Vec<_>>();
                    let return_t = def_stmt.ty.clone();
                    let fun_t = Type::Fun(param_ts, Rc::new(return_t));
                    let fun_t = fun_t.instantiate(&gsub0);
                    self.unify(sub, *s, e.span_of(), t0, &fun_t);
                    let b = b.map_type(&mut |t| t.apply(sub));
                    let ts1 = ts1.iter().map(|t| t.apply(sub)).collect::<Vec<_>>();
                    let ts = ts.iter().map(|t| t.apply(sub)).collect::<Vec<_>>();
                    // let xts = xts.mapv(|t| t.apply(sub));
                    goals.push(Bound::Trait(*s, *x, ts, Map::new()));
                    Expr::Assoc(*s, t0.apply(sub), b, *x1, ts1)
                }
                Bound::Type(_, _) => todo!(),
                Bound::Err(_) => todo!(),
            },
            Expr::Unresolved(_, _t, _x, _ts) => todo!(),
        }
    }

    pub fn solve(
        &mut self,
        goal: &Bound,
        where_clause: &[Bound],
        sub0: &mut Map<Name, Type>,
    ) -> Option<Candidate> {
        let mut solutions = vec![];
        for b in where_clause {
            let mut sub1 = sub0.clone();
            if self.matches(&mut sub1, goal, b) {
                *sub0 = sub1;
                solutions.push(Candidate::Bound(b.clone()));
            }
        }
        for stmt in self.impls.clone() {
            let stmt = self.instantiate_impl(&stmt);
            let mut sub1 = sub0.clone();
            if self.matches(&mut sub1, goal, &stmt.head)
                && stmt
                    .where_clause
                    .iter()
                    .all(|subgoal| self.solve(subgoal, where_clause, &mut sub1).is_some())
            {
                *sub0 = sub1;
                solutions.push(Candidate::Impl(stmt));
            }
        }
        // TODO: Return the current solutions
        match solutions.len() {
            0 => None,
            1 => Some(solutions.pop().unwrap()),
            _ => None,
        }
    }

    // Checks if i0 and i1 match, and if so, returns a substitution.
    fn matches(&mut self, s: &mut Map<Name, Type>, b0: &Bound, b1: &Bound) -> bool {
        match (b0, b1) {
            (Bound::Trait(_, x0, ts0, _), Bound::Trait(_, x1, ts1, _)) => {
                x0 == x1
                    && ts0.len() == ts1.len()
                    && ts0
                        .iter()
                        .zip(ts1.iter())
                        .all(|(t0, t1)| self.try_unify(s, t0, t1).is_ok())
            }
            (Bound::Type(_, t0), Bound::Type(_, t1)) => self.try_unify(s, t0, t1).is_ok(),
            (Bound::Path(..), _) | (_, Bound::Path(..)) => unreachable!(),
            (Bound::Err(..), _) | (_, Bound::Err(..)) => true,
            (Bound::Trait(..), Bound::Type(..)) | (Bound::Type(..), Bound::Trait(..)) => false,
        }
    }

    #[track_caller]
    pub fn try_unify(
        &mut self,
        s0: &mut Map<Name, Type>,
        t0: &Type,
        t1: &Type,
    ) -> Result<(), (Type, Type)> {
        let t0 = t0.apply(s0);
        let t1 = t1.apply(s0);
        if let Some(s1) = self.mgu(&t0, &t1) {
            *s0 = Self::compose(s0.clone(), s1);
            Ok(())
        } else {
            Err((t0, t1))
        }
    }

    fn compose(s0: Map<Name, Type>, s1: Map<Name, Type>) -> Map<Name, Type> {
        s1.into_iter()
            .map(|(x, t)| (x, t.apply(&s0)))
            .chain(s0.clone())
            .collect()
    }

    fn mgu_fold<'a>(
        &mut self,
        ts0: impl IntoIterator<Item = &'a Type>,
        ts1: impl IntoIterator<Item = &'a Type>,
    ) -> Option<Map<Name, Type>> {
        ts0.into_iter()
            .zip(ts1)
            .try_fold(Map::new(), |s0, (t0, t1)| {
                let t0 = t0.apply(&s0);
                let t1 = t1.apply(&s0);
                let s1 = self.mgu(&t0, &t1)?;
                Some(Self::compose(s1, s0))
            })
    }

    fn mgu(&mut self, t0: &Type, t1: &Type) -> Option<Map<Name, Type>> {
        match (t0, t1) {
            (Type::Cons(x0, ts0), Type::Cons(x1, ts1)) => {
                (x0 == x1 && ts0.len() == ts1.len()).then(|| self.mgu_fold(ts0, ts1))?
            }
            (Type::Var(x, TypeVar::General), t) | (t, Type::Var(x, TypeVar::General)) => match t {
                Type::Var(x1, TypeVar::General) => {
                    if x != x1 {
                        Some(Map::singleton(*x, t.clone()))
                    } else {
                        Some(Map::new())
                    }
                }
                Type::Var(_, TypeVar::Int) => Some(Map::singleton(*x, t.clone())),
                Type::Var(_, TypeVar::Float) => Some(Map::singleton(*x, t.clone())),
                _ => Some(Map::singleton(*x, t.clone())),
            },

            (Type::Var(x, TypeVar::Int), t) | (t, Type::Var(x, TypeVar::Int)) => match t {
                Type::Cons(x1, ..) if self.ints.contains(x1) => Some(Map::singleton(*x, t.clone())),
                Type::Var(x1, TypeVar::Int) => {
                    if x != x1 {
                        Some(Map::singleton(*x, t.clone()))
                    } else {
                        Some(Map::new())
                    }
                }
                _ => None,
            },
            (Type::Var(x, TypeVar::Float), t) | (t, Type::Var(x, TypeVar::Float)) => match t {
                Type::Cons(x1, ..) if self.floats.contains(x1) => {
                    Some(Map::singleton(*x, t.clone()))
                }
                Type::Var(x1, TypeVar::Int) => {
                    if x != x1 {
                        Some(Map::singleton(*x, t.clone()))
                    } else {
                        Some(Map::new())
                    }
                }
                _ => None,
            },
            (Type::Fun(ts0, t0), Type::Fun(ts1, t1)) => (ts0.len() == ts1.len()).then(|| {
                self.mgu_fold(
                    ts0.iter().chain([t0.as_ref()]),
                    ts1.iter().chain([t1.as_ref()]),
                )
            })?,
            (Type::Tuple(ts0), Type::Tuple(ts1)) => (ts0.len() == ts1.len())
                .then(|| self.mgu_fold(ts0.iter().chain(ts1), ts1.iter().chain(ts0)))?,
            (Type::Record(xts0), Type::Record(xts1)) => {
                if xts0.len() == xts1.len() {
                    let xts0 = xts0.sort_keys();
                    let xts1 = xts1.sort_keys();
                    if xts0.same_keys_sorted(&xts1) {
                        self.mgu_fold(
                            xts0.values().chain(xts1.values()),
                            xts1.values().chain(xts0.values()),
                        )
                    } else {
                        None
                    }
                } else {
                    return None;
                }
            }
            (Type::Generic(x0), Type::Generic(x1)) => (x0 == x1).then(Map::new),
            (Type::Assoc(b0, x0, ts0), Type::Assoc(b1, x1, ts1)) => match (b0, b1) {
                (Bound::Err(_), _) | (_, Bound::Err(_)) => Some(Map::new()),
                (Bound::Path(..), _) | (_, Bound::Path(..)) => unreachable!(),
                (Bound::Type(_, t0), Bound::Type(_, t1)) => {
                    if x0 == x1 && ts0.len() == ts1.len() {
                        self.mgu_fold(
                            ts0.iter().chain([t0.as_ref()]),
                            ts1.iter().chain([t1.as_ref()]),
                        )
                    } else {
                        None
                    }
                }
                (Bound::Trait(_, x0, ts0, _), Bound::Trait(_, x1, ts1, _)) => {
                    if x0 == x1 && ts0.len() == ts1.len() {
                        self.mgu_fold(ts0, ts1)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            (Type::Assoc(b, x, _), t2) | (t2, Type::Assoc(b, x, _)) => {
                self.mgu(t2, b.get_type(x).unwrap())
            }
            (Type::Err, _) | (_, Type::Err) => Some(Map::new()),
            (Type::Never, _) | (_, Type::Never) => Some(Map::new()),
            _ => None,
        }
    }

    pub fn instantiate_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let sub = s
            .generics
            .iter()
            .map(|x| (*x, self.new_tyvar(TypeVar::General)))
            .collect::<Vec<_>>();
        let head = s.head.map_type(&mut |t| t.instantiate(&sub));
        let body = s
            .where_clause
            .iter()
            .map(|i| i.map_type(&mut |t| t.instantiate(&sub)))
            .collect::<Vec<_>>();
        let stmt_defs = s
            .defs
            .iter()
            .map(|s| Rc::new(s.map_type(&mut |t| t.apply(&sub))))
            .collect::<Vec<_>>();
        let stmt_types = s
            .types
            .iter()
            .map(|s| Rc::new(s.map_type(&mut |t| t.apply(&sub))))
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
