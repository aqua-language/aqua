pub mod annotate;
pub mod apply;
pub mod canonicalize;
pub mod defaults;
pub mod expand;
pub mod gather_goals;
pub mod instantiate;
pub mod local;
pub mod type_var;
pub mod unify;

use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;

use crate::ast::Candidate;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtImpl;
use crate::ast::StmtVar;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::collections::map::Map;
use crate::collections::set::Set;
use crate::declare;
use crate::diag::Report;
use crate::span::Span;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::AcceptVisitor;

use self::gather_goals::GatherGoals;
use self::type_var::TypeVarKind;
use self::type_var::TypeVarValue;

#[derive(Debug)]
pub struct Context {
    var_stack: Vec<VarScope>,
    infer_stack: Vec<InferScope>,
    pub type_impls: Vec<StmtImpl>,
    pub report: Report,
    pub decls: declare::Context
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum TraitSolverError {
    Unsolved,
    Ambiguous(Vec<Candidate>),
}

fn ints() -> impl Iterator<Item = Name> {
    [
        "i8".into(),
        "i32".into(),
        "i64".into(),
        "i128".into(),
        "u8".into(),
        "u32".into(),
        "u64".into(),
        "u128".into(),
    ]
    .into_iter()
}

fn floats() -> impl Iterator<Item = Name> {
    ["f32".into(), "f64".into()].into_iter()
}

fn bool() -> Type {
    Type::Cons("bool".into(), vec![])
}

fn char() -> Type {
    Type::Cons("char".into(), vec![])
}

fn unit() -> Type {
    Type::Tuple(vec![])
}

fn string() -> Type {
    Type::Cons("String".into(), vec![])
}

pub fn default_int() -> Type {
    Type::Cons("i32".into(), vec![])
}

pub fn default_float() -> Type {
    Type::Cons("f64".into(), vec![])
}

#[derive(Default, Debug)]
pub struct VarScope(Map<Name, (Span, Type)>);

#[derive(Debug)]
pub struct InferScope {
    pub table: InPlaceUnificationTable<TypeVar>,
    goals: Set<Goal>,
    pub where_clause: Vec<Trait>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Goal {
    pub s: Span,
    pub tr: Trait,
}

impl Goal {
    pub fn new(s: Span, tr: Trait) -> Goal {
        Goal { s, tr }
    }
}

impl InferScope {
    pub fn new(where_clause: Vec<Trait>) -> InferScope {
        InferScope {
            table: InPlaceUnificationTable::new(),
            goals: vec![].into(),
            where_clause,
        }
    }
}

impl VarScope {
    pub fn new() -> VarScope {
        VarScope::default()
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            var_stack: vec![VarScope::new()],
            infer_stack: vec![InferScope::new(vec![])],
            report: Report::new(),
            type_impls: vec![],
            decls: declare::Context::default(),
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.type_scope().goals.insert(goal);
    }

    pub fn add_goals(&mut self, goals: Vec<Goal>) {
        for goal in goals {
            self.add_goal(goal);
        }
    }

    pub fn take_goals(&mut self) -> Set<Goal> {
        std::mem::replace(&mut self.type_scope().goals, Set::new())
            .into_iter()
            .map(|g| {
                let tr = g.tr.apply(self);
                let tr = tr.canonicalize(self);
                Goal::new(g.s, tr)
            })
            .collect::<Set<_>>()
    }

    pub fn assumptions(&self) -> Vec<Trait> {
        self.infer_stack
            .iter()
            .flat_map(|s| s.where_clause.clone())
            .collect()
    }

    pub fn infer(&mut self, p: &Program) -> Program {
        p.visit(&mut self.decls);
        self.map_program(p)
    }

    pub fn type_scope(&mut self) -> &mut InferScope {
        self.infer_stack.last_mut().unwrap()
    }

    pub fn get_value(&mut self, a: TypeVar) -> TypeVarValue {
        self.type_scope().table.probe_value(a)
    }

    pub fn union_value(&mut self, a: TypeVar, b: Type) {
        self.type_scope()
            .table
            .union_value(a, TypeVarValue::Known(b));
    }

