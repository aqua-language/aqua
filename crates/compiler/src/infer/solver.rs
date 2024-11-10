use crate::ast::Impl;
use crate::ast::Name;
use crate::ast::Trait;
use crate::ast::Type;
use crate::collections::set::Set;
use crate::span::Span;

use super::impl_var::ImplVarValue;
use super::Context;

#[derive(Debug)]
enum TraitSolverError {
    NoSolution,
    MultipleSolutions(Vec<Impl>),
    MaxDepth,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Constraint {
    // Generated when calling a function that has a where-clause.
    WhereClause(Span, Impl),
    // Generated when referring to an associated function.
    ExprAssoc(Span, Type, Impl, Name, Vec<Type>),
    // Generated when referring to an associated type.
    TypeAssoc(Span, Type, Impl, Name, Vec<Type>),
    // Generated when referring to a field.
    Field(Span, Type, Type, Name),
}

impl Context {
    pub fn solve_constraints(&mut self, span: Span) {
        let mut constraints = self.take_constraints();
        let premises = self.premises();
        let mut retries = vec![];
        let mut fuel = 0;
        loop {
            if fuel == 1 {
                // Retry by applying defaults.
                for constraint in constraints.iter() {
                    constraint.defaults(self);
                }
            } else if fuel > 100 {
                self.report.err(
                    span,
                    "Infinite loop detected",
                    "Unable to solve constraints",
                );
                return;
            }
            constraints = constraints
                .iter()
                .map(|c| c.apply(self).expand().canonicalize(self))
                .collect::<Set<_>>();
            let num_constraints = constraints.len();
            for constraint in constraints {
                match self.solve_constraint(&constraint, &premises) {
                    Ok(_) => {
                        self.commit(|this| {
                            this.solve_constraint(&constraint, &premises)
                                .expect("Constraint should be solvable");
                        });
                    }
                    Err(TraitSolverError::NoSolution) => {
                        self.report.err(
                            *constraint.span_of(),
                            "Unsatisfiable trait constraint",
                            format!("No solution found for constraint {constraint}"),
                        );
                    }
                    Err(e) => {
                        retries.push((constraint, e));
                    }
                }
            }
            if retries.is_empty() || num_constraints == retries.len() {
                for (constraint, e) in retries {
                    match e {
                        TraitSolverError::NoSolution => unreachable!(),
                        TraitSolverError::MultipleSolutions(candidates) => {
                            let msg = candidates
                                .iter()
                                .enumerate()
                                .map(|(i, c)| format!("{}: {}", i + 1, c))
                                .collect::<Vec<_>>()
                                .join("\n");
                            self.report.err(
                                *constraint.span_of(),
                                "Ambiguous trait implementation",
                                format!(
                                    "Found multiple solutions for constraint {constraint}:\n{msg}"
                                ),
                            );
                        }
                        TraitSolverError::MaxDepth => {
                            self.report.err(
                                *constraint.span_of(),
                                "Max depth reached",
                                "Unable to solve constraints",
                            );
                        }
                    }
                }
                break;
            } else {
                fuel += 1;
                constraints = retries.drain(..).map(|(c, _)| c).collect();
            }
        }
    }

    fn commit<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.commit = true;
        let v = f(self);
        self.commit = false;
        v
    }

