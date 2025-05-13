// TODO: Need to replace Places with Variables.
use std::rc::Rc;

use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::StmtLocal;
use crate::report::Report;
use crate::traversal::mapper::Mapper;

use super::Pass;

/// Converts `Expr::Lambda` functions into `Expr::Closure`, where the captured variables are explicitly passed.
#[derive(Debug)]
pub struct Context {
    uid: usize,
    stack: Vec<ClosureScope>,
    pub report: Report,
}

/// Scope of a closure.
#[derive(Debug)]
pub struct ClosureScope {
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
    /// Create a new context.
    pub fn new() -> Context {
        Self::default()
    }

    /// Bind a local variable in the current scope.
    fn bind(&mut self, l: Local) {
        if let Some(scope) = self.stack.last_mut() {
            scope.bindings.push(l);
        }
    }

    /// Enter a new scope.
    pub fn push_closure_scope(&mut self) {
        self.stack.push(ClosureScope {
            bindings: vec![],
            captures: vec![],
        });
    }

    /// Exit the current scope.
    pub fn pop_closure_scope(&mut self) -> Vec<Place> {
        self.stack.pop().unwrap().captures
    }

    /// Capture a place. The place is captured in all scopes until its variable is bound.
    fn capture(&mut self, p: Place) {
        for scope in self.stack.iter_mut().rev() {
            if scope.bindings.contains(&p.local) {
                break;
            }
            scope.captures.push(p.clone());
        }
    }

    /// Create a new unique identifier. Every closure gets a unique identifier.
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
                self.push_closure_scope();
                ls.iter().for_each(|l| self.bind(l.clone()));
                let ls = ls.iter().map(|l| self.map_local(l)).collect();
                let t1 = self.map_type(t1);
                let e = self.map_expr(e);
                self.exit_scope();
                let ps = self.pop_closure_scope();
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
