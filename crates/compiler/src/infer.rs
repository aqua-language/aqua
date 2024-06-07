pub mod type_var;
pub mod unify;

use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;

use crate::apply::Annotate;
use crate::apply::Apply;
use crate::apply::Canonicalize;
use crate::apply::Defaults;
use crate::apply::Expand;
use crate::apply::GatherGoals;
use crate::apply::Instantiate;
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
use crate::ast::StmtTraitDef;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::collections::map::Map;
use crate::diag::Report;
use crate::lexer::Span;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

use self::type_var::TypeVarKind;
use self::type_var::TypeVarValue;

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Context {
    stack: Vec<ValueScope>,
    type_stack: Vec<TypeScope>,
    pub type_impls: Vec<StmtImpl>,
    pub trait_impls: HashMap<Name, Vec<Rc<StmtImpl>>>,
    defs: HashMap<Name, Rc<StmtDef>>,
    structs: HashMap<Name, Rc<StmtStruct>>,
    enums: HashMap<Name, Rc<StmtEnum>>,
    traits: HashMap<Name, Rc<StmtTrait>>,
    types: HashMap<Name, Rc<StmtType>>,
    pub report: Report,
    pub std: StandardTypes,
}

#[derive(Debug)]
pub struct StandardTypes {
    pub ints: Vec<Name>,
    pub int_default: Type,
    pub floats: Vec<Name>,
    pub float_default: Type,
    pub string: Type,
    pub char: Type,
    pub bool: Type,
}

#[derive(Debug)]
enum TraitSolverError {
    Unsolved,
    Ambiguous(Vec<Candidate>),
}

impl StandardTypes {
    pub fn new() -> StandardTypes {
        StandardTypes {
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
}

#[derive(Default, Debug)]
pub struct ValueScope {
    binds: Map<Name, Binding>,
}

#[derive(Debug)]
pub struct TypeScope {
    pub table: InPlaceUnificationTable<TypeVar>,
    pub goals: Vec<Bound>,
    pub where_clause: Vec<Bound>,
}

impl TypeScope {
    pub fn new() -> TypeScope {
        TypeScope {
            table: InPlaceUnificationTable::new(),
            goals: vec![],
            where_clause: vec![],
        }
    }
}

impl ValueScope {
    pub fn new() -> ValueScope {
        ValueScope::default()
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
            stack: vec![ValueScope::new()],
            type_stack: vec![],
            report: Report::new(),
            trait_impls: HashMap::new(),
            type_impls: vec![],
            defs: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            types: HashMap::new(),
            std: StandardTypes::new(),
        }
    }

    pub fn infer(&mut self, p: &Program) -> Program {
        self.map_program(p)
    }

    pub fn apply(&mut self, t: &Type) -> Type {
        Apply::new(self).map_type(t)
    }

    pub fn type_scope(&mut self) -> &mut TypeScope {
        self.type_stack.last_mut().unwrap()
    }

    pub fn get_type(&mut self, a: TypeVar) -> TypeVarValue {
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
        let snapshot = self.type_scope().table.snapshot();
        match self.try_unify(&t0, &t1) {
            Ok(_) => self.type_scope().table.commit(snapshot),
            Err((t0, t1)) => {
                self.type_scope().table.rollback_to(snapshot);
                self.report.err2(
                    s0,
                    s1,
                    "Type mismatch",
                    format!("Expected {t0}"),
                    format!("Found {t1}"),
                );
            }
        }
    }

    pub fn try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
        let t0 = Apply::new(self).map_type(t0);
        let t1 = Apply::new(self).map_type(t1);
        self._try_unify(&t0, &t1)
    }