    pub fn union(&mut self, a: TypeVar, b: TypeVar) {
        self.type_scope().table.union(a, b);
    }

    pub fn fresh(&mut self, kind: TypeVarKind) -> Type {
        Type::Var(self.type_scope().table.new_key(TypeVarValue::Unknown(kind)))
    }

    pub fn unify(&mut self, s0: Span, s1: Span, t0: &Type, t1: &Type) {
        t0.visit(&mut GatherGoals::new(self, s0));
        t1.visit(&mut GatherGoals::new(self, s1));
        let t0 = &t0.expand();
        let t1 = &t1.expand();
        let snapshot = self.type_scope().table.snapshot();
        if self.try_unify(&t0, &t1).is_ok() {
            self.type_scope().table.commit(snapshot)
        } else {
            self.type_scope().table.rollback_to(snapshot);
            let t0 = t0.apply(self);
            let t1 = t1.apply(self);
            self.report.err2(
                s0,
                s1,
                "Type mismatch",
                format!("Expected {t0}"),
                format!("Found {t1}"),
            );
        }
    }

    pub fn try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), ()> {
        match (t0, t1) {
            (Type::Var(x0), Type::Var(x1)) => match (self.get_value(*x0), self.get_value(*x1)) {
                (TypeVarValue::Known(t3), TypeVarValue::Known(t4)) => self.try_unify(&t3, &t4),
                (TypeVarValue::Known(t3), TypeVarValue::Unknown(k1))
                    if k1.is_unifiable_with(&t3) =>
                {
                    self.union_value(*x1, t3);
                    Ok(())
                }
                (TypeVarValue::Unknown(k0), TypeVarValue::Known(t4))
                    if k0.is_unifiable_with(&t4) =>
                {
                    self.union_value(*x0, t4);
                    Ok(())
                }
                (TypeVarValue::Unknown(k0), TypeVarValue::Unknown(k1)) if k0.is_compatible(k1) => {
                    if x0 != x1 {
                        self.union(*x0, *x1);
                    }
                    Ok(())
                }
                _ => Err(()),
            },
            (Type::Var(x1), t3) | (t3, Type::Var(x1)) => match self.get_value(*x1) {
                TypeVarValue::Known(t4) => self.try_unify(t3, &t4),
                TypeVarValue::Unknown(k1) if k1.is_unifiable_with(t3) => {
                    self.union_value(*x1, t3.clone());
                    Ok(())
                }
                _ => Err(()),
            },
            (Type::Cons(x0, ts0), Type::Cons(x1, ts1)) if x0 == x1 && ts0.len() == ts1.len() => ts0
                .iter()
                .zip(ts1.iter())
                .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
            (Type::Tuple(ts0), Type::Tuple(ts1)) if ts0.len() == ts1.len() => ts0
                .iter()
                .zip(ts1.iter())
                .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
            (Type::Fun(ts0, t0), Type::Fun(ts1, t1)) if ts0.len() == ts1.len() => ts0
                .iter()
                .chain([t0.as_ref()])
                .zip(ts1.iter().chain([t1.as_ref()]))
                .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
            (Type::Record(xts0), Type::Record(xts1)) if xts0.len() == xts1.len() => {
                let xts0 = xts0.sort_keys();
                let xts1 = xts1.sort_keys();
                if xts0.same_keys_sorted(&xts1) {
                    xts0.values()
                        .zip(xts1.values())
                        .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
                } else {
                    Err(())
                }
            }
            (Type::Generic(x0), Type::Generic(x1)) if x0 == x1 => Ok(()),
            (Type::Assoc(b0, x0, ts0), Type::Assoc(b1, x1, ts1))
                if x0 == x1 && ts0.len() == ts1.len() =>
            {
                match (b0, b1) {
                    (Trait::Type(t0), Trait::Type(t1)) => ts0
                        .iter()
                        .chain([t0.as_ref()])
                        .zip(ts1.iter().chain([t1.as_ref()]))
                        .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
                    (Trait::Cons(x0, ts0, _), Trait::Cons(x1, ts1, _))
                        if x0 == x1 && ts0.len() == ts1.len() =>
                    {
                        ts0.iter()
                            .zip(ts1.iter())
                            .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
                    }
                    (Trait::Err, _) | (_, Trait::Err) => Ok(()),
                    (Trait::Path(..), _) | (_, Trait::Path(..)) => unreachable!(),
                    _ => Err(()),
                }
            }
            (Type::Assoc(b, x, _), t0) | (t0, Type::Assoc(b, x, _)) => {
                if let Some(t1) = b.as_type(x) {
                    self.try_unify(t0, t1)
                } else {
                    Err(())
                }
            }
            (Type::Err, _) | (_, Type::Err) => Ok(()),
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            (Type::Unknown, Type::Unknown) => Ok(()),
            _ => Err(()),
        }
    }

