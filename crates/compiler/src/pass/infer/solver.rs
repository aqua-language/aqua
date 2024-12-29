use crate::ast::Impl;
use crate::ast::Name;
use crate::ast::Trait;
use crate::ast::Type;
use crate::collections::set::Set;
use crate::syntax::span::Span;

use super::Context;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Constraint {
    // Generated when calling a function that has a where-clause.
    WhereClause(Span, Impl),
    // Generated when referring to an associated function.
    AssocDef(Span, Type, Impl, Name, Vec<Type>),
    // Generated when referring to an associated type.
    AssocType(Span, Type, Impl, Name, Vec<Type>),
    // Generated when referring to a field.
    Field(Span, Type, Type, Name),
}

#[derive(Debug)]
enum Error {
    Fatal(FatalError),
    Retry(RetryError),
}

impl From<FatalError> for Error {
    fn from(e: FatalError) -> Self {
        Error::Fatal(e)
    }
}

impl From<RetryError> for Error {
    fn from(e: RetryError) -> Self {
        Error::Retry(e)
    }
}

#[derive(Debug)]
enum FatalError {
    ImplNotFound,
    FieldNotFound(Name, Type),
    NotAStruct(Type),
}

#[derive(Debug)]
enum RetryError {
    MultipleImplsFound(Vec<Impl>),
    StructNotFound,
    MaxDepth,
}

impl Context {
    pub fn solve_constraints(&mut self, span: Span) {
        let mut constraints = self.take_constraints();
        let premises = self.premises();
        let mut retries = vec![];
        let mut fuel = 0;
        loop {
            constraints = constraints
                .iter()
                .map(|c| c.apply(self).expand().canonicalize(self))
                .collect::<Set<_>>();
            // Try to solve all constraints
            let mut num_solved = 0;
            for c in constraints {
                match self.with_rollback(|this| this.solve_constraint(&c, &premises)) {
                    Ok(()) => {
                        num_solved += 1;
                        self.solve_constraint(&c, &premises)
                            .expect("Constraint should be solvable");
                    }
                    Err(e @ Error::Fatal(_)) => {
                        let c = c.apply(self);
                        self.report_solver_error(&c, &e);
                    }
                    Err(e @ Error::Retry(_)) => {
                        retries.push((c, e));
                    }
                }
            }
            // If all constraints are solved, we are done.
            if retries.is_empty() {
                break;
            }
            // If we did not solve any constraints, we failed.
            if num_solved == 0 && fuel > 0 {
                retries
                    .iter()
                    .for_each(|(c, e)| self.report_solver_error(c, e));
                break;
            }
            if fuel == 100 {
                self.report.err(
                    span,
                    "Infinite loop detected",
                    "Unable to solve constraints",
                );
                break;
            }
            // Retry solving the constraints that we failed to solve.
            constraints = retries.drain(..).map(|(c, _)| c).collect();
            if fuel == 0 {
                // Retry by applying defaults.
                constraints.iter().for_each(|c| c.defaults(self));
            }
            fuel += 1;
        }
    }

