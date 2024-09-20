pub mod annotate;
pub mod apply;
pub mod canonicalize;
pub mod defaults;
pub mod expand;
pub mod gather_goals;
pub mod impl_var;
pub mod instantiate;
pub mod intrinstics;
pub mod local;
pub mod type_var;
pub mod unify;

use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;
use impl_var::ImplVarValue;

use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::ImplVar;
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
use crate::traversal::visitor::Visitor;
use intrinstics::bool;
use intrinstics::char;
use intrinstics::string;
use intrinstics::unit;

use self::type_var::TypeVarKind;
use self::type_var::TypeVarValue;

#[derive(Debug)]
pub struct Context {
    expr_stack: Vec<ExprScope>,
    type_stack: Vec<TypeScope>,
    pub report: Report,
    pub decls: declare::Context,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum TraitSolverError {
    NoSolution,
    MultipleSolutions(Vec<Impl>),
}

#[derive(Default, Debug)]
pub struct ExprScope(Map<Name, (Span, Type)>);

#[derive(Debug)]
pub struct TypeScope {
    pub type_table: InPlaceUnificationTable<TypeVar>,
    pub impl_table: InPlaceUnificationTable<ImplVar>,
    constraints: Set<Constraint>,
    pub where_clause: Vec<Impl>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Constraint {
    // Generated when calling a function that has a where-clause.
    WhereClause(Span, Impl),
    // Generated when referring to an associated function.
    ExprAssoc(Span, Type, Impl, Name, Vec<Type>),
    // Generated when referring to an associated type.
    TypeAssoc(Span, Type, Impl, Name, Vec<Type>),
}

impl TypeScope {
    pub fn new(where_clause: Vec<Impl>) -> TypeScope {
        TypeScope {
            type_table: InPlaceUnificationTable::new(),
            impl_table: InPlaceUnificationTable::new(),
            constraints: vec![].into(),
            where_clause,
        }
    }
}

impl ExprScope {
    pub fn new() -> ExprScope {
        ExprScope::default()
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            expr_stack: vec![ExprScope::new()],
            type_stack: vec![TypeScope::new(vec![])],
            report: Report::new(),
            decls: declare::Context::default(),
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.type_scope().constraints.insert(constraint);
    }

    pub fn add_constraints(&mut self, constraints: Vec<Constraint>) {
        for constraint in constraints {
            self.add_constraint(constraint);
        }
    }

    pub fn take_constraints(&mut self) -> Set<Constraint> {
        std::mem::take(&mut self.type_scope().constraints)
            .into_iter()
            .map(|c| c.apply(self).canonicalize(self))
            .collect::<Set<_>>()
    }

    pub fn assumptions(&self) -> Vec<Impl> {
        self.type_stack
            .iter()
            .flat_map(|s| s.where_clause.clone())
            .collect()
    }

    pub fn infer(&mut self, p: &Program) -> Program {
        p.visit(&mut self.decls);
        self.map_program(p)
    }

    pub fn type_scope(&mut self) -> &mut TypeScope {
        self.type_stack.last_mut().unwrap()
    }

    pub fn get_type_value(&mut self, a: TypeVar) -> TypeVarValue {
        self.type_scope().type_table.probe_value(a)
    }

    pub fn get_impl_value(&mut self, a: ImplVar) -> ImplVarValue {
        self.type_scope().impl_table.probe_value(a)
    }

    pub fn union_impl_value(&mut self, a: ImplVar, b: Impl) {
        self.type_scope()
            .impl_table
            .union_value(a, ImplVarValue::Known(b));
    }

    pub fn union_type_value(&mut self, a: TypeVar, b: Type) {
        self.type_scope()
            .type_table
            .union_value(a, TypeVarValue::Known(b));
    }

    pub fn union(&mut self, a: TypeVar, b: TypeVar) {
        self.type_scope().type_table.union(a, b);
    }

