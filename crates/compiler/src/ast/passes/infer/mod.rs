pub mod annotate;
pub mod apply;
pub mod canonicalize;
pub mod defaults;
pub mod expand;
pub mod gather_goals;
pub mod impl_var;
pub mod instantiate;
pub mod local;
pub mod primitives;
pub mod solver;
pub mod type_var;
pub mod unify;

use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;
use impl_var::ImplVarValue;
use solver::Constraint;

use crate::analysis::declare;
use crate::ast::Ast;
use crate::ast::Effect;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::ImplVar;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtImpl;
use crate::ast::StmtLocal;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::collections::map::Map;
use crate::collections::set::Set;
use crate::report::Diagnostic;
use crate::report::Report;
use crate::syntax::span::Span;
use crate::traversal::mappable::Mappable;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitable::Visitable;
use crate::traversal::visitor::Visitor;
use primitives::bool;
use primitives::char;
use primitives::string;

use self::type_var::TypeVarKind;
use self::type_var::TypeVarValue;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Vec<Local>>,
    contexts: Vec<TypeContext>,
    pub report: Report,
    pub decls: declare::Context,
    pub depth: usize,
    pub rollback: bool,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
        program.visit(&mut self.decls);
        program.map(self)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TypeContext {
    pub type_union_find: InPlaceUnificationTable<TypeVar>,
    pub impl_union_find: InPlaceUnificationTable<ImplVar>,
    constraints: Set<Constraint>,
    pub where_clause: Vec<Impl>,
}