    // This function assumes there are no "Known" type variables in either t0 or t1. Any known
    // type variables should be applied before calling this function.
    // Problem: Unification of type variables of different types does not work well
    pub fn _try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
        match (t0, t1) {
            (Type::Var(x0), t) | (t, Type::Var(x0)) => {
                let k0 = self.get_type(*x0).unknown().unwrap();
                match t {
                    Type::Var(x1) => {
                        let k1 = self.get_type(*x1).unknown().unwrap();
                        if k0.is_mergeable(k1) {
                            self.union(*x0, *x1);
                            Ok(())
                        } else {
                            Err((t0.clone(), t1.clone()))
                        }
                    }
                    Type::Cons(x1, _)
                        if k0.is_int() && self.std.ints.contains(x1)
                            || k0.is_float() && self.std.floats.contains(x1) =>
                    {
                        self.union_value(*x0, t.clone());
                        Ok(())
                    }
                    t if k0.is_general() => {
                        self.union_value(*x0, t.clone());
                        Ok(())
                    }
                    _ => Err((t0.clone(), t1.clone())),
                }
            }
            (Type::Cons(x0, ts0), Type::Cons(x1, ts1)) if x0 == x1 && ts0.len() == ts1.len() => ts0
                .iter()
                .zip(ts1.iter())
                .try_for_each(|(t0, t1)| self._try_unify(t0, t1)),
            (Type::Tuple(ts0), Type::Tuple(ts1)) if ts0.len() == ts1.len() => ts0
                .iter()
                .zip(ts1.iter())
                .try_for_each(|(t0, t1)| self._try_unify(t0, t1)),
            (Type::Fun(ts0, t0), Type::Fun(ts1, t1)) if ts0.len() == ts1.len() => ts0
                .iter()
                .chain([t0.as_ref()])
                .zip(ts1.iter().chain([t1.as_ref()]))
                .try_for_each(|(t0, t1)| self._try_unify(t0, t1)),
            (Type::Record(xts0), Type::Record(xts1)) if xts0.len() == xts1.len() => {
                let xts0 = xts0.sort_keys();
                let xts1 = xts1.sort_keys();
                if xts0.same_keys_sorted(&xts1) {
                    xts0.values()
                        .zip(xts1.values())
                        .try_for_each(|(t0, t1)| self._try_unify(t0, t1))
                } else {
                    Err((t0.clone(), t1.clone()))
                }
            }
            (Type::Generic(x0), Type::Generic(x1)) if x0 == x1 => Ok(()),
            (Type::Assoc(b0, x0, ts0), Type::Assoc(b1, x1, ts1))
                if x0 == x1 && ts0.len() == ts1.len() =>
            {
                match (b0, b1) {
                    (Bound::Err(_), _) | (_, Bound::Err(_)) => Ok(()),
                    (Bound::Path(..), _) | (_, Bound::Path(..)) => unreachable!(),
                    (Bound::Type(_, t0), Bound::Type(_, t1)) => ts0
                        .iter()
                        .chain([t0.as_ref()])
                        .zip(ts1.iter().chain([t1.as_ref()]))
                        .try_for_each(|(t0, t1)| self._try_unify(t0, t1)),
                    (Bound::Trait(_, x0, ts0, _), Bound::Trait(_, x1, ts1, _))
                        if x0 == x1 && ts0.len() == ts1.len() =>
                    {
                        ts0.iter()
                            .zip(ts1.iter())
                            .try_for_each(|(t0, t1)| self._try_unify(t0, t1))
                    }
                    _ => Err((t0.clone(), t1.clone())),
                }
            }
            (Type::Assoc(b, x, _), t0) | (t0, Type::Assoc(b, x, _)) => {
                if let Some(t1) = b.get_type(x) {
                    self._try_unify(t0, t1)
                } else {
                    Err((t0.clone(), t1.clone()))
                }
            }
            (Type::Err, _) | (_, Type::Err) => Ok(()),
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            (Type::Unknown, Type::Unknown) => Ok(()),
            _ => Err((t0.clone(), t1.clone())),
        }
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

    fn solve_goals(&mut self) {
        let goals = std::mem::take(&mut self.type_scope().goals);
        let assumptions = self
            .type_stack
            .iter()
            .rev()
            .flat_map(|s| s.where_clause.clone())
            .collect::<Vec<_>>();
        let goals = Apply::new(self).map_bounds(&goals);
        let goals = Canonicalize::new(self).map_bounds(&goals);
        let goals = goals.iter().cloned().collect::<HashSet<_>>();
        for goal in goals {
            let goal = Apply::new(self).map_bound(&goal);
            match self.solve::<false>(&goal, &assumptions) {
                Ok(_) => {
                    self.solve::<true>(&goal, &assumptions)
                        .expect("Goals should be solved");
                }
                Err(TraitSolverError::Unsolved) => {
                    self.report.err(
                        goal.span(),
                        "Trait is not implemented",
                        format!("Found no implementation for trait {}", goal.verbose()),
                    );
                }
                Err(TraitSolverError::Ambiguous(cs)) => {
                    let s = cs
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.report.err(
                        goal.span(),
                        "Ambiguous trait implementation",
                        format!("Found multiple implementations for trait:\n{s}"),
                    );
                }
            }
        }
    }