    pub fn get(&self, x1: &Name) -> &(Span, Type) {
        self.var_stack
            .iter()
            .rev()
            .find_map(|s| s.0.iter().rev().find(|(x0, _)| x0 == x1).map(|(_, b)| b))
            .unwrap_or_else(|| panic!("Unknown variable {x1}"))
    }

    pub fn bind(&mut self, x: Name, b: (Span, Type)) {
        self.var_stack.last_mut().unwrap().0.insert(x, b);
    }

    fn solve_goals(&mut self) {
        let goals = self.take_goals();
        let assumptions = self.assumptions();
        for goal in goals {
            let tr = goal.tr.apply(self);
            let tr = tr.expand();
            match self.solve(&tr, &assumptions, false) {
                Err(_) => {
                    tr.defaults(self);
                    let tr = tr.apply(self);
                    let tr = tr.expand();
                    match self.solve(&tr, &assumptions, false) {
                        Ok(_) => {}
                        Err(TraitSolverError::Unsolved) => {
                            self.report.err(
                                goal.s,
                                "Trait is not implemented",
                                format!("Found no implementation for trait {}", tr.verbose()),
                            );
                            return;
                        }
                        Err(TraitSolverError::Ambiguous(cs)) => {
                            let s = cs
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join("\n");
                            self.report.err(
                                goal.s,
                                "Ambiguous trait implementation",
                                format!("Found multiple implementations for trait {tr}:\n{s}"),
                            );
                            return;
                        }
                    }
                }
                Ok(_) => {}
            }
            self.solve(&tr, &assumptions, true)
                .expect("Goals should be solved");
        }
    }

    fn solve(
        &mut self,
        goal: &Trait,
        assumptions: &[Trait],
        commit: bool,
    ) -> Result<Candidate, TraitSolverError> {
        let Trait::Cons(x, _, _) = goal else {
            todo!();
        };
        let mut solutions = vec![];
        for b in assumptions {
            if self.matches(goal, b, commit) {
                solutions.push(Candidate::Bound(b.clone()));
            }
        }
        if let Some(impls) = self.decls.trait_impls.get(x).cloned() {
            for stmt in impls {
                let gsub = stmt
                    .generics
                    .iter()
                    .map(|x| (*x, self.fresh(TypeVarKind::General)))
                    .collect::<Map<_, _>>();
                let stmt = stmt.instantiate(&gsub);
                let stmt = stmt.annotate(self);
                if self.matches(goal, &stmt.head, commit)
                    && stmt
                        .where_clause
                        .iter()
                        .all(|subgoal| self.solve(subgoal, assumptions, commit).is_ok())
                {
                    solutions.push(Candidate::Impl(stmt));
                }
            }
        }
        match solutions.len() {
            0 => Err(TraitSolverError::Unsolved),
            1 => Ok(solutions.pop().unwrap()),
            _ => Err(TraitSolverError::Ambiguous(solutions)),
        }
    }