    pub fn fresh_tv(&mut self, kind: TypeVarKind) -> Type {
        Type::Var(
            self.type_scope()
                .type_table
                .new_key(TypeVarValue::Unknown(kind)),
        )
    }

    pub fn fresh_tvs(&mut self, n: usize) -> Vec<Type> {
        (0..n)
            .map(|_| self.fresh_tv(TypeVarKind::General))
            .collect()
    }

    pub fn fresh_iv(&mut self) -> Impl {
        Impl::Var(self.type_scope().impl_table.new_key(ImplVarValue::Unknown))
    }

    pub fn unify(&mut self, s0: Span, s1: Span, t0: &Type, t1: &Type) {
        t0.gather_constraints(self);
        t1.gather_constraints(self);
        let t0 = &t0.expand();
        let t1 = &t1.expand();
        let snapshot = self.type_scope().type_table.snapshot();
        if self.try_unify(&t0, &t1).is_ok() {
            self.type_scope().type_table.commit(snapshot)
        } else {
            self.type_scope().type_table.rollback_to(snapshot);
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
            (Type::Var(x0), Type::Var(x1)) => {
                match (self.get_type_value(*x0), self.get_type_value(*x1)) {
                    (TypeVarValue::Known(t3), TypeVarValue::Known(t4)) => self.try_unify(&t3, &t4),
                    (TypeVarValue::Known(t3), TypeVarValue::Unknown(k1))
                        if k1.is_unifiable_with(&t3) =>
                    {
                        self.union_type_value(*x1, t3);
                        Ok(())
                    }
                    (TypeVarValue::Unknown(k0), TypeVarValue::Known(t4))
                        if k0.is_unifiable_with(&t4) =>
                    {
                        self.union_type_value(*x0, t4);
                        Ok(())
                    }
                    (TypeVarValue::Unknown(k0), TypeVarValue::Unknown(k1))
                        if k0.is_compatible(k1) =>
                    {
                        if x0 != x1 {
                            self.union(*x0, *x1);
                        }
                        Ok(())
                    }
                    _ => Err(()),
                }
            }
            (Type::Var(x1), t3) | (t3, Type::Var(x1)) => match self.get_type_value(*x1) {
                TypeVarValue::Known(t4) => self.try_unify(t3, &t4),
                TypeVarValue::Unknown(k1) if k1.is_unifiable_with(t3) => {
                    self.union_type_value(*x1, t3.clone());
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
            (Type::Lambda(ts0, t0), Type::Lambda(ts1, t1)) if ts0.len() == ts1.len() => ts0
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
                    (Impl::Type(t0), Impl::Type(t1)) => ts0
                        .iter()
                        .chain([t0.as_ref()])
                        .zip(ts1.iter().chain([t1.as_ref()]))
                        .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
                    (Impl::Trait(i0), Impl::Trait(i1))
                        if i0.x == i1.x && i0.ts.len() == i1.ts.len() =>
                    {
                        ts0.iter()
                            .zip(ts1.iter())
                            .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
                    }
                    (Impl::Err, _) | (_, Impl::Err) => Ok(()),
                    (Impl::Path(..), _) | (_, Impl::Path(..)) => unreachable!(),
                    _ => Err(()),
                }
            }
            (Type::Assoc(b, x, _), t0) | (t0, Type::Assoc(b, x, _)) => {
                if let Some(t1) = b.as_trait().unwrap().xts.get(x) {
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
        self.expr_stack
            .iter()
            .rev()
            .find_map(|s| s.0.iter().rev().find(|(x0, _)| x0 == x1).map(|(_, b)| b))
            .unwrap_or_else(|| panic!("Unknown variable {x1}"))
    }

    pub fn bind(&mut self, x: Name, b: (Span, Type)) {
        self.expr_stack.last_mut().unwrap().0.insert(x, b);
    }

    fn solve_constraints(&mut self) {
        let mut constraints = self.take_constraints();
        let assumptions = self.assumptions();
        let mut retries = vec![];
        let mut fuel = 0;
        loop {
            let num_constraints = constraints.len();
            for constraint in constraints {
                if fuel == 1 {
                    // Retry by applying defaults.
                    constraint.defaults(self);
                } else if fuel > 100 {
                    self.report.err(
                        *constraint.span_of(),
                        "Infinite loop detected",
                        "Unable to solve constraints",
                    );
                    return;
                }
                match self.solve_constraint(&constraint, &assumptions, false) {
                    Ok(_) => {
                        self.solve_constraint(&constraint, &assumptions, true)
                            .expect("Constraint should be solvable");
                    }
                    Err(TraitSolverError::NoSolution) => {
                        self.report.err(
                            *constraint.span_of(),
                            "Unsatisfiable trait constraint",
                            format!("No solution found for constraint {constraint}"),
                        );
                    }
                    Err(TraitSolverError::MultipleSolutions(candidates)) => {
                        retries.push((constraint, candidates));
                    }
                }
            }
            if retries.is_empty() || num_constraints == retries.len() {
                for (constraint, candidates) in retries {
                    let msg = candidates
                        .iter()
                        .enumerate()
                        .map(|(i, c)| format!("{}: {}", i + 1, c))
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.report.err(
                        *constraint.span_of(),
                        "Ambiguous trait implementation",
                        format!("Found multiple solutions for constraint {constraint}:\n{msg}"),
                    );
                }
                break;
            } else {
                fuel += 1;
                constraints = retries.drain(..).map(|(c, _)| c).collect();
            }
        }
    }

    fn solve_constraint(
        &mut self,
        constraint: &Constraint,
        assumptions: &[Impl],
        commit: bool,
    ) -> Result<Impl, TraitSolverError> {
        let constraint = constraint.apply(self).expand();
        match &constraint {
            Constraint::WhereClause(_, i) => {
                let tr = i.as_trait().unwrap();
                self.solve_trait_impl(tr, assumptions, commit)
            }
            Constraint::ExprAssoc(_, def_type, i, def_name, def_type_args) => match i {
                Impl::Path(..) => unreachable!(),
                Impl::Trait(impl_trait) => self.solve_trait_impl_def(
                    impl_trait,
                    def_type,
                    def_name,
                    def_type_args,
                    assumptions,
                    commit,
                ),
                Impl::Var(v) => {
                    if let ImplVarValue::Known(i) = self.get_impl_value(*v) {
                        Ok(i)
                    } else {
                        let i1 =
                            self.solve_def(def_type, def_name, def_type_args, assumptions, commit)?;
                        self.union_impl_value(*v, i1.clone());
                        Ok(i1)
                    }
                }
                Impl::Type(impl_type) => self.solve_type_impl_def(
                    impl_type,
                    def_type,
                    def_name,
                    def_type_args,
                    assumptions,
                    commit,
                ),
                Impl::Unknown => unreachable!(),
                Impl::Err => Err(TraitSolverError::NoSolution),
            },
            Constraint::TypeAssoc(_, _, _, _, _) => {
                todo!();
            }
        }
    }

    fn solve_def(
        &mut self,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        assumptions: &[Impl],
        commit: bool,
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];
        for impl_stmt in self.decls.type_impls.clone() {
            let Some(def_stmt) = impl_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            self.transaction(commit, |this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);
                (this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&impl_stmt.where_clause, assumptions, commit)
                    && this.solve_where_clauses(&def_stmt.where_clause, assumptions, commit))
                .then(|| solutions.push(impl_stmt.head))
                .is_some()
            })
        }

