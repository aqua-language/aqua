//! Lift defs to the top-level
//! * Assume defs can only capture defs, and not vars or generics.

use std::rc::Rc;

use runtime::HashMap;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Map;
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
use crate::ast::TraitBound;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::Type;
use crate::diag::Report;

#[derive(Debug)]
pub struct Context {
    stack: Stack,
    pub top: Vec<Stmt>,
    pub unique: HashMap<Name, usize>,
    pub report: Report,
}

#[derive(Debug)]
struct Stack {
    unique: HashMap<Name, usize>,
    scopes: Vec<Scope>,
}

impl Stack {
    fn new() -> Stack {
        Stack {
            unique: HashMap::default(),
            scopes: vec![Scope::new()],
        }
    }

    fn bind(&mut self, old: Name) -> Name {
        let uid = self
            .unique
            .entry(old)
            .and_modify(|uid| *uid += 1)
            .or_insert_with(|| 0);
        let new = if *uid == 0 { old } else { old.suffix(uid) };
        self.scopes.last_mut().unwrap().0.insert(old, new);
        new
    }

    #[track_caller]
    fn get(&self, x: &Name) -> Name {
        if let Some(x) = self.scopes.iter().rev().find_map(|s| s.0.get(x)) {
            *x
        } else {
            panic!(
                "Variable not found: {:?}: {}",
                x,
                std::panic::Location::caller()
            )
        }
    }
}

#[derive(Debug)]
struct Scope(Map<Name, Name>);

