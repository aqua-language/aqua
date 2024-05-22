use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Block;
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
use crate::ast::StmtVar;
use crate::ast::Type;

#[derive(Debug, Default)]
pub struct Context {
    ids: HashMap<Name, HashMap<Vec<Type>, Name>>,
    defs: HashMap<Name, StmtDef>,
    structs: HashMap<Name, StmtStruct>,
    enums: HashMap<Name, StmtEnum>,
    impls: Vec<StmtImpl>,
    stmts: Vec<Stmt>,
    stack: Stack,
}

#[derive(Debug, Default)]
struct Stack(Vec<Scope>);

#[derive(Debug, Default)]
struct Scope(Map<Name, Type>);

impl Stack {
    fn bind(&mut self, x: Name, t: Type) {
        self.0.last_mut().unwrap().0.insert(x, t);
    }

    fn get(&self, x: &Name) -> Type {
        self.0
            .iter()
            .rev()
            .find_map(|s| s.0.get(x))
            .unwrap()
            .clone()
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn monomorphise(&mut self, p: &Program) -> Program {
        p.stmts.iter().for_each(|stmt| self.decl_stmt(stmt));
        let stmts = p
            .stmts
            .iter()
            .filter_map(|stmt| self.expr_stmt(stmt))
            .collect();
        Program::new(stmts)
    }

    fn mangled(&self, name: &Name, ts: &Vec<Type>) -> Option<Name> {
        self.ids
            .get(name)
            .and_then(|instances| instances.get(ts).copied())
    }

    fn mangle(&mut self, x: Name, ts: Vec<Type>) -> Name {
        self.ids
            .get_mut(&x)
            .map(|insts| {
                let len = insts.len();
                insts.entry(ts).or_insert_with(|| x.suffix(len))
            })
            .copied()
            .unwrap()
    }

    fn decl_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => self.decl_stmt_def(s),
            Stmt::Trait(_) => {}
            Stmt::Impl(s) => self.decl_stmt_impl(s),
            Stmt::Struct(s) => self.decl_stmt_struct(s),
            Stmt::Enum(s) => self.decl_stmt_enum(s),
            Stmt::Type(_) => {}
            Stmt::Expr(_) => {}
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn expr_stmt(&mut self, s: &Stmt) -> Option<Stmt> {
        match s {
            Stmt::Var(s) => Some(Stmt::Var(Rc::new(self.stmt_var(s)))),
            Stmt::Def(_) => None,
            Stmt::Trait(_) => None,
            Stmt::Impl(_) => None,
            Stmt::Struct(_) => None,
            Stmt::Enum(_) => None,
            Stmt::Type(_) => None,
            Stmt::Expr(e) => Some(Stmt::Expr(Rc::new(self.expr(e)))),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.stmt_var(s))),
            Stmt::Def(_) => unreachable!(),
            Stmt::Trait(_) => unreachable!(),
            Stmt::Impl(_) => unreachable!(),
            Stmt::Struct(_) => unreachable!(),
            Stmt::Enum(_) => unreachable!(),
            Stmt::Type(_) => unreachable!(),
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.expr(e))),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn decl_stmt_def(&mut self, s: &StmtDef) {
        self.defs.insert(s.name, s.clone());
    }

    fn decl_stmt_struct(&mut self, s: &StmtStruct) {
        self.structs.insert(s.name, s.clone());
    }

    fn decl_stmt_enum(&mut self, s: &StmtEnum) {
        self.enums.insert(s.name, s.clone());
    }

    fn decl_stmt_impl(&mut self, s: &StmtImpl) {
        self.impls.push(s.clone());
    }

    fn stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let x = s.name;
        let t = self.ty(&s.ty);
        let e = self.expr(&s.expr);
        StmtVar::new(span, x, t, e)
    }

    fn stmt_def(&mut self, s: &StmtDef, ts: &Vec<Type>) -> Name {
        self.mangled(&s.name, ts).unwrap_or_else(|| {
            self.scoped(|ctx| {
                let x = ctx.mangle(s.name, ts.clone());
                s.generics
                    .iter()
                    .zip(ts)
                    .for_each(|(g, t)| ctx.stack.bind(*g, t.clone()));
                let span = s.span;
                let ps = s.params.map_values(|t| ctx.ty(t));
                let t = ctx.ty(&s.ty);
                let b = StmtDefBody::UserDefined(ctx.expr(s.body.as_expr()));
                ctx.stmts.push(Stmt::Def(Rc::new(StmtDef::new(
                    span,
                    x,
                    vec![],
                    ps,
                    t,
                    vec![],
                    b,
                ))));
                x
            })
        })
    }

    fn stmt_struct(&mut self, x: &Name, ts: &Vec<Type>) -> Name {
        let s = self.structs.get(x).unwrap().clone();
        self.mangled(&s.name, ts).unwrap_or_else(|| {
            self.scoped(|ctx| {
                let x = ctx.mangle(s.name, ts.clone());
                s.generics
                    .iter()
                    .zip(ts)
                    .for_each(|(g, t)| ctx.stack.bind(*g, t.clone()));
                let span = s.span;
                let fields = s
                    .fields
                    .iter()
                    .map(|(x, t)| (*x, ctx.ty(t)))
                    .collect();
                ctx.stmts.push(Stmt::Struct(Rc::new(StmtStruct::new(
                    span,
                    x,
                    vec![],
                    fields,
                ))));
                x
            })
        })
    }

    fn stmt_enum(&mut self, x: &Name, ts: &Vec<Type>) -> Name {
        let s = self.enums.get(x).unwrap().clone();
        self.mangled(&s.name, ts).unwrap_or_else(|| {
            self.scoped(|ctx| {
                let x = ctx.mangle(s.name, ts.clone());
                s.generics
                    .iter()
                    .zip(ts)
                    .for_each(|(g, t)| ctx.stack.bind(*g, t.clone()));
                let span = s.span;
                let variants = s.variants.map_values(|t| ctx.ty(t));
                ctx.stmts.push(Stmt::Enum(Rc::new(StmtEnum::new(
                    span,
                    x,
                    vec![],
                    variants,
                ))));
                x
            })
        })
    }

    fn _stmt_impl(&mut self, _s: &StmtImpl, _ts: &[Type]) -> StmtImpl {
        // let span = s.span;
        // let tb = s.trait_bound.clone();
        // let x = s.name.clone();
        // let ts = s.params.clone();
        // let defs = s.defs.iter().map(|d| self.stmt_def(d)).collect();
        // StmtImpl::new(span, tb, x, ts, defs)
        todo!()
    }

    fn scoped<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.stack.0.push(Scope::default());
        let result = f(self);
        self.stack.0.pop();
        result
    }

    fn block(&mut self, b: &Block) -> Block {
        self.scoped(|this| {
            let span = b.span;
            let stmts = b.stmts.iter().map(|s| this.stmt(s)).collect();
            let expr = this.expr(&b.expr);
            Block::new(span, stmts, expr)
        })
    }

    pub fn expr(&mut self, e: &Expr) -> Expr {
        let t = self.ty(e.ty());
        let s = e.span();
        match e {
            Expr::Unresolved(_, _, _) => unreachable!(),
            Expr::Int(_, _, v) => Expr::Int(s, t, *v),
            Expr::Float(_, _, v) => Expr::Float(s, t, *v),
            Expr::Bool(_, _, b) => Expr::Bool(s, t, *b),
            Expr::Char(_, _, v) => Expr::Char(s, t, *v),
            Expr::String(_, _, v) => Expr::String(s, t, *v),
            Expr::Struct(_, _, x, ts, xes) => {
                let x = self.stmt_struct(x, ts);
                let xvs = xes.iter().map(|(n, e)| (*n, self.expr(e))).collect();
                Expr::Struct(s, t, x, vec![], xvs)
            }
            Expr::Tuple(_, _, es) => {
                let vs = es.iter().map(|e| self.expr(e)).collect();
                Expr::Tuple(s, t, vs)
            }
            Expr::Record(_, _, xes) => {
                let vs = xes.iter().map(|(n, e)| (*n, self.expr(e))).collect();
                Expr::Record(s, t, vs)
            }
            Expr::Enum(_, _, x, ts, x1, e) => {
                let x = self.stmt_enum(x, ts);
                let e = self.expr(e);
                Expr::Enum(s, t, x, vec![], *x1, Rc::new(e))
            }
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                Expr::Field(s, t, Rc::new(e), *x)
            }
            Expr::Index(_, _, e, i) => {
                let e = self.expr(e);
                Expr::Index(s, t, Rc::new(e), *i)
            }
            Expr::Var(_, _, x) => Expr::Var(s, t, *x),
            Expr::Def(_, _, x, ts) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect::<Vec<_>>();
                let stmt = self.defs.get(x).unwrap();
                match &stmt.body {
                    StmtDefBody::UserDefined(_) => {
                        let x = self.stmt_def(&stmt.clone(), &ts);
                        Expr::Def(s, t, x, vec![])
                    }
                    StmtDefBody::Builtin(_) => Expr::Def(s, t, *x, ts),
                }
            }
            Expr::Assoc(_, _, tb, x, ts) => {
                let x = self.mangle(*x, ts.clone());
                Expr::Assoc(s, t, tb.clone(), x, vec![])
            }
            Expr::Call(_, _, e, es) => {
                let e = self.expr(e);
                let vs = es.iter().map(|e| self.expr(e)).collect();
                Expr::Call(s, t, Rc::new(e), vs)
            }
            Expr::Block(_, _, b) => {
                let b = self.block(b);
                Expr::Block(s, t, b)
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Match(_, _, _e, _pes) => todo!(),
            Expr::Array(_, _, _) => todo!(),
            Expr::Assign(_, _, _, _) => todo!(),
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::While(_, _, e, b) => {
                let e = self.expr(e);
                let b = self.block(b);
                Expr::While(s, t, Rc::new(e), b)
            }
            Expr::Fun(_, _, _, _, _) => unreachable!(),
            Expr::Err(_, _) => unreachable!(),
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => todo!(),
        }
    }

    fn ty(&mut self, t: &Type) -> Type {
        match t {
            Type::Unresolved(_) => unreachable!(),
            Type::Cons(x, ts) => {
                let x = self.mangle(*x, ts.clone());
                Type::Cons(x, vec![])
            }
            Type::Alias(..) => unreachable!(),
            Type::Assoc(_, _, _) => todo!(),
            Type::Var(_) => unreachable!(),
            Type::Generic(x) => self.stack.get(x),
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
            Type::Array(_, _) => todo!(),
            Type::Never => Type::Never,
            Type::Hole => Type::Hole,
            Type::Err => unreachable!(),
        }
    }
}