    fn solve_constraint(
        &mut self,
        constraint: &Constraint,
        premises: &[Impl],
    ) -> Result<Impl, TraitSolverError> {
        match &constraint {
            Constraint::WhereClause(_, i) => {
                let tr = i.as_trait().unwrap();
                self.solve_trait_impl(tr, premises)
            }
            Constraint::ExprAssoc(_, def_type, imp, def_name, def_type_args) => match imp {
                Impl::Trait(impl_trait) => self.solve_trait_impl_def(
                    impl_trait,
                    def_type,
                    def_name,
                    def_type_args,
                    premises,
                ),
                Impl::Var(v) => {
                    if let ImplVarValue::Known(i) = self.get_impl_value(*v) {
                        Ok(i)
                    } else {
                        let i1 = self.solve_def(def_type, def_name, def_type_args, premises)?;
                        if self.commit {
                            self.union_impl_value(*v, i1.clone());
                        }
                        Ok(i1)
                    }
                }
                Impl::Type(impl_type) => {
                    self.solve_type_impl_def(impl_type, def_type, def_name, def_type_args, premises)
                }
                Impl::Err => Err(TraitSolverError::NoSolution),
                Impl::Unknown => unreachable!(),
                Impl::Path(..) => unreachable!(),
            },
            Constraint::TypeAssoc(_, _, _, _, _) => {
                todo!();
            }
            Constraint::Field(_s, _t0, _t1, _x) => {
                todo!()
            }
        }
    }

    // Sometimes the type arguments are unknown
    fn solve_def(
        &mut self,
        def_type: &Type,
        def_name: &Name,
        def_type_args: &[Type],
        premises: &[Impl],
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

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
                    solutions.push(i.clone());
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
                    solutions.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
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
                        solutions.push(impl_stmt.head);
                    }

                    Ok(satisfied)
                })?;
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
        premises: &[Impl],
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

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

                let satisfied = this.try_unify(impl_type0, impl_type1).is_ok()
                    && this.try_unify(def_type, &def_stmt.type_of()).is_ok()
                    && this.solve_where_clauses(&def_stmt.where_clause, premises)?
                    && this.solve_where_clauses(&impl_stmt.where_clause, premises)?;

                if satisfied {
                    solutions.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        self.check_solutions(solutions)
    }

    fn solve_where_clauses(
        &mut self,
        where_clause: &[Impl],
        premises: &[Impl],
    ) -> Result<bool, TraitSolverError> {
        self.depth += 1;
        let result = if self.depth > 10 {
            Err(TraitSolverError::MaxDepth)
        } else {
            Ok(where_clause.iter().all(|i| {
                let tr = i.as_trait().unwrap();
                self.solve_trait_impl(tr, premises).is_ok()
            }))
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
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

        for i in premises {
            self.transaction(|this| {
                let impl_trait1 = i.as_trait().unwrap();

                let satisfied = this.traits_match(impl_trait0, impl_trait1);

                if satisfied {
                    solutions.push(i.clone());
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
                    solutions.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
        }

        self.check_solutions(solutions)
    }

    // Solve a trait impl constraint. This occurs for example when calling a function with
    // where-clauses, e.g., def f[T](x:T) where Trait[T] = ...; f(1);
    fn solve_trait_impl(
        &mut self,
        impl_trait0: &Trait,
        premises: &[Impl],
    ) -> Result<Impl, TraitSolverError> {
        let mut solutions = vec![];

        for i in premises {
            self.transaction(|this| {
                let impl_trait1 = i.as_trait().unwrap();

                let satisfied = this.traits_match(impl_trait0, impl_trait1);

                if satisfied {
                    solutions.push(i.clone());
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
                    solutions.push(impl_stmt.head);
                }

                Ok(satisfied)
            })?;
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
    }

    fn check_solutions(&self, solutions: Vec<Impl>) -> Result<Impl, TraitSolverError> {
        match solutions.len() {
            0 => Err(TraitSolverError::NoSolution),
            1 => Ok(solutions.first().unwrap().clone()),
            _ => Err(TraitSolverError::MultipleSolutions(solutions)),
        }
    }

    fn transaction(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<bool, TraitSolverError>,
    ) -> Result<(), TraitSolverError> {
        let type_snapshot = self.type_scope().type_table.snapshot();
        match f(self) {
            Ok(satisfied) => {
                if satisfied && self.commit {
                    self.type_scope().type_table.commit(type_snapshot);
                } else {
                    self.type_scope().type_table.rollback_to(type_snapshot);
                }
                Ok(())
            }
            Err(e) => {
                self.type_scope().type_table.rollback_to(type_snapshot);
                return Err(e);
            }
        }
    }
}