impl Scope {
    fn new() -> Scope {
        Scope(Map::new())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            stack: Stack::new(),
            top: vec![],
            unique: HashMap::default(),
            report: Report::new(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    fn scoped<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.stack.scopes.push(Scope::new());
        let t = f(self);
        self.stack.scopes.pop();
        t
    }

    pub fn lift(&mut self, program: &Program) -> Program {
        program.stmts.iter().for_each(|s| self.decl_stmt(s));
        program.stmts.iter().for_each(|s| self.top_stmt(s));
        let stmts = std::mem::take(&mut self.top);
        Program::new(stmts)
    }

    fn top_stmt(&mut self, stmt: &Stmt) {
        let stmt = self.stmt(stmt);
        self.top.push(stmt);
    }

    fn decl_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.stack.bind(s.name);
            }
            Stmt::Trait(s) => {
                self.stack.bind(s.name);
                for s in &s.defs {
                    self.stack.bind(s.name);
                }
                for s in &s.types {
                    self.stack.bind(s.name);
                }
            }
            Stmt::Impl(s) => {}
            Stmt::Struct(s) => {
                self.stack.bind(s.name);
            }
            Stmt::Enum(s) => {
                self.stack.bind(s.name);
            }
            Stmt::Type(s) => {
                self.stack.bind(s.name);
            }
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }

    fn stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.stmt_var(s))),
            Stmt::Def(s) => Stmt::Def(Rc::new(self.stmt_def(s))),
            Stmt::Trait(s) => Stmt::Trait(Rc::new(self.stmt_trait(s))),
            Stmt::Impl(s) => Stmt::Impl(Rc::new(self.stmt_impl(s))),
            Stmt::Struct(s) => Stmt::Struct(Rc::new(self.stmt_struct(s))),
            Stmt::Enum(s) => Stmt::Enum(Rc::new(self.stmt_enum(s))),
            Stmt::Type(s) => Stmt::Type(Rc::new(self.stmt_type(s))),
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.expr(e))),
            Stmt::Err(s) => Stmt::Err(*s),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let ty = self.ty(&s.ty);
        let expr = self.expr(&s.expr);
        let name = self.stack.get(&s.name);
        StmtVar::new(span, name, ty, expr)
    }

    fn stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        let name = self.stack.get(&s.name);
        self.scoped(|ctx| {
            let span = s.span;
            let generics = s.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let params = s.params.map(|x, t| (ctx.stack.bind(*x), ctx.ty(t)));
            let ty = ctx.ty(&s.ty);
            let where_clause = s.where_clause.iter().map(|b| ctx.bound(b)).collect();
            let body = ctx.stmt_def_body(&s.body);
            StmtDef::new(span, name, generics, params, ty, where_clause, body)
        })
    }

    fn bound(&mut self, b: &Bound) -> Bound {
        match b {
            Bound::Unresolved(..) => unreachable!(),
            Bound::Trait(s, b) => {
                let b = self.trait_bound(b);
                Bound::Trait(*s, b)
            }
            Bound::Err(s) => Bound::Err(*s),
        }
    }

    fn trait_bound(&mut self, b: &TraitBound) -> TraitBound {
        let x = self.stack.get(&b.name);
        let ts = b.ts.iter().map(|t| self.ty(t)).collect();
        let xts = b.xts.iter().map(|(n, t)| (*n, self.ty(t))).collect();
        TraitBound::new(x, ts, xts)
    }

    fn stmt_def_body(&mut self, s: &StmtDefBody) -> StmtDefBody {
        match s {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(self.expr(e)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }

    fn stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        let name = self.stack.get(&s.name);
        self.scoped(|ctx| {
            let span = s.span;
            let generics = s.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let where_clause = s.where_clause.iter().map(|b| ctx.bound(b)).collect();
            let defs = s.defs.iter().map(|d| Rc::new(ctx.trait_def(d))).collect();
            let types = s.types.iter().map(|t| Rc::new(ctx.trait_type(t))).collect();
            StmtTrait::new(span, name, generics, where_clause, defs, types)
        })
    }

    fn trait_def(&mut self, d: &StmtTraitDef) -> StmtTraitDef {
        let name = d.name;
        self.scoped(|ctx| {
            let span = d.span;
            let generics = d.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let params = d.params.map_values(|t| ctx.ty(t));
            let ty = ctx.ty(&d.ty);
            let where_clause = d.where_clause.iter().map(|b| ctx.bound(b)).collect();
            StmtTraitDef::new(span, name, generics, params, ty, where_clause)
        })
    }

    fn trait_type(&mut self, t: &StmtTraitType) -> StmtTraitType {
        self.scoped(|ctx| {
            let span = t.span;
            let name = t.name;
            let generics = t.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            StmtTraitType::new(span, name, generics)
        })
    }

    fn stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.scoped(|ctx| {
            let span = s.span;
            let generics = s.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let head = ctx.bound(&s.head);
            let where_clause = s.where_clause.iter().map(|b| ctx.bound(b)).collect();
            let defs = s.defs.iter().map(|d| Rc::new(ctx.stmt_def(d))).collect();
            let types = s.types.iter().map(|t| Rc::new(ctx.stmt_type(t))).collect();
            StmtImpl::new(span, generics, head, where_clause, defs, types)
        })
    }

    fn stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let name = self.stack.get(&s.name);
        self.scoped(|ctx| {
            let span = s.span;
            let generics = s.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let fields = s.fields.iter().map(|(x, t)| (*x, ctx.ty(t))).collect();
            StmtStruct::new(span, name, generics, fields)
        })
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let name = self.stack.get(&s.name);
        self.scoped(|ctx| {
            let span = s.span;
            let generics = s.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let variants = s.variants.iter().map(|(x, t)| (*x, ctx.ty(t))).collect();
            StmtEnum::new(span, name, generics, variants)
        })
    }

    fn stmt_type(&mut self, t: &StmtType) -> StmtType {
        let name = self.stack.get(&t.name);
        self.scoped(|ctx| {
            let span = t.span;
            let generics = t.generics.iter().map(|g| ctx.stack.bind(*g)).collect();
            let ty = ctx.stmt_type_body(&t.body);
            StmtType::new(span, name, generics, ty)
        })
    }

    fn stmt_type_body(&mut self, s: &StmtTypeBody) -> StmtTypeBody {
        match s {
            StmtTypeBody::UserDefined(t) => StmtTypeBody::UserDefined(self.ty(t)),
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
        }
    }

    fn ty(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Unresolved(..) => unreachable!(),
            Type::Cons(x, ts) => {
                let x = self.stack.get(x);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Type::Cons(x, ts)
            }
            Type::Alias(x, ts) => {
                let x = self.stack.get(x);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Type::Alias(x, ts)
            }
            Type::Assoc(b, x, ts) => {
                let b = self.trait_bound(b);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Type::Assoc(b, *x, ts)
            }
            Type::Var(..) => unreachable!(),
            Type::Generic(x) => {
                let x = self.stack.get(x);
                Type::Generic(x)
            }
            Type::Fun(ts, t) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                let t = self.ty(t);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, self.ty(t))).collect();
                Type::Record(xts)
            }
            Type::Array(t, n) => {
                let t = self.ty(t);
                Type::Array(Rc::new(t), *n)
            }
            Type::Never => Type::Never,
            Type::Hole => Type::Hole,
            Type::Err => Type::Err,
        }
    }

    fn expr(&mut self, expr: &Expr) -> Expr {
        let s = expr.span();
        let t = self.ty(expr.ty());
        match expr {
            Expr::Unresolved(_, _, _) => unreachable!(),
            Expr::Int(_, _, v) => Expr::Int(s, t, *v),
            Expr::Float(_, _, v) => Expr::Float(s, t, *v),
            Expr::Bool(_, _, v) => Expr::Bool(s, t, *v),
            Expr::String(_, _, v) => Expr::String(s, t, *v),
            Expr::Char(_, _, v) => Expr::Char(s, t, *v),
            Expr::Struct(_, _, x, ts, xes) => {
                let x = self.stack.get(x);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                let xes = xes.iter().map(|(x, e)| (*x, self.expr(e))).collect();
                Expr::Struct(s, t, x, ts, xes)
            }
            Expr::Tuple(_, _, es) => {
                let es = es.iter().map(|e| self.expr(e)).collect();
                Expr::Tuple(s, t, es)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes.iter().map(|(x, e)| (*x, self.expr(e))).collect();
                Expr::Record(s, t, xes)
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                let x0 = self.stack.get(x0);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                let x1 = self.stack.get(x1);
                let e = self.expr(e);
                Expr::Enum(s, t, x0, ts, x1, Rc::new(e))
            }
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                Expr::Field(s, t, Rc::new(e), *x)
            }
            Expr::Index(_, _, e, i) => {
                let e = self.expr(e);
                let i = *i;
                Expr::Index(s, t, Rc::new(e), i)
            }
            Expr::Var(_, _, x) => {
                let x = self.stack.get(x);
                Expr::Var(s, t, x)
            }
            Expr::Def(_, _, x, ts) => {
                let x = self.stack.get(x);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Expr::Def(s, t, x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = self.expr(e);
                let es = es.iter().map(|e| self.expr(e)).collect();
                Expr::Call(s, t, Rc::new(e), es)
            }
            Expr::Block(_, _, b) => {
                let b = self.block(b);
                Expr::Block(s, t, b)
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, b, x, ts) => {
                let b = self.trait_bound(b);
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Expr::Assoc(s, t, b, *x, ts)
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::Array(_, _, es) => {
                let es = es.iter().map(|e| self.expr(e)).collect();
                Expr::Array(s, t, es)
            }
            Expr::Assign(_, _, e0, e1) => {
                let e0 = self.expr(e0);
                let e1 = self.expr(e1);
                Expr::Assign(s, t, Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, e) => {
                let e = self.expr(e);
                Expr::Return(s, t, Rc::new(e))
            }
            Expr::Continue(_, _) => Expr::Continue(s, t),
            Expr::Break(_, _) => todo!(),
            Expr::While(_, _, e, b) => {
                let e = self.expr(e);
                let b = self.block(b);
                Expr::While(s, t, Rc::new(e), b)
            }
            Expr::Fun(_, _, _ps, _t1, _e) => {
                todo!()
                // let ps = ps
                //     .iter()
                //     .map(|p| Param::new(p.span, p.name, self.ty(&p.ty)))
                //     .collect();
                // let t1 = self.ty(t1);
                // let e = self.expr(e);
                // Expr::Fun(s, t, ps, t1, Rc::new(e))
            }
            Expr::For(_, _, x, e, b) => {
                let x = self.stack.get(x);
                let e = self.expr(e);
                let b = self.block(b);
                Expr::For(s, t, x, Rc::new(e), b)
            }
            Expr::Err(_, _) => Expr::Err(s, t),
            Expr::Value(_, _) => unreachable!(),
        }
    }

    fn block(&mut self, b: &Block) -> Block {
        self.scoped(|ctx| {
            let span = b.span;
            b.stmts.iter().for_each(|s| ctx.decl_stmt(s));
            let stmts = b.stmts.iter().flat_map(|s| ctx.lift_stmt(s)).collect();
            let expr = ctx.expr(&b.expr);
            Block::new(span, stmts, expr)
        })
    }

    fn lift_stmt(&mut self, stmt: &Stmt) -> Option<Stmt> {
        match stmt {
            Stmt::Var(s) => Some(Stmt::Var(Rc::new(self.stmt_var(s)))),
            Stmt::Expr(e) => Some(Stmt::Expr(Rc::new(self.expr(e)))),
            Stmt::Err(s) => Some(Stmt::Err(*s)),
            Stmt::Def(_)
            | Stmt::Trait(_)
            | Stmt::Impl(_)
            | Stmt::Struct(_)
            | Stmt::Enum(_)
            | Stmt::Type(_) => {
                self.top_stmt(stmt);
                None
            }
        }
    }
}