    fn report_solver_error(&mut self, c: &Constraint, e: &Error) {
        match e {
            Error::Retry(RetryError::MultipleImplsFound(candidates)) => {
                let msg = candidates
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}: {}", i + 1, c))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.report.err(
                    *c.span(),
                    "Ambiguous trait implementation",
                    format!("Found multiple solutions for constraint {c}:\n{msg}"),
                );
            }
            Error::Retry(RetryError::MaxDepth) => {
                self.report.err(
                    *c.span(),
                    "Trait solver timed out",
                    "The trait solver reached the maximum recursion depth",
                );
            }
            Error::Retry(RetryError::StructNotFound) => {
                self.report.err(
                    *c.span(),
                    "Could not infer which struct is being accessed.",
                    "Please provide a type annotation.",
                );
            }
            Error::Fatal(FatalError::ImplNotFound) => {
                self.report.err(
                    *c.span(),
                    "Unsatisfiable trait constraint",
                    format!("No solution found for constraint {c}"),
                );
            }
            Error::Fatal(FatalError::NotAStruct(t)) => {
                self.report.err(
                    *c.span(),
                    "Not a struct or record",
                    format!("Attempted to index into {t} which is not a struct or record"),
                );
            }
            Error::Fatal(FatalError::FieldNotFound(x, t)) => {
                let t = t.apply(self);
                self.report.err(
                    *c.span(),
                    "Field not found",
                    format!("Field {x} not found in type {t}"),
                );
            }
        }
    }

    fn with_rollback<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.rollback = true;
        let v = f(self);
        self.rollback = false;
        v
    }

    fn solve_constraint(
        &mut self,
        constraint: &Constraint,
        premises: &[Impl],
    ) -> Result<(), Error> {
        match &constraint {
            Constraint::WhereClause(_, i) => {
                let tr = i.as_trait().unwrap();
                self.solve_trait_impl(tr, premises).map(|_| ())
            }
            Constraint::AssocDef(_, def_type, imp, def_name, def_type_args) => self
                .solve_assoc_def(&def_type, imp, def_name, def_type_args)
                .map(|_| ()),
            Constraint::AssocType(_, _, _, _, _) => {
                todo!();
            }
            Constraint::Field(s, t0, t1, x) => self.solve_field(s, t0, t1, x).map(|_| ()),
        }
    }

    fn solve_assoc_def(
        &mut self,
        def_type: &Type,
        imp: &Impl,
        def_name: &Name,
        def_type_args: &[Type],
    ) -> Result<(), Error> {
        match imp {
            Impl::Trait(impl_trait) => self
                .solve_trait_impl_def(impl_trait, def_type, def_name, def_type_args, &[])
                .map(|_| ()),
            Impl::Var(v) => {
                if self.get_impl_value(*v).is_unknown() {
                    let i = self.solve_def(def_type, def_name, def_type_args, &[])?;
                    if !self.rollback {
                        self.union_impl_value(*v, i.clone());
                    }
                }
                Ok(())
            }
            Impl::Type(impl_type) => self
                .solve_type_impl_def(impl_type, def_type, def_name, def_type_args, &[])
                .map(|_| ()),
            Impl::Err => Ok(()),
            Impl::Unknown => unreachable!(),
            Impl::Path(..) => unreachable!(),
        }
    }

    fn solve_field(&mut self, s: &Span, t0: &Type, t1: &Type, x: &Name) -> Result<(), Error> {
        match t1 {
            Type::Struct(x0, ts) => {
                if let Some(stmt) = self.decls.structs.get(&x0) {
                    let stmt = stmt.clone().instantiate(ts);
                    let t2 = stmt
                        .fields
                        .iter()
                        .find_map(|(x1, t)| (x1 == x).then_some(t));
                    if let Some(t2) = t2 {
                        if self.rollback {
                            self.try_unify(t0, t2).ok();
                        } else {
                            self.unify(*s, *s, t0, t2);
                        }
                        Ok(())
                    } else {
                        Err(FatalError::FieldNotFound(*x, t1.clone()).into())
                    }
                } else {
                    Err(FatalError::NotAStruct(t1.clone()).into())
                }
            }
            Type::Record(xts) => {
                let t2 = xts.iter().find_map(|(x1, t)| (x1 == x).then_some(t));
                if let Some(t2) = t2 {
                    if self.rollback {
                        self.try_unify(t0, t2).ok();
                    } else {
                        self.unify(*s, *s, t0, t2);
                    }
                    Ok(())
                } else {
                    Err(FatalError::FieldNotFound(*x, t1.clone()).into())
                }
            }
            Type::Var(_) => Err(RetryError::StructNotFound.into()),
            _ => Err(FatalError::NotAStruct(t1.clone()).into()),
        }
    }

    // ::def_name[def_type_args] : def_type
    fn solve_def(
        &mut self,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        premises: &[Impl],
    ) -> Result<Impl, Error> {
        let mut impls = vec![];

        for i in premises {
            let impl_trait0 = i.as_trait().unwrap();
            let stmt_trait = self.decls.traits.get(&impl_trait0.x).unwrap().clone();
            let Some(def_stmt) = stmt_trait.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            self.transaction(|this| {
                let ts = this.fresh_tvs(stmt_trait.generics.len());
                let stmt_trait = stmt_trait.instantiate(&ts).annotate(this);
                let def_stmt = stmt_trait
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);

                let satisfied = this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                    && this.solve_where_clauses(&stmt_trait.where_clause, premises)?;

                if satisfied {
                    impls.push(i.clone());
                }

                Ok(satisfied)
            })?;
        }

        for impl_stmt in self.decls.type_impls.clone() {
            let Some(def_stmt) = impl_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            self.transaction(|this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);

                let satisfied = this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                    && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                if satisfied {
                    impls.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        if let Ok(candidate) = self.unique_impl(impls) {
            return Ok(candidate);
        } else {
            impls = vec![]
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
                self.transaction(|this| {
                    let ts = this.fresh_tvs(impl_stmt.generics.len());
                    let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                    let def_stmt = impl_stmt
                        .get_def(def_name)
                        .unwrap()
                        .instantiate(&def_type_args);

                    let satisfied = this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                        && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                        && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                    if satisfied {
                        impls.push(impl_stmt.head);
                    }

                    Ok(satisfied)
                })?;
            }
        }

        self.unique_impl(impls)
    }

    fn solve_type_impl_def(
        &mut self,
        impl_type0: &Type,
        def_type0: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        premises: &[Impl],
    ) -> Result<Impl, Error> {
        let mut impls = vec![];

        for impl_stmt in self.decls.type_impls.clone() {
            let Some(def_stmt) = impl_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            self.transaction(|this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);
                let impl_type1 = impl_stmt.head.as_type().unwrap();
                let def_type1 = def_stmt.type_of().apply(this);

                let satisfied = this.try_unify(impl_type0, impl_type1).is_ok()
                    && this.try_unify(def_type0, &def_type1).is_ok()
                    && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                    && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                if satisfied {
                    impls.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        self.unique_impl(impls)
    }

    fn solve_where_clauses(
        &mut self,
        where_clause: &[Impl],
        premises: &[Impl],
    ) -> Result<bool, Error> {
        self.depth += 1;
        let result = if self.depth > 10 {
            Err(RetryError::MaxDepth.into())
        } else {
            for i in where_clause {
                let impl_trait = i.as_trait().unwrap();
                match self.solve_trait_impl(impl_trait, premises) {
                    Err(e) => {
                        self.depth -= 1;
                        return Err(e);
                    }
                    Ok(_) => {}
                }
            }
            Ok(true)
        };
        self.depth -= 1;
        result
    }

    fn solve_trait_impl_def(
        &mut self,
        impl_trait0: &Trait,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        premises: &[Impl],
    ) -> Result<Impl, Error> {
        let mut impls = vec![];

        for i in premises {
            self.transaction(|this| {
                let impl_trait1 = i.as_trait().unwrap();
                let satisfied = this.traits_match(impl_trait0, impl_trait1);
                if satisfied {
                    impls.push(i.clone());
                }
                Ok(satisfied)
            })?;
        }

        let impl_stmts = self
            .decls
            .trait_impls
            .get(&impl_trait0.x)
            .cloned()
            .unwrap_or_default();

        for impl_stmt in impl_stmts {
            self.transaction(|this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let def_stmt = impl_stmt
                    .get_def(def_name)
                    .unwrap()
                    .instantiate(&def_type_args);
                let impl_trait1 = impl_stmt.head.as_trait().unwrap();

                let satisfied = this.traits_match(impl_trait0, impl_trait1)
                    && this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                    && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                if satisfied {
                    impls.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        self.unique_impl(impls)
    }

    // Solve a trait impl constraint. This occurs for example when calling a function with
    // where-clauses, e.g., def f[T](x:T) where Trait[T] = ...; f(1);
    fn solve_trait_impl(&mut self, impl_trait0: &Trait, premises: &[Impl]) -> Result<Impl, Error> {
        let mut impls = vec![];

        if impl_trait0.ts.len() == 1 {
            let t = impl_trait0.ts.first().unwrap();
            let t = t.apply(self);
            match t {
                Type::Record(xts) => {
                    if xts.iter().all(|(_, t)| {
                        let tr = Trait::new(impl_trait0.x, vec![t.clone()]);
                        self.solve_trait_impl(&tr, premises).is_ok()
                    }) {
                        return Ok(Impl::Trait(impl_trait0.clone()));
                    }
                }
                Type::Struct(x, ts) => {
                    let stmt = self.decls.structs.get(&x).unwrap().instantiate(&ts);
                    let xts = stmt.fields;
                    if xts.iter().all(|(_, t)| {
                        let tr = Trait::new(impl_trait0.x, vec![t.clone()]);
                        self.solve_trait_impl(&tr, premises).is_ok()
                    }) {
                        return Ok(Impl::Trait(impl_trait0.clone()));
                    }
                }
                _ => (),
            }
        }

        for i in premises {
            self.transaction(|this| {
                let impl_trait1 = i.as_trait().unwrap();
                let satisfied = this.traits_match(impl_trait0, impl_trait1);
                if satisfied {
                    impls.push(i.clone());
                }
                Ok(satisfied)
            })?;
        }

        let impl_stmts = self
            .decls
            .trait_impls
            .get(&impl_trait0.x)
            .cloned()
            .unwrap_or_default();

        for impl_stmt in impl_stmts {
            self.transaction(|this| {
                let ts = this.fresh_tvs(impl_stmt.generics.len());
                let impl_stmt = impl_stmt.instantiate(&ts).annotate(this);
                let impl_trait1 = impl_stmt.head.as_trait().unwrap();

                let satisfied = this.traits_match(impl_trait0, impl_trait1)
                    && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                if satisfied {
                    impls.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        self.unique_impl(impls)
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
    }

    fn unique_impl(&self, impls: Vec<Impl>) -> Result<Impl, Error> {
        match impls.len() {
            0 => Err(FatalError::ImplNotFound.into()),
            1 => Ok(impls.into_iter().next().unwrap()),
            _ => Err(RetryError::MultipleImplsFound(impls).into()),
        }
    }

    fn transaction(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<bool, Error>,
    ) -> Result<(), Error> {
        let type_snapshot = self.type_scope().type_table.snapshot();
        match f(self) {
            Ok(satisfied) => {
                if satisfied && !self.rollback {
                    self.type_scope().type_table.commit(type_snapshot);
                } else {
                    self.type_scope().type_table.rollback_to(type_snapshot);
                }
                Ok(())
            }
            Err(e) => {
                self.type_scope().type_table.rollback_to(type_snapshot);
                Err(e)
            }
        }
    }
}