        if let Ok(candidate) = self.check_solutions(solutions) {
            return Ok(candidate);
        } else {
            solutions = vec![]
        }

        for (trait_name, trait_stmt) in self.decls.traits.clone() {
            let Some(def_stmt) = trait_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }

            let impl_stmts = self
                .decls
                .trait_impls
                .get(&trait_name)
                .cloned()
                .unwrap_or_default();

            for impl_stmt in impl_stmts.clone() {
                self.transaction(commit, |this| {
                    let ts = this.fresh_tvs(impl_stmt.generics.len());
                    let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                    let def_stmt = impl_stmt
                        .get_def(def_name)
                        .unwrap()
                        .instantiate(&def_type_args);
                    (this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                        && this.solve_where_clauses(&def_stmt.where_clause, assumptions, commit)
                        && this.solve_where_clauses(&impl_stmt.where_clause, assumptions, commit))
                    .then(|| solutions.push(impl_stmt.head))
                    .is_some()
                })
            }
        }

        self.check_solutions(solutions)
    }

    fn solve_type_impl_def(
        &mut self,
        impl_type0: &Type,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        assumptions: &[Impl],
        commit: bool,
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];
        for impl_stmt in self.decls.type_impls.clone() {
            let Some(def_stmt) = impl_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            self.transaction(commit, |this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);
                let impl_type1 = impl_stmt.head.as_type().unwrap();
                (this.try_unify(impl_type0, impl_type1).is_ok()
                    && this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&impl_stmt.where_clause, assumptions, commit)
                    && this.solve_where_clauses(&def_stmt.where_clause, assumptions, commit))
                .then(|| solutions.push(impl_stmt.head))
                .is_some()
            })
        }

        self.check_solutions(solutions)
    }

    fn solve_where_clauses(
        &mut self,
        where_clause: &[Impl],
        assumptions: &[Impl],
        commit: bool,
    ) -> bool {
        where_clause.iter().all(|i| {
            let tr = i.as_trait().unwrap();
            self.solve_trait_impl(tr, assumptions, commit).is_ok()
        })
    }

    fn solve_trait_impl_def(
        &mut self,
        impl_trait0: &Trait,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        assumptions: &[Impl],
        commit: bool,
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

        let impl_stmts = self
            .decls
            .trait_impls
            .get(&impl_trait0.x)
            .cloned()
            .unwrap_or_default();

        for impl_stmt in impl_stmts {
            self.transaction(commit, |this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);
                let impl_trait1 = impl_stmt.head.as_trait().unwrap();
                (this.traits_match(impl_trait0, impl_trait1)
                    && this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&impl_stmt.where_clause, assumptions, commit)
                    && this.solve_where_clauses(&def_stmt.where_clause, assumptions, commit))
                .then(|| solutions.push(impl_stmt.head))
                .is_some()
            })
        }

        self.check_solutions(solutions)
    }

    // Solve a trait impl constraint. This occurs for example when calling a function with
    // where-clauses, e.g., def f[T](x:T) where Trait[T] = ...; f(1);
    fn solve_trait_impl(
        &mut self,
        impl_trait0: &Trait,
        assumptions: &[Impl],
        commit: bool,
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

        for i in assumptions {
            self.transaction(commit, |this| {
                let impl_trait1 = i.as_trait().unwrap();
                this.traits_match(impl_trait0, impl_trait1)
                    .then(|| solutions.push(i.clone()))
                    .is_some()
            })
        }

        let impl_stmts = self
            .decls
            .trait_impls
            .get(&impl_trait0.x)
            .cloned()
            .unwrap_or_default();

        for impl_stmt in impl_stmts {
            self.transaction(commit, |this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let tr1 = impl_stmt.head.as_trait().unwrap();
                (this.traits_match(impl_trait0, tr1)
                    && this.solve_where_clauses(&impl_stmt.where_clause, assumptions, commit))
                .then(|| solutions.push(impl_stmt.head))
                .is_some()
            })
        }

        self.check_solutions(solutions)
    }

    /// Check if two traits are unifiable
    fn traits_match(&mut self, tr0: &Trait, tr1: &Trait) -> bool {
        tr0.x == tr1.x
            && tr0.ts.len() == tr1.ts.len()
            && tr0
                .ts
                .iter()
                .zip(tr1.ts.iter())
                .all(|(t0, t1)| self.try_unify(t0, t1).is_ok())
            && tr0.xts.len() == tr1.xts.len()
            && tr0
                .xts
                .iter()
                .zip(tr1.xts.iter())
                .all(|((x0, t0), (x1, t1))| x0 == x1 && { self.try_unify(t0, t1).is_ok() })
    }

    fn check_solutions(&self, solutions: Vec<Impl>) -> Result<Impl, TraitSolverError> {
        match solutions.len() {
            0 => Err(TraitSolverError::NoSolution),
            1 => Ok(solutions.first().unwrap().clone()),
            _ => Err(TraitSolverError::MultipleSolutions(solutions)),
        }
    }

    fn transaction(&mut self, commit: bool, f: impl FnOnce(&mut Self) -> bool) {
        let snapshot = self.type_scope().type_table.snapshot();
        if f(self) && commit {
            self.type_scope().type_table.commit(snapshot);
        } else {
            self.type_scope().type_table.rollback_to(snapshot);
        }
    }

    #[allow(unused)]
    fn debug(&mut self, label: &str) {
        println!("Debug ({label})");
        println!("* Substitutions:");
        for i in 0..self.type_scope().type_table.len() as u32 {
            let x = TypeVar(i);
            let xr = self.type_scope().type_table.find(x);
            let t = self.type_scope().type_table.probe_value(x);
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
        println!("* Constraints:");
        for constraint in self.type_scope().constraints.iter() {
            println!("    {}", constraint);
        }
        println!("* Assumptions:");
        for assumption in self.type_stack.iter().rev().flat_map(|s| &s.where_clause) {
            println!("    {}", assumption.verbose());
        }
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.expr_stack.push(ExprScope::new());
    }

    fn exit_scope(&mut self) {
        self.expr_stack.pop();
    }

    fn map_program(&mut self, program: &Program) -> Program {
        let program = program.annotate(self);
        let stmts = self.map_stmts(&program.stmts);
        let program = Program::new(program.span, stmts);
        let p = program.apply(self);
        self.solve_constraints();
        p.defaults(self);
        let p = p.expand();
        let p = p.apply(self);
        p
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Trait(_) => s.clone(),
            Stmt::Struct(_) => s.clone(),
            Stmt::Enum(_) => s.clone(),
            Stmt::Type(_) => s.clone(),
            Stmt::Err(_) => s.clone(),
            Stmt::Expr(e) => {
                self.visit_expr(e);
                Stmt::Expr(e.clone())
            }
            _ => self._map_stmt(s),
        }
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.type_stack.push(TypeScope::new(s.where_clause.clone()));
        let generics = s.generics.clone();
        let head = s.head.clone();
        let defs = s
            .defs
            .iter()
            .map(|d| Rc::new(self.map_stmt_def(d)))
            .collect::<Vec<_>>();
        let types = s.types.clone();
        let where_clause = s.where_clause.clone();
        self.type_stack.pop();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.type_stack.push(TypeScope::new(s.where_clause.clone()));
                for (x, t) in &s.params {
                    self.bind(*x, (x.span, t.clone()));
                }
                let e = e.annotate(self);
                self.visit_expr(&e);
                self.unify(s.span, e.span_of(), &s.ty, e.type_of());
                self.solve_constraints();
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
                self.type_stack.pop();
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
        self.visit_expr(&s.expr);
        self.unify(s.span, s.expr.span_of(), &s.ty, s.expr.type_of());
        self.bind(s.name, (s.span, s.ty.clone()));
        StmtVar::new(s.expr.span_of(), s.name, s.ty.clone(), s.expr.clone())
    }
}

