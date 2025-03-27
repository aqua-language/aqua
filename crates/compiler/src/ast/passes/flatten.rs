use std::rc::Rc;

use smol_str::format_smolstr;

use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Stmt;
use crate::ast::StmtLocal;
use crate::ast::Type;
use crate::report::span::Span;
use crate::report::Report;
use crate::traversal::mapper::Mapper;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Vec<Stmt>>,
    uid_counter: u32,
    #[allow(dead_code)]
    pub report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack: vec![vec![]],
            uid_counter: 0,
            report: Report::new(),
        }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.stack.last_mut().unwrap().push(stmt);
    }

    fn fresh_name(&mut self, s: Span) -> Name {
        let data = format_smolstr!("x{}", self.uid_counter);
        self.uid_counter += 1;
        Name::new(s, data)
    }

    fn new_local(&mut self, s: Span, e: Expr, m: bool) -> Local {
        let x = self.fresh_name(s);
        let l = Local::new(s, x, Type::Unknown, m);
        self.stack
            .last_mut()
            .unwrap()
            .push(Stmt::Local(Rc::new(StmtLocal::new(s, l.clone(), Some(e)))));
        l
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            // &foo() => var x = foo(); &x
            Expr::Ref(s, t, e, m) if !e.is_place() => {
                let e = self.map_expr(e);
                let s1 = e.span();
                let l = self.new_local(s1, e, *m);
                let p = Expr::Place(*s, t.clone(), Place::new(*s, l.clone(), vec![]));
                Expr::Ref(*s, t.clone(), Rc::new(p), *m)
            }
            _ if e.is_place() => {
                let mut elems = vec![];
                let mut current = e;
                loop {
                    match current {
                        Expr::Field(s, t, e, x) => {
                            elems.push(PlaceElem::Field(*s, t.clone(), *x));
                            current = e;
                        }
                        Expr::Index(s, t, e, i) => {
                            elems.push(PlaceElem::Index(*s, t.clone(), *i));
                            current = e;
                        }
                        Expr::Deref(s, t, e) => {
                            elems.push(PlaceElem::Deref(*s, t.clone()));
                            current = e;
                        }
                        Expr::Local(s, t, x, m) => {
                            elems.reverse();
                            let l = Local::new(*s, x.clone(), t.clone(), *m);
                            let p = Place::new(*s, l.clone(), elems);
                            break Expr::Place(*s, e.ty().clone(), p);
                        }
                        _ => {
                            elems.reverse();
                            let s = current.span();
                            let t = current.ty().clone();
                            let l = self.new_local(s, current.clone(), false);
                            break Expr::Place(s, t, Place::new(s, l.clone(), elems));
                        }
                    }
                }
            }
            _ => self._map_expr(e),
        }
    }

    fn map_program(&mut self, p: &Ast) -> Ast {
        let mut stmts = vec![];
        for stmt in &p.stmts {
            let s = self.map_stmt(stmt);
            stmts.append(&mut self.stack.last_mut().unwrap());
            stmts.push(s);
        }
        Ast {
            span: p.span,
            stmts,
        }
    }

    fn map_block(&mut self, b: &Block) -> Block {
        self.stack.push(vec![]);
        let mut stmts = vec![];
        for stmt in &b.stmts {
            let s = self.map_stmt(stmt);
            stmts.append(&mut self.stack.last_mut().unwrap());
            stmts.push(s);
        }
        let expr = b.expr.as_ref().map(|e| self.map_expr(e));
        stmts.extend(self.stack.pop().unwrap());
        Block {
            span: b.span,
            stmts,
            expr,
        }
    }
}