    // Checks if i0 and i1 match
    fn matches(&mut self, b0: &Trait, b1: &Trait, commit: bool) -> bool {
        let snapshot = self.type_scope().table.snapshot();
        let b = match (b0, b1) {
            (Trait::Cons(x0, ts0, xts0), Trait::Cons(x1, ts1, xts1)) => {
                x0 == x1
                    && ts0.len() == ts1.len()
                    && ts0
                        .iter()
                        .zip(ts1.iter())
                        .all(|(t0, t1)| self.try_unify(t0, t1).is_ok())
                    && xts0.len() == xts1.len()
                    && xts0
                        .iter()
                        .zip(xts1.iter())
                        .all(|((x0, t0), (x1, t1))| x0 == x1 && { self.try_unify(t0, t1).is_ok() })
            }
            (Trait::Type(t0), Trait::Type(t1)) => self.try_unify(t0, t1).is_ok(),
            (Trait::Path(..), _) | (_, Trait::Path(..)) => unreachable!(),
            (Trait::Err, _) | (_, Trait::Err) => true,
            (Trait::Cons(..), Trait::Type(..)) | (Trait::Type(..), Trait::Cons(..)) => false,
            (Trait::Var(..), _) | (_, Trait::Var(..)) => todo!(),
        };
        if commit {
            self.type_scope().table.commit(snapshot);
        } else {
            self.type_scope().table.rollback_to(snapshot);
        }
        b
    }

    #[allow(unused)]
    fn debug(&mut self, s: &str) {
        println!("Debug ({s})");
        println!("* Substitutions:");
        for i in 0..self.type_scope().table.len() as u32 {
            let x = TypeVar(i);
            let xr = self.type_scope().table.find(x);
            let t = self.type_scope().table.probe_value(x);
            if xr != x {
                println!("    '{} -> '{}", x, xr);
            } else {
                match t {
                    TypeVarValue::Known(t) => {
                        println!("    '{} -> {}", x, t.verbose());
                    }
                    TypeVarValue::Unknown(k) => {
                        println!("    '{} -> {}", x, k);
                    }
                }
            }
        }
        println!("* Goals:");
        for goal in self.type_scope().goals.iter() {
            println!("    {}", goal.tr.verbose());
        }
        println!("* Assumptions:");
        for assumption in self.infer_stack.iter().rev().flat_map(|s| &s.where_clause) {
            println!("    {}", assumption.verbose());
        }
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.var_stack.push(VarScope::new());
    }

    fn exit_scope(&mut self) {
        self.var_stack.pop();
    }