    fn solve<const KEEP: bool>(
        &mut self,
        goal: &Bound,
        assumptions: &[Bound],
    ) -> Result<Candidate, TraitSolverError> {
        let Bound::Trait(_, x, _, _) = goal else {
            todo!();
        };
        let mut solutions = vec![];
        for b in assumptions {
            if self.matches::<{ KEEP }>(goal, b) {
                solutions.push(Candidate::Bound(b.clone()));
            }
        }
        if let Some(impls) = self.trait_impls.get(x).cloned() {
            for stmt in impls {
                let gsub = stmt
                    .generics
                    .iter()
                    .map(|x| (*x, self.fresh(TypeVarKind::General)))
                    .collect::<Map<_, _>>();
                let stmt = Instantiate::new(&gsub).map_stmt_impl(stmt.as_ref());
                let stmt = Annotate::new(self).map_stmt_impl(&stmt);
                if self.matches::<{ KEEP }>(goal, &stmt.head)
                    && stmt
                        .where_clause
                        .iter()
                        .all(|subgoal| self.solve::<{ KEEP }>(subgoal, assumptions).is_ok())
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
    fn matches<const KEEP: bool>(&mut self, b0: &Bound, b1: &Bound) -> bool {
        let snapshot = self.type_scope().table.snapshot();
        let b = match (b0, b1) {
            (Bound::Trait(_, x0, ts0, xts0), Bound::Trait(_, x1, ts1, xts1)) => {
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
            (Bound::Type(_, t0), Bound::Type(_, t1)) => self.try_unify(t0, t1).is_ok(),
            (Bound::Path(..), _) | (_, Bound::Path(..)) => unreachable!(),
            (Bound::Err(..), _) | (_, Bound::Err(..)) => true,
            (Bound::Trait(..), Bound::Type(..)) | (Bound::Type(..), Bound::Trait(..)) => false,
        };
        if KEEP {
            self.type_scope().table.commit(snapshot);
        } else {
            self.type_scope().table.rollback_to(snapshot);
        }
        b
    }

    #[allow(unused)]
    fn debug(&mut self, s: &str) {
        println!("Debug ({s})");
        println!("* Substitutions: {{");
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
        println!("  }}");
        println!("* Goals: {{");
        for goal in &self.type_scope().goals {
            println!("    {}", goal.verbose());
        }
        println!("  }}");
        println!("* Assumptions: {{");
        for assumption in self.type_stack.iter().rev().flat_map(|s| &s.where_clause) {
            println!("    {}", assumption.verbose());
        }
        println!("  }}");
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
impl Visitor for Context {
    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.defs.insert(s.name, s.clone());
            }
            Stmt::Impl(s) => match s.head {
                Bound::Trait(_, x, _, _) => {
                    self.trait_impls
                        .entry(x)
                        .or_insert_with(Vec::new)
                        .push(s.clone());
                }
                Bound::Type(_, _) => {}
                Bound::Path(_, _) => {}
                Bound::Err(_) => {}
            },
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
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.push(ValueScope::new());
    }

    fn exit_scope(&mut self) {
        self.stack.pop();
    }

    fn map_program(&mut self, p: &Program) -> Program {
        self.type_stack.push(TypeScope::new());
        let program = Annotate::new(self).map_program(p);
        self.visit_program(&program);
        let stmts = self.map_stmts(&program.stmts);
        let program = Program::new(stmts);
        let program = Apply::new(self).map_program(&program);
        GatherGoals::new(self).visit_program(&program);
        self.solve_goals();
        Defaults::new(self).visit_program(&program);
        let program = Expand::new().map_program(&program);
        let program = Apply::new(self).map_program(&program);
        self.type_stack.pop();
        program
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Struct(s) => Stmt::Struct(s.clone()),
            Stmt::Enum(s) => Stmt::Enum(s.clone()),
            Stmt::Type(s) => Stmt::Type(s.clone()),
            _ => self._map_stmt(s),
        }
    }

    fn map_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let e = self.map_expr(&s.expr);
        self.unify(s.span, e.span_of(), &s.ty, e.type_of());
        self.bind(s.name, Binding::Var(s.span, s.ty.clone()));
        StmtVar::new(e.span_of(), s.name, s.ty.clone(), e)
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        // self.type_stack.push(TypeScope {
        //     table: InPlaceUnificationTable::new(),
        //     goals: vec![],
        // });
        let generics = s.generics.clone();
        let head = s.head.clone();
        let defs = s
            .defs
            .iter()
            .map(|d| Rc::new(self.map_stmt_def(d)))
            .collect::<Vec<_>>();
        let types = s.types.clone();
        let where_clause = s.where_clause.clone();
        // self.type_scope().goals.extend(s.where_clause.clone());
        // self.type_stack.pop();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        self.type_stack.push(TypeScope::new());
        let generics = self.map_generics(&s.generics);
        let where_clause = self.map_bounds(&s.where_clause);
        let types = self.map_trait_types(&s.types);
        self.type_scope().where_clause.extend(where_clause.clone());
        self.type_scope().where_clause.push(s.bound());
        let defs = self.map_trait_defs(&s.defs);
        self.type_stack.pop();
        StmtTrait::new(s.span, s.name, generics, where_clause, defs, types)
    }

    fn map_stmt_trait_def(&mut self, s: &StmtTraitDef) -> StmtTraitDef {
        let span = s.span;
        let name = s.name;
        let generics = self.map_generics(&s.generics);
        let params = self.map_params(&s.params).into();
        let ty = self.map_type(&s.ty);
        let where_clause = self.map_bounds(&s.where_clause);
        self.type_scope().where_clause.extend(where_clause.clone());
        GatherGoals::new(self).visit_stmt_trait_def(&s);
        self.solve_goals();
        StmtTraitDef::new(span, name, generics, params, ty, where_clause)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        match &s.body {
            StmtDefBody::UserDefined(e) => {
                self.type_stack.push(TypeScope::new());
                for (x, t) in &s.params {
                    self.bind(*x, Binding::Var(x.span, t.clone()));
                }
                let e = Annotate::new(self).map_expr(e);
                let e = self.map_expr(&e);
                self.unify(s.span, e.span_of(), &s.ty, e.type_of());
                let goals = std::mem::take(&mut self.type_scope().goals);
                for goal in goals {
                    // TODO: Try to solve each goal multiple times
                    if self.solve::<false>(&goal, &s.where_clause).is_err() {
                        // ctx.report
                        //     .err(goal.span, "Unsolved goal", "Could not solve goal");
                    }
                }
                let stmt = StmtDef::new(
                    s.span,
                    s.name,
                    s.generics.clone(),
                    s.params.clone(),
                    s.ty.clone(),
                    s.where_clause.clone(),
                    StmtDefBody::UserDefined(e),
                );
                Defaults::new(self).visit_stmt_def(&stmt);
                let s = Apply::new(self).map_stmt_def(&stmt);
                self.type_stack.pop();
                s
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
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Path(..) => unreachable!(),
            Expr::Int(s, t0, v) => Expr::Int(*s, t0.clone(), *v),
            Expr::Float(s, t0, v) => Expr::Float(*s, t0.clone(), *v),
            Expr::Bool(s, t0, v) => {
                let t1 = self.std.bool.clone();
                self.unify(*s, *s, t0, &t1);
                Expr::Bool(*s, t0.clone(), *v)
            }
            Expr::Char(s, t0, v) => {
                let t1 = self.std.char.clone();
                self.unify(*s, *s, t0, &t1);
                Expr::Char(*s, t0.clone(), *v)
            }
            Expr::String(s, t0, v) => {
                let t1 = self.std.string.clone();
                self.unify(*s, *s, t0, &t1);
                Expr::String(*s, t0.clone(), *v)
            }
            Expr::Struct(s, t0, x, ts, xes) => {
                let stmt = self.structs.get(x).unwrap().clone();
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
                        .find_map(|(x0, t)| (*x0 == x).then_some(t))
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
                let stmt = self.enums.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts.clone())
                    .collect::<Map<_, _>>();
                let t2 = stmt
                    .variants
                    .iter()
                    .find_map(|(x0, t)| (x0 == x1).then_some(t))
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
                let Binding::Var(s1, t1) = self.get(x).clone() else {
                    unreachable!()
                };
                self.unify(*s, s1, t0, &t1);
                Expr::Var(*s, t0.clone(), *x)
            }
            Expr::Def(s, t0, x, ts0) => {
                let stmt = self.defs.get(x).unwrap().clone();
                let gsub = stmt
                    .generics
                    .clone()
                    .into_iter()
                    .zip(ts0.clone())
                    .collect::<Map<_, _>>();
                let preds = stmt
                    .where_clause
                    .iter()
                    .map(|p| p.instantiate(&gsub))
                    .collect::<Vec<_>>();
                self.type_scope().goals.extend(preds);
                let t2 = Type::Fun(
                    stmt.params.values().cloned().collect::<Vec<_>>(),
                    Rc::new(stmt.ty.clone()),
                )
                .instantiate(&gsub);
                self.unify(*s, stmt.span, t0, &t2);
                Expr::Def(*s, t0.clone(), *x, ts0.clone())
            }
            Expr::Call(s, t0, e, es) => {
                let e = self.map_expr(e);
                let es = self.map_exprs(es);
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
                let t = Apply::new(self).map_type(&e.type_of());
                match t {
                    Type::Cons(x0, ts) => {
                        let stmt = self.structs.get(&x0).unwrap().clone();
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
                    Type::Record(ref xts) => {
                        let t1 = xts.iter().find_map(|(x1, t)| (x1 == x).then_some(t));
                        if let Some(t1) = t1 {
                            self.unify(*s, e.span_of(), t0, t1);
                            Expr::Field(*s, t0.clone(), e.into(), *x)
                        } else {
                            self.report.err(
                                *s,
                                "Unknown field",
                                format!("Field {x} not found in record {t}"),
                            );
                            Expr::Field(*s, Type::Err, e.into(), *x)
                        }
                    }
                    _ => {
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
                let t = Apply::new(self).map_type(&e.type_of());
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
                self.unify(*s, e0.span_of(), t0, &Type::Tuple(vec![]));
                Expr::Assign(*s, t0.clone(), Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, _) => todo!(),
            Expr::Continue(_, _) => todo!(),
            Expr::Break(_, _) => todo!(),
            Expr::Fun(_, _, _, _, _) => todo!(),
            Expr::Match(_, _, _, _) => todo!(),
            Expr::While(_, _, _, _) => todo!(),
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
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Assoc(s, t0, b, x0, ts0) => match b {
                Bound::Path(_, _) => unreachable!(),
                Bound::Trait(_, x1, ts1, _) => {
                    let stmt0 = self.traits.get(x1).unwrap();
                    let stmt1 = stmt0.defs.iter().find(|d| &d.name == x0).unwrap();
                    let gsub = stmt0
                        .generics
                        .clone()
                        .into_iter()
                        .chain(stmt1.generics.clone())
                        .zip(ts1.clone().into_iter().chain(ts0.clone()))
                        .collect::<Map<_, _>>();
                    let t1 = Type::Fun(
                        stmt1.params.values().cloned().collect::<Vec<_>>(),
                        stmt1.ty.clone().into(),
                    )
                    .annotate(self)
                    .instantiate(&gsub);
                    self.type_scope().goals.push(b.clone());
                    self.unify(*s, *s, t0, &t1);
                    Expr::Assoc(*s, t0.clone(), b.clone(), *x0, ts0.clone())
                }
                Bound::Type(_, _) => todo!(),
                Bound::Err(_) => todo!(),
            },
            Expr::Unresolved(_, _t, _x, _ts) => todo!(),
            Expr::Query(..) => todo!(),
            Expr::QueryInto(_, _, _, _, _, _) => unreachable!(),
            Expr::InfixBinaryOp(_, _, _, _, _) => unreachable!(),
            Expr::PrefixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::PostfixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::Annotate(_, _, _) => unreachable!(),
            Expr::Paren(_, _, _) => unreachable!(),
            Expr::Dot(_, _, _, _, _, _) => unreachable!(),
            Expr::IfElse(_, _, _, _, _) => unreachable!(),
            Expr::IntSuffix(_, _, _, _) => unreachable!(),
            Expr::FloatSuffix(_, _, _, _) => unreachable!(),
        }
    }
}