impl TypeContext {
    pub fn new(where_clause: Vec<Impl>) -> TypeContext {
        TypeContext {
            type_union_find: InPlaceUnificationTable::new(),
            impl_union_find: InPlaceUnificationTable::new(),
            constraints: vec![].into(),
            where_clause,
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack: vec![Vec::new()],
            contexts: vec![TypeContext::new(vec![])],
            report: Report::new(),
            decls: declare::Context::default(),
            depth: 0,
            rollback: false,
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.type_ctx().constraints.add(constraint);
    }

    pub fn add_constraints(&mut self, constraints: Vec<Constraint>) {
        for constraint in constraints {
            self.add_constraint(constraint);
        }
    }

    pub fn take_constraints(&mut self) -> Set<Constraint> {
        std::mem::take(&mut self.type_ctx().constraints)
    }

    pub fn premises(&self) -> Vec<Impl> {
        self.contexts
            .iter()
            .flat_map(|s| s.where_clause.clone())
            .collect()
    }

    pub fn type_ctx(&mut self) -> &mut TypeContext {
        self.contexts.last_mut().unwrap()
    }

    pub fn get_type_value(&mut self, a: TypeVar) -> TypeVarValue {
        self.type_ctx().type_union_find.probe_value(a)
    }

    pub fn get_impl_value(&mut self, a: ImplVar) -> ImplVarValue {
        self.type_ctx().impl_union_find.probe_value(a)
    }

    pub fn union_impl_value(&mut self, a: ImplVar, b: Impl) {
        self.type_ctx()
            .impl_union_find
            .union_value(a, ImplVarValue::Known(b));
    }

    pub fn union_type_value(&mut self, a: TypeVar, b: Type) {
        self.type_ctx()
            .type_union_find
            .union_value(a, TypeVarValue::Known(b));
    }

    pub fn union(&mut self, a: TypeVar, b: TypeVar) {
        self.type_ctx().type_union_find.union(a, b);
    }

    pub fn fresh_tv(&mut self, kind: TypeVarKind) -> Type {
        Type::Var(
            self.type_ctx()
                .type_union_find
                .new_key(TypeVarValue::Unknown(kind)),
        )
    }

    pub fn fresh_tvs(&mut self, n: usize) -> Vec<Type> {
        (0..n)
            .map(|_| self.fresh_tv(TypeVarKind::General))
            .collect()
    }

    pub fn fresh_iv(&mut self) -> Impl {
        Impl::Var(
            self.type_ctx()
                .impl_union_find
                .new_key(ImplVarValue::Unknown),
        )
    }

    pub fn unify(&mut self, s0: Span, s1: Span, t0: &Type, t1: &Type) {
        t0.gather_constraints(self);
        t1.gather_constraints(self);
        let t0 = &t0.expand();
        let t1 = &t1.expand();
        let snapshot = self.type_ctx().type_union_find.snapshot();
        if self.try_unify(&t0, &t1).is_ok() {
            self.type_ctx().type_union_find.commit(snapshot)
        } else {
            self.type_ctx().type_union_find.rollback_to(snapshot);
            let t0 = t0.apply(self);
            let t1 = t1.apply(self);
            self.report.add(Diagnostic::err2(
                s0,
                s1,
                "Type mismatch",
                format!("Expected {t0}"),
                format!("Found {t1}"),
            ));
        }
    }

    pub fn try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
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
                        if k0.merge(k1).is_some() =>
                    {
                        if x0 != x1 {
                            self.union(*x0, *x1);
                        }
                        Ok(())
                    }
                    _ => Err((t0.clone(), t1.clone())),
                }
            }
            (Type::Var(x1), t3) | (t3, Type::Var(x1)) => match self.get_type_value(*x1) {
                TypeVarValue::Known(t4) => self.try_unify(t3, &t4),
                TypeVarValue::Unknown(k1) if k1.is_unifiable_with(t3) => {
                    self.union_type_value(*x1, t3.clone());
                    Ok(())
                }
                _ => Err((t0.clone(), t1.clone())),
            },
            (Type::Builtin(x0, ts0), Type::Builtin(x1, ts1))
            | (Type::Struct(x0, ts0), Type::Struct(x1, ts1))
            | (Type::Enum(x0, ts0), Type::Enum(x1, ts1))
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
                    _ => Err((t0.clone(), t1.clone())),
                }
            }
            (Type::Assoc(_b, _x, _), _t0) | (_t0, Type::Assoc(_b, _x, _)) => {
                todo!()
                // if let Some(t1) = b.as_trait().unwrap().xts.get(x) {
                //     self.try_unify(t0, t1)
                // } else {
                //     Err(())
                // }
            }
            (Type::Unit, Type::Unit) => Ok(()),
            (Type::Err, _) | (_, Type::Err) => Ok(()),
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            (Type::Unknown, Type::Unknown) => Ok(()),
            _ => Err((t0.clone(), t1.clone())),
        }
    }

    pub fn get(&self, x1: &Name) -> &Local {
        self.stack
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|l0| l0.name == *x1))
            .unwrap_or_else(|| panic!("Unknown variable {x1}"))
    }

    pub fn bind(&mut self, l: Local) {
        self.stack.last_mut().unwrap().push(l)
    }

    #[allow(unused)]
    fn debug(&mut self, label: &str) {
        println!("Debug ({label})");
        println!("* Substitutions:");
        for i in 0..self.type_ctx().type_union_find.len() as u32 {
            let x = TypeVar(i);
            let xr = self.type_ctx().type_union_find.find(x);
            let t = self.type_ctx().type_union_find.probe_value(x);
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
        for constraint in self.type_ctx().constraints.iter() {
            println!("    {}", constraint);
        }
        println!("* premises:");
        for assumption in self.contexts.iter().rev().flat_map(|s| &s.where_clause) {
            println!("    {}", assumption.verbose());
        }
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.push(Vec::new());
    }

    fn exit_scope(&mut self) {
        self.stack.pop();
    }

    fn map_program(&mut self, program: &Ast) -> Ast {
        let program = program.annotate(self);
        let stmts = self.map_stmts(&program.stmts);
        let program = Ast::new(program.span, stmts);
        let p = program.apply(self);
        self.solve_constraints(p.span);
        p.defaults(self);
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
        self.contexts.push(TypeContext::new(s.where_clause.clone()));
        let generics = s.generics.clone();
        let head = s.head.clone();
        let defs = s
            .defs
            .iter()
            .map(|d| Rc::new(self.map_stmt_def(d)))
            .collect::<Vec<_>>();
        let types = s.types.clone();
        let where_clause = s.where_clause.clone();
        self.contexts.pop();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.contexts.push(TypeContext::new(s.where_clause.clone()));
                for l in &s.params {
                    self.bind(l.clone());
                }
                let e = e.annotate(self);
                self.visit_expr(&e);
                self.unify(s.span, e.span(), &s.ty, e.ty());
                self.solve_constraints(s.span);
                let stmt = StmtDef::new(
                    s.span,
                    s.name,
                    s.generics.clone(),
                    s.params.clone(),
                    s.ty.clone(),
                    s.effect.clone(),
                    s.where_clause.clone(),
                    ExprBody::UserDefined(Rc::new(e)),
                );
                stmt.defaults(self);
                let s = stmt.apply(self);
                self.contexts.pop();
                s
            }
            ExprBody::Builtin(b) => StmtDef::new(
                s.span,
                s.name,
                s.generics.clone(),
                s.params.clone(),
                s.ty.clone(),
                s.effect.clone(),
                s.where_clause.clone(),
                ExprBody::Builtin(b.clone()),
            ),
        }
    }

    fn map_stmt_local(&mut self, s: &StmtLocal) -> StmtLocal {
        if let Some(e) = &s.expr {
            self.visit_expr(e);
            self.unify(s.span, e.span(), &s.local.ty, e.ty());
        }
        self.bind(s.local.clone());
        StmtLocal::new(s.span, s.local.clone(), s.expr.clone())
    }
}

