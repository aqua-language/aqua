// TODO: Need to replace Places with Variables.

use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::StmtLocal;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    uid: usize,
    stack: Vec<Scope>,
    pub report: Report,
}

/// Scope of a lambda function.
#[derive(Debug)]
pub struct Scope {
    bindings: Vec<Local>,
    captures: Vec<Place>,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            uid: 0,
            stack: vec![],
            report: Report::new(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    fn bind(&mut self, l: Local) {
        if let Some(scope) = self.stack.last_mut() {
            scope.bindings.push(l);
        }
    }

    pub fn push_scope(&mut self) {
        self.stack.push(Scope {
            bindings: vec![],
            captures: vec![],
        });
    }

    pub fn pop_scope(&mut self) -> Vec<Place> {
        self.stack.pop().unwrap().captures
    }

    // Place is captured in all scopes until its variable is bound.
    fn capture(&mut self, p: Place) {
        for scope in self.stack.iter_mut().rev() {
            if scope.bindings.contains(&p.local) {
                break;
            }
            scope.captures.push(p.clone());
        }
    }

    fn new_uid(&mut self) -> usize {
        let uid = self.uid;
        self.uid += 1;
        uid
    }
}

impl Mapper for Context {
    fn map_stmt_local(&mut self, stmt: &StmtLocal) -> StmtLocal {
        self.bind(stmt.local.clone());
        self._map_stmt_local(stmt)
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Lambda(s, t, ls, t1, e) => {
                self.push_scope();
                ls.iter().for_each(|l| self.bind(l.clone()));
                let ls = ls.iter().map(|l| self.map_local(l)).collect();
                let t1 = self.map_type(t1);
                let e = self.map_expr(e);
                self.exit_scope();
                let ps = self.pop_scope();
                let uid = self.new_uid();
                Expr::Closure(*s, t.clone(), uid, ls, ps, t1, Rc::new(e))
            }
            Expr::Place(_, _, p) => {
                self.capture(p.clone());
                self._map_expr(e)
            }
            Expr::Call(s, t, e, es) if !e.is_def() => {
                let es = std::iter::once(e.as_ref())
                    .chain(es.iter())
                    .map(|e| self.map_expr(e))
                    .collect();
                let x_fn = Name::new(*s, "Call");
                let e_fn = Expr::Assoc(*s, t.clone(), Impl::Unknown, x_fn, vec![]);
                Expr::Call(*s, t.clone(), Rc::new(e_fn), es)
            }
            _ => self._map_expr(e),
        }
    }
}