impl Visitor for Context {
    fn visit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Path(..) => unreachable!(),
            Expr::Int(_, _, _) => {}
            Expr::Float(_, _, _) => {}
            Expr::Bool(s, t0, _) => {
                let t1 = bool();
                self.unify(*s, *s, t0, &t1);
            }
            Expr::Char(s, t0, _) => {
                let t1 = char();
                self.unify(*s, *s, t0, &t1);
            }
            Expr::String(s, t0, _) => {
                let t1 = string();
                self.unify(*s, *s, t0, &t1);
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
                    self.visit_expr(&e);
                    self.unify(e.span_of(), stmt.span, e.type_of(), &t2);
                }
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(*s, stmt.span, t0, &t1);
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
                self.visit_expr(e);
                self.unify(e.span_of(), stmt.span, e.type_of(), &t2);
                let t1 = Type::Cons(*x, ts.clone());
                self.unify(*s, stmt.span, t0, &t1);
            }
            Expr::Tuple(s, t0, es) => {
                self.visit_exprs(es);
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t1 = Type::Tuple(ts);
                self.unify(*s, *s, t0, &t1);
            }
            Expr::Var(s, t0, x) => {
                let (s1, t1) = self.get(x).clone();
                self.unify(*s, s1, t0, &t1);
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
                    .map(|p| Constraint::WhereClause(*s, p.instantiate(&gsub)))
                    .collect::<Vec<_>>();
                self.add_constraints(preds);
                let t2 = Type::Lambda(
                    stmt.params.values().cloned().collect::<Vec<_>>(),
                    Rc::new(stmt.ty.clone()),
                )
                .instantiate(&gsub);
                self.unify(*s, stmt.span, t0, &t2);
            }
            Expr::Call(s, t0, e1, es) => {
                let ts = es.iter().map(|e| e.type_of().clone()).collect::<Vec<_>>();
                let t2 = Type::Lambda(ts, Rc::new(t0.clone()));
                self.unify(*s, e1.span_of(), e1.type_of(), &t2);
                self.visit_expr(e1);
                self.visit_exprs(es);
            }
            Expr::Block(s, t0, b) => {
                self.visit_block(b);
                self.unify(*s, e.span_of(), t0, b.expr.type_of());
            }
            Expr::Field(s, t0, e, x) => {
                self.visit_expr(e);
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
                        } else {
                            self.report.err(
                                *s,
                                "Unknown field",
                                format!("Field {x} not found in {x0}"),
                            );
                            self.unify(*s, e.span_of(), t0, &Type::Err);
                        }
                    }
                    Type::Record(xts) => {
                        let t1 = xts.iter().find_map(|(x1, t)| (x1 == x).then_some(t));
                        if let Some(t1) = t1 {
                            self.unify(*s, e.span_of(), t0, t1);
                        } else {
                            let t = e.type_of().apply(self);
                            self.report.err(
                                *s,
                                "Unknown field",
                                format!("Field {x} not found in record {t}"),
                            );
                            self.unify(*s, e.span_of(), t0, &Type::Err);
                        }
                    }
                    _ => {
                        let t = e.type_of().apply(self);
                        self.report.err(
                            *s,
                            "Unknown type",
                            format!("Type {t} must be known at this point."),
                        );
                        self.unify(*s, e.span_of(), t0, &Type::Err);
                    }
                }
            }
            Expr::Index(s, t0, e, i) => {
                self.visit_expr(e);
                let t = e.type_of().apply(self);
                let Type::Tuple(ts) = &t else {
                    self.report.err(
                        *s,
                        "Unknown type",
                        format!("Type {t} must be known at this point."),
                    );
                    return;
                };
                let Some(t1) = ts.get(i.data) else {
                    self.report.err(
                        *s,
                        format!("Index {i} out of bounds ({i} >= {})", ts.len()),
                        format!("Index {i} out of bounds."),
                    );
                    return;
                };
                self.unify(*s, e.span_of(), t0, t1);
            }
            Expr::Array(s, t0, es) => {
                self.visit_exprs(es);
                let t1 = es
                    .first()
                    .map(|e| e.type_of().clone())
                    .unwrap_or_else(|| self.fresh_tv(TypeVarKind::General));
                for e in es.iter() {
                    self.unify(*s, e.span_of(), e.type_of(), &t1)
                }
                let t2 = Type::Cons(Name::from("Array"), vec![t1]);
                self.unify(*s, *s, t0, &t2);
            }
            Expr::Err(_, _) => {}
            Expr::Assign(s, t0, e0, e1) => {
                self.visit_expr(e0);
                self.visit_expr(e1);
                self.unify(*s, e0.span_of(), e0.type_of(), e1.type_of());
                self.unify(*s, *s, t0, &unit());
            }
            Expr::Return(_, _, _) => todo!(),
            Expr::Continue(_, _) => todo!(),
            Expr::Break(_, _) => todo!(),
            Expr::Lambda(s, t0, xts0, t1, e0) => {
                self.enter_scope();
                for (x, t) in xts0 {
                    self.bind(*x, (x.span, t.clone()));
                }
                let ts0 = xts0.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>();
                let t2 = Type::Lambda(ts0, Rc::new(t1.clone()));
                self.unify(*s, *s, t0, &t2);
                self.visit_expr(e0);
                self.unify(*s, e0.span_of(), &t1, e0.type_of());
                self.exit_scope();
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(s, t0, e0, b) => {
                self.visit_expr(e0);
                self.visit_block(b);
                self.unify(*s, e0.span_of(), e0.type_of(), &bool());
                self.unify(*s, *s, t0, &unit());
                self.unify(*s, b.expr.span_of(), t0, b.expr.type_of());
            }
            Expr::Record(s, t0, xes) => {
                xes.iter().for_each(|(_, e)| self.visit_expr(e));
                let xts = xes
                    .iter()
                    .map(|(x, e)| (*x, e.type_of().clone()))
                    .collect::<Map<_, _>>();
                let t1 = Type::Record(xts);
                self.unify(*s, *s, t0, &t1);
            }
            Expr::For(s, t0, _, e0, b) => {
                self.visit_expr(e0);
                self.visit_block(b);
                self.unify(*s, *s, t0, &unit());
                self.unify(*s, *s, t0, &b.expr.type_of());
            }
            Expr::Assoc(s, t, i, x1, ts1) => {
                self.type_scope().constraints.insert(Constraint::ExprAssoc(
                    *s,
                    t.clone(),
                    i.clone(),
                    *x1,
                    ts1.clone(),
                ));
            }
            Expr::Update(_, _, _, _, _) => todo!(),
            // TODO: Desugar all in previous pass.
            Expr::Query(..) => unreachable!(),
            Expr::QueryInto(..) => unreachable!(),
            Expr::InfixBinaryOp(..) => unreachable!(),
            Expr::PrefixUnaryOp(..) => unreachable!(),
            Expr::PostfixUnaryOp(..) => unreachable!(),
            Expr::Annotate(..) => unreachable!(),
            Expr::Paren(..) => unreachable!(),
            Expr::Dot(..) => unreachable!(),
            Expr::IfElse(s, t, e0, b0, b1) => {
                self.visit_expr(e0);
                self.visit_block(b0);
                self.visit_block(b1);
                self.unify(*s, *s, e0.type_of(), &bool());
                self.unify(*s, b0.expr.span_of(), t, b0.expr.type_of());
                self.unify(*s, b1.expr.span_of(), t, b1.expr.type_of());
            }
            Expr::IntSuffix(..) => unreachable!(),
            Expr::FloatSuffix(..) => unreachable!(),
            Expr::LetIn(..) => unreachable!(),
            Expr::Anonymous(..) => unreachable!(),
            Expr::Closure(_, _, _xts0, _xts1, _t, _e) => {
                todo!()
            }
        }
    }
}