impl Visitor for Context {
    fn visit_expr(&mut self, e: &Expr) {
        match e {
            Expr::Int(..) => {}
            Expr::Float(..) => {}
            Expr::Bool(s, t0, _) => self.unify(*s, *s, t0, &bool()),
            Expr::Char(s, t0, _) => self.unify(*s, *s, t0, &char()),
            Expr::String(s, t0, _) => self.unify(*s, *s, t0, &string()),
            Expr::Unit(s, t) => self.unify(*s, *s, t, &Type::Unit),
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
                        .find_map(|(y, t)| (x == *y).then_some(t))
                        .unwrap()
                        .instantiate(&gsub);
                    self.visit_expr(&e);
                    self.unify(e.span(), stmt.span, e.ty(), &t2);
                }
                let t1 = Type::Struct(*x, ts.clone());
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
                self.unify(e.span(), stmt.span, e.ty(), &t2);
                let t1 = Type::Enum(*x, ts.clone());
                self.unify(*s, stmt.span, t0, &t1);
            }
            Expr::Tuple(s, t0, es) => {
                self.visit_exprs(es);
                let ts = es.iter().map(|e| e.ty().clone()).collect::<Vec<_>>();
                let t1 = Type::Tuple(ts);
                self.unify(*s, *s, t0, &t1);
            }
            Expr::Def(s, t0, x, ts0) => {
                let stmt = self.decls.defs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts0.clone())
                    .collect::<Map<_, _>>();
                let constraints = stmt
                    .where_clause
                    .iter()
                    .map(|p| Constraint::WhereClause(*s, p.instantiate(&gsub)))
                    .collect::<Vec<_>>();
                self.add_constraints(constraints);
                let t2 = Type::Function(
                    stmt.params.iter().map(|l| l.ty.clone()).collect(),
                    Rc::new(stmt.ty.clone()),
                    stmt.effect.clone(),
                )
                .instantiate(&gsub);
                self.unify(*s, stmt.span, t0, &t2);
            }
            Expr::Call(s, t0, e1, es) => {
                let ts = es.iter().map(|e| e.ty().clone()).collect::<Vec<_>>();
                let t2 = Type::Function(ts, Rc::new(t0.clone()), Effect::Unknown);
                self.unify(*s, e1.span(), e1.ty(), &t2);
                self.visit_expr(e1);
                self.visit_exprs(es);
            }
            Expr::Block(s, t0, b) => {
                self.visit_block(b);
                self.unify(*s, e.span(), t0, b.ty());
            }
            Expr::Array(s, t0, es) => {
                self.visit_exprs(es);
                let t1 = es
                    .first()
                    .map(|e| e.ty().clone())
                    .unwrap_or_else(|| self.fresh_tv(TypeVarKind::General));
                for e in es.iter() {
                    self.unify(*s, e.span(), e.ty(), &t1)
                }
                let t2 = Type::Builtin(Name::from("Array"), vec![t1]);
                self.unify(*s, *s, t0, &t2);
            }
            Expr::Assign(s, t0, e0, e1) => {
                self.visit_expr(e0);
                self.visit_expr(e1);
                self.unify(*s, e0.span(), e0.ty(), e1.ty());
                self.unify(*s, *s, t0, &Type::Unit);
            }
            Expr::Return(_, _, _) => todo!(),
            Expr::Continue(_, _, _) => todo!(),
            Expr::Break(_, _, _) => todo!(),
            Expr::Lambda(s, t0, ls, t1, e0) => {
                self.enter_scope();
                for l in ls {
                    self.bind(l.clone());
                }
                let ts0 = ls.iter().map(|l| l.ty.clone()).collect::<Vec<_>>();
                let t2 = Type::Function(ts0, Rc::new(t1.clone()), Effect::Unknown);
                self.unify(*s, *s, t0, &t2);
                self.visit_expr(e0);
                self.unify(*s, e0.span(), &t1, e0.ty());
                self.exit_scope();
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(s, t0, _l, e, b) => {
                self.visit_expr(e);
                self.visit_block(b);
                self.unify(*s, e.span(), e.ty(), &bool());
                self.unify(*s, *s, t0, &Type::Unit);
                self.unify(*s, b.span, t0, b.ty());
            }
            Expr::Record(s, t0, xes) => {
                xes.iter().for_each(|(_, e)| self.visit_expr(e));
                let xts = xes
                    .iter()
                    .map(|(x, e)| (*x, e.ty().clone()))
                    .collect::<Map<_, _>>();
                let t1 = Type::Record(xts);
                self.unify(*s, *s, t0, &t1);
            }
            Expr::For(s, t0, _l, _, e, b) => {
                self.visit_expr(e);
                self.visit_block(b);
                self.unify(*s, *s, t0, &Type::Unit);
                self.unify(*s, *s, t0, &b.ty());
            }
            Expr::Loop(_, _, _, _) => todo!(),
            Expr::Assoc(s, t, i, x1, ts1) => {
                self.type_ctx().constraints.add(Constraint::AssocDef(
                    *s,
                    t.clone(),
                    i.clone(),
                    *x1,
                    ts1.clone(),
                ));
            }
            Expr::IfElse(s, t, e, b0, b1) => {
                self.visit_expr(e);
                self.visit_block(b0);
                self.visit_block(b1);
                self.unify(*s, *s, e.ty(), &bool());
                self.unify(*s, b0.span, t, b0.ty());
                self.unify(*s, b1.span, t, b1.ty());
            }
            Expr::Ref(s, t, e1, m) => {
                let Expr::Place(s1, t1, p1) = e1.as_ref() else {
                    unreachable!()
                };
                let loan = Loan::new(p1.clone(), *m);
                let t2 = Type::Ref(vec![loan], Rc::new(t1.clone()), *m);
                self.unify(*s, *s1, t, &t2);
            }
            Expr::Place(s, t0, p) => {
                let mut ty = self.get(&p.local.name).ty.clone();
                self.unify(p.local.span, p.local.span, &p.local.ty, &ty);
                for elem in &p.elems {
                    self.add_constraint(Constraint::PlaceElem(*s, ty.clone(), elem.clone()));
                    ty = elem.ty().clone();
                }
                self.unify(*s, p.span, t0, &ty);
            }
            Expr::Err(s, t) => {
                self.unify(*s, *s, t, &Type::Never);
            }
            // TODO: Desugar all in previous pass.
            Expr::Path(..) => unreachable!(),
            Expr::Field(..) => unreachable!(),
            Expr::Index(..) => unreachable!(),
            Expr::Local(..) => unreachable!(),
            Expr::Query(..) => unreachable!(),
            Expr::QueryInto(..) => unreachable!(),
            Expr::InfixBinaryOp(..) => unreachable!(),
            Expr::PrefixUnaryOp(..) => unreachable!(),
            Expr::PostfixUnaryOp(..) => unreachable!(),
            Expr::Annotate(..) => unreachable!(),
            Expr::Paren(..) => unreachable!(),
            Expr::Dot(..) => unreachable!(),
            Expr::IntSuffix(..) => unreachable!(),
            Expr::FloatSuffix(..) => unreachable!(),
            Expr::Anonymous(..) => unreachable!(),
            Expr::Closure(_, _, _, _xts0, _xts1, _t, _e) => {
                todo!()
            }
            Expr::Deref(..) => unreachable!(),
        }
    }
}
