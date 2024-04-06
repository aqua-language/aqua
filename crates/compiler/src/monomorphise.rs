use std::collections::HashMap;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
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
use crate::ast::Uid;
use crate::builtins::Fun;
use crate::builtins::Record;
use crate::builtins::Tuple;
use crate::builtins::Value;
use crate::builtins::Variant;

#[derive(Debug, Default)]
pub struct Context {
    ids: HashMap<(Name, Vec<Type>), Uid>,
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
struct Scope(Vec<(Name, Binding)>);

#[derive(Debug)]
enum Binding {
    Generic(Type),
    Var(Type),
}

impl Context {
    fn monomorphise(&mut self, name: Name, ts: Vec<Type>) {
        self.ids
            .entry((name, ts))
            .and_modify(|uid| *uid = uid.increment())
            .or_insert_with(|| name.uid);
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn interpret(&mut self, p: &Program) {
        p.stmts.iter().for_each(|stmt| self.decl_stmt(stmt));
        p.stmts.iter().for_each(|stmt| self.stmt(stmt));
    }

    // First pass: Declare all items so that they can later be instantiated
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

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(s) => self.stmt_def(s),
            Stmt::Trait(_) => {}
            Stmt::Impl(s) => self.stmt_impl(s),
            Stmt::Struct(_) => {}
            Stmt::Enum(_) => {}
            Stmt::Type(_) => {}
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let x = s.name.clone();
        let t = self.ty(&s.ty);
        let e = self.expr(&s.expr);
        StmtVar::new(span, x, t, e)
    }

    fn decl_stmt_def(&mut self, s: &StmtDef) {
        self.defs.insert(s.name.clone(), s.clone());
    }

    fn decl_stmt_struct(&mut self, s: &StmtStruct) {
        self.structs.insert(s.name.clone(), s.clone());
    }

    fn decl_stmt_enum(&mut self, s: &StmtEnum) {
        self.enums.insert(s.name.clone(), s.clone());
    }

    fn decl_stmt_impl(&mut self, s: &StmtImpl) {
        self.stmts.push(tb.clone(), s.clone());
    }

    fn stmt_expr(&mut self, _: &Expr) {
        todo!()
    }

    fn block(&mut self, b: &Block) -> Block {
        self.scoped(|this| {
            b.stmts.iter().for_each(|s| this.stmt(s));
            this.expr(&b.expr)
        })
    }

    pub fn expr(&mut self, e: &Expr) -> Expr {
        let t = self.ty(&e.ty());
        let s = e.span();
        match e {
            Expr::Unresolved(_, _, _) => unreachable!(),
            Expr::Int(_, _, v) => Expr::Int(s, t, v.clone()),
            Expr::Float(_, _, v) => Expr::Float(s, t, v.clone()),
            Expr::Bool(_, _, b) => Expr::Bool(s, t, *b),
            Expr::Char(_, _, v) => Expr::Char(s, t, v.clone()),
            Expr::String(_, _, v) => Expr::String(s, t, v.clone()),
            Expr::Struct(_, _, x, ts, xes) => {
                let x = self.ids.get_def(x).clone();
                let xvs = xes.iter().map(|(n, e)| (n.clone(), self.expr(e))).collect();
                Expr::Struct(s, t, xvs)
            }
            Expr::Tuple(_, _, es) => {
                let vs = es.iter().map(|e| self.expr(e)).collect();
                Value::from(Tuple::new(vs))
            }
            Expr::Record(_, _, xes) => {
                let xvs = xes.iter().map(|(n, e)| (n.clone(), self.expr(e))).collect();
                Value::from(Record::new(xvs))
            }
            Expr::Enum(_, _, _, _, x1, e) => {
                let v = self.expr(e);
                Value::from(Variant::new(x1.clone(), v))
            }
            Expr::Field(_, _, e, x) => {
                let rec = self.expr(e).as_record();
                rec[x].clone()
            }
            Expr::Index(_, _, e, i) => {
                let tup = self.expr(e).as_tuple();
                tup[i].clone()
            }
            Expr::Var(_, _, x) => self.stack.get_val(x).clone(),
            Expr::Def(_, _, x, ts) => Value::from(Fun::new_def(x.clone(), ts.clone())),
            Expr::Assoc(_, _, tb, x, ts) => {
                Value::from(Fun::new_assoc(tb.clone(), x.clone(), ts.clone()))
            }
            Expr::Call(_, _, e, es) => {
                let f = self.expr(e).as_function();
                let vs = es.iter().map(|e| self.expr(e)).collect::<Vec<_>>();
                match f {
                    Fun::Def(x, ts) => {
                        let s = self.stack.get_def(&x).clone();
                        match s.body {
                            StmtDefBody::UserDefined(e) => self.scoped(|ctx| {
                                for (p, v) in s.params.iter().zip(vs) {
                                    ctx.stack.bind_val(p.name.clone(), v)
                                }
                                ctx.expr(&e)
                            }),
                            StmtDefBody::Builtin(builtin) => (builtin.fun)(self, &ts, &vs),
                        }
                    }
                    Fun::Assoc(tb, x, _) => {
                        println!("Looking for: {}", tb);
                        let s = self.stack.impls.get(&Wrapper(tb)).unwrap().clone();
                        println!("Found: {}", s);
                        let s = s.defs.iter().find(|d| d.name == x).unwrap();
                        self.scoped(|ctx| {
                            for (p, v) in s.params.iter().zip(vs) {
                                ctx.stack.bind_val(p.name.clone(), v)
                            }
                            ctx.expr(&e)
                        })
                    }
                }
            }
            Expr::Block(_, _, b) => self.block(b),
            Expr::Query(_, _, _) => todo!(),
            Expr::Match(_, _, _e, _pes) => {
                todo!()
                // let mut iter = pes.iter();
                // let (Pat::Enum(_, _, _, _, x_then, p), e_then) = iter.next().unwrap() else {
                //     unreachable!();
                // };
                // let Pat::Var(_, _, x_var) = p.as_ref() else {
                //     unreachable!();
                // };
                // let (Pat::Wildcard(_, _), e_else) = iter.next().unwrap() else {
                //     unreachable!()
                // };
                // let Value::Variant(v) = self.expr(e) else {
                //     unreachable!();
                // };
                // if *x_then == v.name {
                //     let v = v.value.as_ref().clone();
                //     self.stack.bind(x_var.clone(), Binding::Var(v));
                //     self.expr(e_then)
                // } else {
                //     self.expr(e_else)
                // }
            }
            Expr::Array(_, _, _) => todo!(),
            Expr::Assign(_, _, _, _) => todo!(),
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::While(_, _, e0, b) => {
                while self.expr(e0).as_bool() {
                    self.block(b);
                }
                Value::from(Tuple::new(vec![]))
            }
            Expr::Fun(_, _, _, _, _) => unreachable!(),
            Expr::Err(_, _) => unreachable!(),
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => todo!(),
        }
    }

    fn ty(&mut self, t: &Type) -> Type {}
}
