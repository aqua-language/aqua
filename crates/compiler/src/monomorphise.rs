pub mod mangle;

use std::collections::HashSet;
use std::rc::Rc;

use ena::unify::InPlaceUnificationTable;

use mangle::Mangler;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::declare;
use crate::infer::instantiate::Instantiate;
use crate::infer::type_var::TypeVarKind;
use crate::infer::type_var::TypeVarValue;
use crate::traversal::mapper::AcceptMapper;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::AcceptVisitor;

#[derive(Debug, Default)]
pub struct Context {
    unique: HashSet<Name>,
    decls: declare::Context,
    stmts: Vec<Stmt>,
    table: InPlaceUnificationTable<TypeVar>,
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn monomorphise(&mut self, p: &Program) -> Program {
        p.visit(&mut self.decls);
        let stmts1 = p
            .stmts
            .iter()
            .filter_map(|stmt| self.top_stmt(stmt))
            .collect::<Vec<_>>();
        let mut stmts0 = std::mem::take(&mut self.stmts);
        stmts0.extend(stmts1);
        Program::new(p.span, stmts0)
    }

    pub fn fresh(&mut self, kind: TypeVarKind) -> Type {
        Type::Var(self.table.new_key(TypeVarValue::Unknown(kind)))
    }

    fn top_stmt(&mut self, s: &Stmt) -> Option<Stmt> {
        match s {
            Stmt::Var(s) => Some(Stmt::Var(Rc::new(self.map_stmt_var(s)))),
            Stmt::Def(_) => None,
            Stmt::Trait(_) => None,
            Stmt::Impl(_) => None,
            Stmt::Struct(_) => None,
            Stmt::Enum(_) => None,
            Stmt::Type(_) => None,
            Stmt::Expr(e) => Some(Stmt::Expr(Rc::new(self.map_expr(e)))),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn monomorphise_stmt_def(&mut self, stmt: &StmtDef, ts: &[Type]) -> Name {
        let x = Mangler::mangle_fun(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        } else {
            self.unique.insert(x);
        }
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = stmt.map(&mut Instantiate::new(&gsub));
        let ps = stmt.params.mapv(|t| self.map_type(t));
        let t = stmt.ty.map(self);
        let b = ExprBody::UserDefined(Rc::new(stmt.body.as_expr().map(self)));
        let stmt = StmtDef::new(stmt.span, x, vec![], ps, t, vec![], b);
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
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_struct(stmt);
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
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_enum(stmt);
        let variants = stmt.variants.mapv(|t| self.map_type(t));
        let stmt = StmtEnum::new(stmt.span, x, vec![], variants);
        self.stmts.push(Stmt::Enum(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_impl_def(
        &mut self,
        stmt: &StmtImpl,
        x0: Name,
        ts0: &[Type],
        x1: Name,
        ts1: &[Type],
    ) -> Name {
        let x = Mangler::mangle_impl_def(x0, ts0, x1, ts1);
        if self.unique.contains(&x) {
            return x;
        } else {
            self.unique.insert(x);
        }
        let stmt_def = stmt.get_def(x1).unwrap();
        let gsub = stmt_def
            .generics
            .clone()
            .into_iter()
            .zip(ts0.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_def(stmt_def);
        let ps = self.map_params(&stmt_def.params).into();
        let t = self.map_type(&stmt_def.ty);
        let b = self.map_stmt_def_body(&stmt_def.body);
        let stmt = StmtDef::new(stmt.span, x, vec![], ps, t, vec![], b);
        self.stmts.push(Stmt::Def(Rc::new(stmt)));
        x
    }

    fn solve(&mut self, goal: &Trait) -> Option<StmtImpl> {
        let snapshot = self.table.snapshot();
        let Trait::Cons(x, _, _) = goal else {
            todo!();
        };
        if let Some(impls) = self.decls.trait_impls.get(x).cloned() {
            for stmt in impls {
                let gsub = stmt
                    .generics
                    .iter()
                    .map(|x| (*x, self.fresh(TypeVarKind::General)))
                    .collect::<Map<_, _>>();
                let stmt = Instantiate::new(&gsub).map_stmt_impl(stmt.as_ref());
                if self.matches(goal, &stmt.head)
                    && stmt
                        .where_clause
                        .iter()
                        .all(|subgoal| self.solve(subgoal).is_some())
                {
                    self.table.commit(snapshot);
                    return Some(stmt);
                }
            }
        }
        self.table.rollback_to(snapshot);
        None
    }

    fn matches(&mut self, b0: &Trait, b1: &Trait) -> bool {
        match (b0, b1) {
            (Trait::Path(..), Trait::Path(..)) => unreachable!(),
            (Trait::Cons(x0, ts0, xts0), Trait::Cons(x1, ts1, xts1)) => {
                x0 == x1
                    && ts0.len() == ts1.len()
                    && ts1
                        .iter()
                        .zip(ts0.iter())
                        .all(|(t0, t1)| self.try_unify(t0, t1).is_ok())
                    && xts1.iter().zip(xts0.iter()).all(|((x0, t0), (x1, t1))| {
                        assert_eq!(x0, x1);
                        self.try_unify(t0, t1).is_ok()
                    })
            }
            (Trait::Type(..), Trait::Type(..)) => todo!(),
            (Trait::Err, Trait::Err) => unreachable!(),
            _ => unreachable!(),
        }
    }

    pub fn try_unify(&mut self, t0: &Type, t1: &Type) -> Result<(), (Type, Type)> {
        match (t0, t1) {
            (Type::Var(x0), t) | (t, Type::Var(x0)) => {
                let k0 = self.table.probe_value(*x0).unknown().unwrap();
                match t {
                    Type::Var(x1) => {
                        let k1 = self.table.probe_value(*x1).unknown().unwrap();
                        if k0.is_compatible(k1) {
                            self.table.union(*x0, *x1);
                            Ok(())
                        } else {
                            Err((t0.clone(), t1.clone()))
                        }
                    }
                    _ => {
                        self.table.union_value(*x0, TypeVarValue::Known(t.clone()));
                        Ok(())
                    }
                }
            }
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
                    Err((t0.clone(), t1.clone()))
                }
            }
            (Type::Generic(..), Type::Generic(..)) => unreachable!(),
            (Type::Assoc(..), Type::Assoc(..)) => unreachable!(),
            (Type::Assoc(b, x, _), t0) | (t0, Type::Assoc(b, x, _)) => {
                if let Some(t1) = b.as_type(x) {
                    self.try_unify(t0, t1)
                } else {
                    Err((t0.clone(), t1.clone()))
                }
            }
            (Type::Err, _) | (_, Type::Err) => unreachable!(),
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            (Type::Unknown, Type::Unknown) => unreachable!(),
            _ => Err((t0.clone(), t1.clone())),
        }
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        let t = self.map_type(e.type_of());
        let s = e.span_of();
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
            Expr::TraitMethod(_, _, b, x1, ts1) => {
                let stmt = self.solve(b).unwrap();
                let Trait::Cons(x0, ts0, _) = b else {
                    unreachable!()
                };
                let x = self.monomorphise_stmt_impl_def(&stmt, *x0, &ts0, *x1, ts1);
                Expr::Def(s, t, x, vec![])
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
            _ => self._map_expr(e),
        }
    }

    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Cons(x, ts) => {
                if let Some(_) = self.decls.types.get(x).cloned() {
                    let ts = self.map_types(ts);
                    Type::Cons(*x, ts)
                } else if let Some(stmt) = self.decls.structs.get(x).cloned() {
                    let x = self.monomorphise_stmt_struct(stmt.as_ref(), ts);
                    Type::Cons(x, vec![])
                } else if let Some(stmt) = self.decls.enums.get(x).cloned() {
                    let x = self.monomorphise_stmt_enum(stmt.as_ref(), ts);
                    Type::Cons(x, vec![])
                } else {
                    unreachable!()
                }
            }
            _ => self._map_type(t),
        }
    }
}