    fn map_program(&mut self, program: &Program) -> Program {
        let program = program.annotate(self);
        let stmts = self.map_stmts(&program.stmts);
        let program = Program::new(program.span, stmts);
        let program = program.apply(self);
        program.gather_goals(self);
        self.solve_goals();
        program.defaults(self);
        let program = program.expand();
        let program = program.apply(self);
        program
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Trait(_) => s.clone(),
            Stmt::Struct(_) => s.clone(),
            Stmt::Enum(_) => s.clone(),
            Stmt::Type(_) => s.clone(),
            Stmt::Err(_) => s.clone(),
            _ => self._map_stmt(s),
        }
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.infer_stack
            .push(InferScope::new(s.where_clause.clone()));
        let generics = s.generics.clone();
        let head = s.head.clone();
        let defs = s
            .defs
            .iter()
            .map(|d| Rc::new(self.map_stmt_def(d)))
            .collect::<Vec<_>>();
        let types = s.types.clone();
        let where_clause = s.where_clause.clone();
        self.infer_stack.pop();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.infer_stack
                    .push(InferScope::new(s.where_clause.clone()));
                for (x, t) in &s.params {
                    self.bind(*x, (x.span, t.clone()));
                }
                let e = e.annotate(self);
                let e = self.map_expr(&e);
                self.unify(s.span, e.span_of(), &s.ty, e.type_of());
                self.solve_goals();
                let stmt = StmtDef::new(
                    s.span,
                    s.name,
                    s.generics.clone(),
                    s.params.clone(),
                    s.ty.clone(),
                    s.where_clause.clone(),
                    ExprBody::UserDefined(Rc::new(e)),
                );
                stmt.defaults(self);
                let s = stmt.apply(self);
                self.infer_stack.pop();
                s
            }
            ExprBody::Builtin(b) => StmtDef::new(
                s.span,
                s.name,
                s.generics.clone(),
                s.params.clone(),
                s.ty.clone(),
                s.where_clause.clone(),
                ExprBody::Builtin(b.clone()),
            ),
        }
    }

    fn map_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let e = self.map_expr(&s.expr);
        self.unify(s.span, e.span_of(), &s.ty, e.type_of());
        self.bind(s.name, (s.span, s.ty.clone()));
        StmtVar::new(e.span_of(), s.name, s.ty.clone(), e)
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Path(..) => unreachable!(),
            Expr::Int(s, t0, v) => Expr::Int(*s, t0.clone(), *v),
            Expr::Float(s, t0, v) => Expr::Float(*s, t0.clone(), *v),
            Expr::Bool(s, t0, v) => {
                let t1 = bool();
                self.unify(*s, *s, t0, &t1);
                Expr::Bool(*s, t0.clone(), *v)
            }
            Expr::Char(s, t0, v) => {
                let t1 = char();
                self.unify(*s, *s, t0, &t1);
                Expr::Char(*s, t0.clone(), *v)
            }
            Expr::String(s, t0, v) => {
                let t1 = string();
                self.unify(*s, *s, t0, &t1);
                Expr::String(*s, t0.clone(), *v)
            }
            Expr::Struct(s, t0, x, ts, xes) => {
                let stmt = self.decls.structs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts.clone())
                    .collect::<Map<_, _>>();
                for (x, e) in xes.clone() {
                    let t2 = stmt
                        .fields
                        .iter()
                        .find_map(|(y, t)| (*y == x).then_some(t))
                        .unwrap()
                        .instantiate(&gsub);
                    let e = self.map_expr(&e);
                    self.unify(e.span_of(), stmt.span, e.type_of(), &t2);
                }
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(*s, stmt.span, t0, &t1);
                Expr::Struct(*s, t0.clone(), *x, ts.clone(), xes.clone())
            }
            Expr::Enum(s, t0, x, ts, x1, e) => {
                let stmt = self.decls.enums.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts.clone())
                    .collect::<Map<_, _>>();
                let t2 = stmt
                    .variants
                    .iter()
                    .find_map(|(y, t)| (y == x1).then_some(t))
                    .unwrap()
                    .instantiate(&gsub);
                let e = self.map_expr(e);
                self.unify(e.span_of(), stmt.span, e.type_of(), &t2);
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(*s, stmt.span, t0, &t1);
                Expr::Enum(*s, t0.clone(), *x, ts.clone(), *x1, e.into())
            }
            Expr::Tuple(s, t0, es) => {
                let es = self.map_exprs(es);
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t1 = Type::Tuple(ts);
                self.unify(*s, *s, t0, &t1);
                Expr::Tuple(*s, t0.clone(), es)
            }
            Expr::Var(s, t0, x) => {
                let (s1, t1) = self.get(x).clone();
                self.unify(*s, s1, t0, &t1);
                Expr::Var(*s, t0.clone(), *x)
            }
            Expr::Def(s, t0, x, ts0) => {
                let stmt = self.decls.defs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts0.clone())
                    .collect::<Map<_, _>>();
                let preds = stmt
                    .where_clause
                    .iter()
                    .map(|p| Goal::new(*s, p.instantiate(&gsub)))
                    .collect::<Vec<_>>();
                self.add_goals(preds);
                let t2 = Type::Fun(
                    stmt.params.values().cloned().collect::<Vec<_>>(),
                    Rc::new(stmt.ty.clone()),
                )
                .instantiate(&gsub);
                self.unify(*s, stmt.span, t0, &t2);
                Expr::Def(*s, t0.clone(), *x, ts0.clone())
            }
            Expr::Call(s, t0, e, es) => {
                let es = self.map_exprs(es);
                let e = self.map_expr(e);
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t2 = Type::Fun(ts, Rc::new(t0.clone()));
                self.unify(*s, e.span_of(), e.type_of(), &t2);
                Expr::Call(*s, t0.clone(), Rc::new(e), es)
            }
            Expr::Block(s, t0, b) => {
                let b = self.map_block(b);
                self.unify(*s, e.span_of(), t0, b.expr.type_of());
                Expr::Block(*s, t0.clone(), b)
            }
            Expr::Field(s, t0, e, x) => {
                let e = self.map_expr(e);
                let t = e.type_of().apply(self);
                match t {
                    Type::Cons(x0, ts) => {
                        let stmt = self.decls.structs.get(&x0).unwrap().clone();
                        let gsub = stmt
                            .generics
                            .clone()
                            .into_iter()
                            .zip(ts.clone())
                            .collect::<Map<_, _>>();
                        let t1 = stmt
                            .fields
                            .iter()
                            .find_map(|(x1, t)| (x1 == x).then_some(t));
                        if let Some(t1) = t1 {
                            let t1 = t1.instantiate(&gsub);
                            self.unify(*s, e.span_of(), t0, &t1);
                            Expr::Field(*s, t0.clone(), e.into(), *x)
                        } else {
                            self.report.err(
                                *s,
                                "Unknown field",
                                format!("Field {x} not found in {x0}"),
                            );
                            Expr::Field(*s, Type::Err, e.into(), *x)
                        }
                    }
                    Type::Record(xts) => {
                        let t1 = xts.iter().find_map(|(x1, t)| (x1 == x).then_some(t));
                        if let Some(t1) = t1 {
                            self.unify(*s, e.span_of(), t0, t1);
                            Expr::Field(*s, t0.clone(), e.into(), *x)
                        } else {
                            let t = e.type_of().apply(self);
                            self.report.err(
                                *s,
                                "Unknown field",
                                format!("Field {x} not found in record {t}"),
                            );
                            Expr::Field(*s, Type::Err, e.into(), *x)
                        }
                    }
                    _ => {
                        let t = e.type_of().apply(self);
                        self.report.err(
                            *s,
                            "Unknown type",
                            format!("Type {t} must be known at this point."),
                        );
                        Expr::Field(*s, Type::Err, e.into(), *x)
                    }
                }
            }
            Expr::Index(s, t0, e, i) => {
                let e = self.map_expr(e);
                let t = e.type_of().apply(self);
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
                self.unify(*s, e.span_of(), t0, t1);
                Expr::Index(*s, t0.clone(), e.into(), *i)
            }
            Expr::Array(s, t0, es) => {
                let es = self.map_exprs(es);
                let t1 = es
                    .first()
                    .map(|e| e.type_of().clone())
                    .unwrap_or_else(|| self.fresh(TypeVarKind::General));
                es.iter()
                    .for_each(|e| self.unify(*s, e.span_of(), e.type_of(), &t1));
                let t2 = Type::Cons(Name::from("Array"), vec![t1]);
                self.unify(*s, *s, t0, &t2);
                Expr::Array(*s, t0.clone(), es)
            }
            Expr::Err(s, t0) => Expr::Err(*s, t0.clone()),
            Expr::Assign(s, t0, e0, e1) => {
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                self.unify(*s, e0.span_of(), e0.type_of(), e1.type_of());
                self.unify(*s, *s, t0, &unit());
                Expr::Assign(*s, t0.clone(), Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, _) => todo!(),
            Expr::Continue(_, _) => todo!(),
            Expr::Break(_, _) => todo!(),
            Expr::Fun(s, t0, xts0, t1, e0) => {
                self.enter_scope();
                for (x, t) in xts0 {
                    self.bind(*x, (x.span, t.clone()));
                }
                let e0 = self.map_expr(e0);
                let ts0 = xts0.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>();
                let t2 = Type::Fun(ts0, Rc::new(t1.clone()));
                self.unify(*s, e0.span_of(), e0.type_of(), &t1);
                self.unify(*s, *s, t0, &t2);
                self.exit_scope();
                Expr::Fun(*s, t0.clone(), xts0.clone(), t1.clone(), Rc::new(e0))
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(s, t0, e0, b) => {
                let e0 = self.map_expr(e0);
                let b = self.map_block(b);
                self.unify(*s, e0.span_of(), e0.type_of(), &bool());
                self.unify(*s, *s, t0, &unit());
                self.unify(*s, b.expr.span_of(), t0, b.expr.type_of());
                Expr::While(*s, t0.clone(), Rc::new(e0), b)
            }
            Expr::Record(s, t0, xes) => {
                let xes = xes
                    .iter()
                    .map(|(x, e)| (*x, self.map_expr(e)))
                    .collect::<Map<_, _>>();
                let xts = xes
                    .iter()
                    .map(|(x, e)| (*x, e.type_of().clone()))
                    .collect::<Map<_, _>>();
                let t1 = Type::Record(xts);
                self.unify(*s, *s, t0, &t1);
                Expr::Record(*s, t0.clone(), xes.clone())
            }
            Expr::Value(_, _) => unreachable!(),
            Expr::For(s, t0, x, e0, b) => {
                let e0 = self.map_expr(e0);
                let b = self.map_block(b);
                self.unify(*s, *s, t0, &unit());
                self.unify(*s, *s, t0, &b.expr.type_of());
                Expr::For(*s, t0.clone(), *x, Rc::new(e0), b)
            }
            Expr::TraitMethod(s, t0, b, x1, ts1) => match b {
                Trait::Path(_, _) => unreachable!(),
                Trait::Cons(x0, ts0, _) => {
                    let stmt0 = self.decls.traits.get(x0).unwrap();
                    let stmt1 = stmt0.get_def(*x1).unwrap();
                    let gsub = stmt0
                        .generics
                        .clone()
                        .into_iter()
                        .chain(stmt1.generics.clone())
                        .zip(ts0.clone().into_iter().chain(ts1.clone()))
                        .collect::<Map<_, _>>();
                    let t1 = Type::Fun(
                        stmt1.params.values().cloned().collect::<Vec<_>>(),
                        Rc::new(stmt1.ty.clone()),
                    )
                    .annotate(self)
                    .instantiate(&gsub);
                    // self.add_goal(Goal::new(*s, b.clone()));
                    self.unify(*s, *s, t0, &t1);
                    Expr::TraitMethod(*s, t0.clone(), b.clone(), *x1, ts1.clone())
                }
                Trait::Type(_) => todo!(),
                Trait::Err => todo!(),
                Trait::Var(_) => todo!(),
            },
            Expr::Unresolved(s, _t, _x, _ts) => {
                // Try to find a single implementation of a trait that provides the method
                self.report
                    .err(*s, "Unresolved method", "Could not resolve method call.");
                Expr::Err(*s, Type::Err)
            }
            Expr::Update(_, _, _, _, _) => todo!(),
            // These should ideally be unreachable, but covering all would require more passes
            Expr::Query(..) => unreachable!(),
            Expr::QueryInto(..) => unreachable!(),
            Expr::InfixBinaryOp(..) => unreachable!(),
            Expr::PrefixUnaryOp(..) => unreachable!(),
            Expr::PostfixUnaryOp(..) => unreachable!(),
            Expr::Annotate(..) => unreachable!(),
            Expr::Paren(..) => unreachable!(),
            Expr::Dot(..) => unreachable!(),
            Expr::IfElse(s, t, e0, b0, b1) => {
                let e0 = self.map_expr(e0);
                let b0 = self.map_block(b0);
                let b1 = self.map_block(b1);
                self.unify(*s, *s, e0.type_of(), &bool());
                self.unify(*s, b0.expr.span_of(), t, b0.expr.type_of());
                self.unify(*s, b1.expr.span_of(), t, b1.expr.type_of());
                Expr::IfElse(*s, t.clone(), Rc::new(e0.clone()), b0, b1)
            }
            Expr::IntSuffix(..) => unreachable!(),
            Expr::FloatSuffix(..) => unreachable!(),
            Expr::LetIn(..) => unreachable!(),
            Expr::Anonymous(..) => unreachable!(),
        }
    }
}
