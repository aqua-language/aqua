pub mod mangle;

use std::collections::HashSet;
use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;

use crate::analysis::declare;
use crate::ast::passes::infer::type_var::TypeVarKind;
use crate::ast::passes::infer::type_var::TypeVarValue;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::report::Report;
use crate::syntax::span::Span;
use crate::traversal::mappable::Mappable;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitable::Visitable;
use mangle::Mangler;

use super::Pass;

/// Monomorphises polymorphic types into monomorphic types.
/// * Mangles named types.
/// * Only monomorphises types that are used.
#[derive(Debug, Default)]
pub struct Context {
    unique: HashSet<Name>,
    decls: declare::Context,
    stmts: Vec<Stmt>,
    type_table: InPlaceUnificationTable<TypeVar>,
    report: Report,
}

impl Pass for Context {
    fn run(&mut self, p: &Ast) -> Ast {
        p.visit(&mut self.decls);
        let stmts1 = p
            .stmts
            .iter()
            .filter_map(|stmt| self.expr_stmt(stmt))
            .collect::<Vec<_>>();
        let mut stmts0 = std::mem::take(&mut self.stmts);
        stmts0.extend(stmts1);
        Ast::new(p.span, stmts0)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn fresh_tv(&mut self, kind: TypeVarKind) -> Type {
        Type::Var(self.type_table.new_key(TypeVarValue::Unknown(kind)))
    }

    pub fn fresh_tvs(&mut self, n: usize) -> Vec<Type> {
        (0..n)
            .map(|_| self.fresh_tv(TypeVarKind::General))
            .collect()
    }

    fn expr_stmt(&mut self, s: &Stmt) -> Option<Stmt> {
        match s {
            Stmt::Local(s) => Some(Stmt::Local(Rc::new(self.map_stmt_local(s)))),
            Stmt::Expr(e) => Some(Stmt::Expr(Rc::new(self.map_expr(e)))),
            _ => None,
        }
    }

    fn monomorphise_stmt_def(&mut self, stmt: &StmtDef, ts: &[Type]) -> Name {
        let x = Mangler::mangle_fun(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        } else {
            self.unique.insert(x);
        }
        let stmt = stmt.instantiate(ts);
        let params = self.map_locals(&stmt.params);
        let t = stmt.ty.map(self);
        let e = stmt.effect.clone();
        let b = ExprBody::UserDefined(Rc::new(stmt.body.as_udf().unwrap().map(self)));
        let stmt = StmtDef::new(stmt.span, x, vec![], params, t, e, vec![], b);
        self.stmts.push(Stmt::Def(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_struct(&mut self, stmt: &StmtStruct, ts: &[Type]) -> Name {
        let x = Mangler::mangle_struct(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        } else {
            self.unique.insert(x);
        }
        let stmt = stmt.instantiate(ts);
        let span = stmt.span;
        let fields = stmt
            .fields
            .iter()
            .map(|(x, t)| (*x, self.map_type(t)))
            .collect();
        let stmt = StmtStruct::new(span, x, vec![], fields);
        self.stmts.push(Stmt::Struct(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_enum(&mut self, stmt: &StmtEnum, ts: &[Type]) -> Name {
        let x = Mangler::mangle_enum(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        } else {
            self.unique.insert(x);
        }
        let stmt = stmt.instantiate(ts);
        let variants = stmt.variants.mapv(|t| self.map_type(t));
        let stmt = StmtEnum::new(stmt.span, x, vec![], variants);
        self.stmts.push(Stmt::Enum(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_trait_impl_def(
        &mut self,
        stmt: &StmtImpl,
        impl_trait: &Trait,
        def_name0: &Name,
        def_type_args: &[Type],
    ) -> Name {
        let def_name1 = Mangler::mangle_trait_impl_def(impl_trait, def_name0, def_type_args);
        if self.unique.contains(&def_name1) {
            return def_name1;
        } else {
            self.unique.insert(def_name1);
        }
        let stmt_def = stmt.get_def(def_name0).unwrap();
        let stmt = stmt_def.instantiate(&def_type_args);
        let ps = self.map_locals(&stmt_def.params).into();
        let t = self.map_type(&stmt_def.ty);
        let e = stmt_def.effect.clone();
        let b = self.map_stmt_def_body(&stmt_def.body);
        let stmt = StmtDef::new(stmt.span, def_name1, vec![], ps, t, e, vec![], b);
        self.stmts.push(Stmt::Def(Rc::new(stmt)));
        def_name1
    }

    fn monomorphise_stmt_type_impl_def(
        &mut self,
        stmt: &StmtImpl,
        impl_type: &Type,
        def_name0: &Name,
        def_type_args: &[Type],
    ) -> Name {
        let def_name1 = Mangler::mangle_type_impl_def(impl_type, def_name0, def_type_args);
        if self.unique.contains(&def_name1) {
            return def_name1;
        } else {
            self.unique.insert(def_name1);
        }
        let stmt_def = stmt.get_def(def_name0).unwrap();
        let stmt = stmt_def.instantiate(&def_type_args);
        let ps = self.map_locals(&stmt_def.params).into();
        let t = self.map_type(&stmt_def.ty);
        let e = stmt_def.effect.clone();
        let b = self.map_stmt_def_body(&stmt_def.body);
        let stmt = StmtDef::new(stmt.span, def_name1, vec![], ps, t, e, vec![], b);
        self.stmts.push(Stmt::Def(Rc::new(stmt)));
        def_name1
    }

    fn solve_type_impl_def(
        &mut self,
        impl_type0: &Type,
        def_type0: &Type,
        def_name: &Name,
        def_type_args: &[Type],
    ) -> Option<StmtImpl> {
        for impl_stmt in self.decls.type_impls.clone() {
            let Some(def_stmt) = impl_stmt.get_def(def_name) else {
                continue;
            };
            if def_stmt.generics.len() != def_type_args.len() {
                continue;
            }
            let snapshot = self.type_table.snapshot();
            let ts = self.fresh_tvs(impl_stmt.generics.len());
            let impl_stmt = impl_stmt.instantiate(&ts);
            let def_stmt = impl_stmt
                .get_def(def_name)
                .unwrap()
                .instantiate(&def_type_args);
            let impl_type1 = self.map_type(impl_stmt.head.as_type().unwrap());
            let def_type1 = self.map_type(&def_stmt.ty());
            if self.try_unify(impl_type0, &impl_type1).is_ok()
                && self.solve_where_clauses(&impl_stmt.where_clause)
                && self.try_unify(def_type0, &def_type1).is_ok()
                && self.solve_where_clauses(&def_stmt.where_clause)
            {
                self.type_table.commit(snapshot);
                return Some(impl_stmt);
            } else {
                self.type_table.rollback_to(snapshot);
            }
        }
        None
    }

    fn solve_trait_impl(&mut self, tr0: &Trait) -> Option<StmtImpl> {
        if let Some(impls) = self.decls.trait_impls.get(&tr0.x).cloned() {
            for stmt in impls {
                let ts = self.fresh_tvs(stmt.generics.len());
                let stmt = stmt.instantiate(&ts);
                let snapshot = self.type_table.snapshot();
                let tr1 = stmt.head.as_trait().unwrap();
                if self.traits_match(tr0, tr1)
                    && stmt
                        .where_clause
                        .iter()
                        .all(|i| self.solve_trait_impl(i.as_trait().unwrap()).is_some())
                {
                    self.type_table.commit(snapshot);
                    return Some(stmt);
                } else {
                    self.type_table.rollback_to(snapshot);
                }
            }
        }
        None
    }

    fn solve_where_clauses(&mut self, where_clause: &[Impl]) -> bool {
        where_clause.iter().all(|i| {
            let tr = i.as_trait().unwrap();
            self.solve_trait_impl(tr).is_some()
        })
    }

    fn traits_match(&mut self, tr0: &Trait, tr1: &Trait) -> bool {
        tr0.x == tr1.x
            && tr0.ts.len() == tr1.ts.len()
            && tr1
                .ts
                .iter()
                .zip(tr0.ts.iter())
                .all(|(t0, t1)| self.try_unify(t0, t1).is_ok())
    }

    pub fn try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
        match (t0, t1) {
            (Type::Var(x0), t) | (t, Type::Var(x0)) => match self.type_table.probe_value(*x0) {
                TypeVarValue::Known(t0) => return self.try_unify(&t0, t),
                TypeVarValue::Unknown(k0) => match t {
                    Type::Var(x1) => match self.type_table.probe_value(*x1) {
                        TypeVarValue::Known(t1) => return self.try_unify(t0, &t1),
                        TypeVarValue::Unknown(k1) => {
                            if k0.merge(k1).is_some() {
                                self.type_table.union(*x0, *x1);
                                Ok(())
                            } else {
                                Err((t0.clone(), t1.clone()))
                            }
                        }
                    },
                    _ => {
                        self.type_table
                            .union_value(*x0, TypeVarValue::Known(t.clone()));
                        Ok(())
                    }
                },
            },
            (Type::Builtin(x0, ts0), Type::Builtin(x1, ts1))
                if x0 == x1 && ts0.len() == ts1.len() =>
            {
                ts0.iter()
                    .zip(ts1.iter())
                    .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
            }
            (Type::Tuple(ts0), Type::Tuple(ts1)) if ts0.len() == ts1.len() => ts0
                .iter()
                .zip(ts1.iter())
                .try_for_each(|(t0, t1)| self.try_unify(t0, t1)),
            (Type::Function(ts0, t0, _e0), Type::Function(ts1, t1, _e1))
                if ts0.len() == ts1.len() =>
            {
                ts0.iter()
                    .chain([t0.as_ref()])
                    .zip(ts1.iter().chain([t1.as_ref()]))
                    .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
            }
            (Type::Record(xts0), Type::Record(xts1)) if xts0.len() == xts1.len() => {
                let xts0 = xts0.sort_keys();
                let xts1 = xts1.sort_keys();
                if xts0.same_keys_sorted(&xts1) {
                    xts0.values()
                        .zip(xts1.values())
                        .try_for_each(|(t0, t1)| self.try_unify(t0, t1))
                } else {
                    Err((t0.clone(), t1.clone()))
                }
            }
            (Type::Assoc(_b, _x, _), _t0) | (_t0, Type::Assoc(_b, _x, _)) => {
                todo!()
            }
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            (Type::Generic(..), Type::Generic(..)) => unreachable!(),
            (Type::Err, _) | (_, Type::Err) => unreachable!(),
            (Type::Unknown, _) | (_, Type::Unknown) => unreachable!(),
            _ => Err((t0.clone(), t1.clone())),
        }
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        let t = self.map_type(e.ty());
        let s = e.span();
        match e {
            Expr::Struct(_, _, x, ts, xes) => {
                let stmt = self.decls.structs.get(x).unwrap().clone();
                let x = self.monomorphise_stmt_struct(&stmt, ts);
                let xes = self.map_expr_fields(xes).into();
                Expr::Struct(s, t, x, vec![], xes)
            }
            Expr::Enum(_, _, x, ts, x1, e) => {
                let stmt = self.decls.enums.get(x).unwrap().clone();
                let x = self.monomorphise_stmt_enum(&stmt, ts);
                let e = self.map_expr(e);
                Expr::Enum(s, t, x, vec![], *x1, Rc::new(e))
            }
            Expr::Assoc(_, _, i, x1, ts1) => {
                let ts1 = self.map_types(ts1);
                match i {
                    Impl::Trait(tr) => {
                        let stmt = self.solve_trait_impl(tr).unwrap();
                        let x = self.monomorphise_stmt_trait_impl_def(&stmt, tr, x1, &ts1);
                        Expr::Def(s, t, x, vec![])
                    }
                    Impl::Type(t1) => {
                        let t1 = self.map_type(t1);
                        let stmt = self.solve_type_impl_def(&t1, &t, x1, &ts1).unwrap();
                        let x = self.monomorphise_stmt_type_impl_def(&stmt, &t1, x1, &ts1);
                        Expr::Def(s, t, x, vec![])
                    }
                    Impl::Var(..) => unreachable!(),
                    Impl::Path(..) => unreachable!(),
                    Impl::Unknown => unreachable!(),
                    Impl::Err => unreachable!(),
                }
            }
            Expr::Def(_, _, x, ts) => {
                let ts = self.map_types(ts);
                let stmt = self.decls.defs.get(x).unwrap();
                match &stmt.body {
                    ExprBody::UserDefined(_) => {
                        let x = self.monomorphise_stmt_def(&stmt.clone(), &ts);
                        Expr::Def(s, t, x, vec![])
                    }
                    ExprBody::Builtin(_) => Expr::Def(s, t, *x, ts),
                }
            }
            Expr::Record(_, _, xes) => {
                let xes = self.map_expr_fields(xes).into();
                let Type::Builtin(x, _) = t else {
                    unreachable!()
                };
                Expr::Struct(s, t.clone(), x, vec![], xes)
            }
            _ => self._map_expr(e),
        }
    }

    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Builtin(x, ts) => {
                if let Some(_) = self.decls.types.get(x).cloned() {
                    let ts = self.map_types(ts);
                    Type::Builtin(*x, ts)
                } else if let Some(stmt) = self.decls.structs.get(x).cloned() {
                    let x = self.monomorphise_stmt_struct(stmt.as_ref(), ts);
                    Type::Builtin(x, vec![])
                } else if let Some(stmt) = self.decls.enums.get(x).cloned() {
                    let x = self.monomorphise_stmt_enum(stmt.as_ref(), ts);
                    Type::Builtin(x, vec![])
                } else {
                    // TODO: This case should be unreachable, but currently fires for records.
                    Type::Builtin(*x, self.map_types(ts))
                }
            }
            Type::Record(xts) => {
                let xts = xts.mapv(|t| self.map_type(t));
                let s = Span::default();
                let x = Mangler::mangle_record(&xts);
                let stmt = StmtStruct::new(s, x, vec![], xts);
                let x = self.monomorphise_stmt_struct(&stmt, &[]);
                Type::Builtin(x, vec![])
            }
            Type::Var(x) => match self.type_table.probe_value(*x) {
                TypeVarValue::Known(t) => self.map_type(&t),
                TypeVarValue::Unknown(_) => t.clone(),
            },
            _ => self._map_type(t),
        }
    }
}
