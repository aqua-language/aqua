// TODO: Need to replace Places with Variables.
// TODO: After inferring, we need to find the generic types that the closure captures
//       and insert them in the struct/impl Fn.
use std::rc::Rc;

use crate::analysis::declare;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::StmtLocal;
use crate::ast::Trait;
use crate::ast::Type;
use crate::report::source::Cache;
use crate::report::span::Span;
use crate::report::symbol::Symbol;
use crate::report::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor as _;

use super::Pass;

/// Converts `Expr::Lambda` functions into `Expr::Closure`, where the captured variables are explicitly passed.
#[derive(Debug)]
pub struct Context {
    uid: u32,
    decls: declare::Context,
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
    fn run(&mut self, program: &Ast, _: &mut Cache) -> Ast {
        self.decls.visit_program(program);
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
            decls: declare::Context::default(),
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
    fn bind(&mut self, l: &Local) {
        if let Some(scope) = self.stack.last_mut() {
            scope.bindings.push(l.clone());
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

    fn new_name(&mut self, span: Span) -> Name {
        let uid = self.uid;
        self.uid += 1;
        Name::new(span, Symbol::from(smol_str::format_smolstr!("C{uid}")))
    }
}

impl Mapper for Context {
    fn map_stmt_local(&mut self, stmt: &StmtLocal) -> StmtLocal {
        self.bind(&stmt.local);
        self._map_stmt_local(stmt)
    }

    fn map_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            // val c: ? = 3;
            // val f = (a: ?, b: ?) => a + b + c;
            //
            // Becomes:
            //
            // struct <id>(c: ?)
            // impl Fn[<id>, (?, ?), ?] {
            //     def call(env: <id>, args: (?, ?)): ? = {
            //         val a = args.0;
            //         val b = args.1;
            //         val c = env.c;
            //         a + b + c
            //     };
            // }
            // val c: ? = 3;
            // val f = <id>(c: c);
            Expr::Lambda(s, t, params, t1, e) => {
                self.push_closure_scope();
                let t = self.map_type(t);
                params.iter().for_each(|l| self.bind(l));
                let params: Vec<Local> = params.iter().map(|l| self.map_local(l)).collect();
                let t1 = self.map_type(t1);
                let e = self.map_expr(e);
                let captures = self.pop_closure_scope();
                let name = self.new_name(*s);
                Expr::Closure(*s, t, name, params, captures, t1, Rc::new(e))
            }
            // Capture variable
            Expr::Place(_, _, p) => {
                self.capture(p.clone());
                self._map_expr(expr)
            }
            // Transform an indirect call to an Fn trait call:
            //
            // f(1, 2);
            //
            // becomes
            //
            // Fn[_, _, _]::call(f, (1, 2));
            Expr::Call(s, t, e, es) if !e.is_def() => {
                // Replace call with Fn::call
                let es: Vec<Expr> = std::iter::once(e.as_ref())
                    .chain(es.iter())
                    .map(|e| self.map_expr(e))
                    .collect();
                let x_fn = Name::new(*s, "call");
                let ty_tuple = Type::Tuple(vec![Type::Unknown; es.len()]);
                let imp = Impl::Trait(Trait::new("Fn".into(), vec![ty_tuple]));
                let expr_fn = Expr::Assoc(*s, t.clone(), imp, x_fn, vec![]);
                Expr::Call(*s, t.clone(), Rc::new(expr_fn), es)
            }
            // def f(a: ?, b: ?) = a + b;
            // f(1, 2);
            //
            // Should not be converted to a closure, since it is a direct call.
            Expr::Call(s, t, e, es) if e.is_def() => Expr::Call(
                *s,
                t.clone(),
                e.clone(),
                es.iter().map(|e| self.map_expr(e)).collect(),
            ),
            // def f(a: i32, b: i32) -> i32 { a + b }
            // let x = f;
            // x(1, 2);
            //
            // Becomes:
            //
            // def f(a: i32, b: i32) -> i32 { a + b }
            // let x = move |a, b| f(a, b);
            // x(1, 2)
            Expr::Def(s, t, x, ts) => {
                // Lower the closure
                let t = self.map_type(t);
                let def = self.decls.defs.get(x).unwrap().clone();
                let args = def
                    .params
                    .iter()
                    .map(|l| Expr::Place(*s, Type::Unknown, Place::new(*s, l.clone(), vec![])))
                    .collect();
                let ts = self.map_types(ts);
                let def_expr = Expr::Def(*s, t.clone(), x.clone(), ts);
                let call_expr = Expr::Call(*s, t.clone(), Rc::new(def_expr), args);
                let name = self.new_name(*s);
                Expr::Closure(
                    *s,
                    t,
                    name,
                    def.params.clone(),
                    Vec::new(),
                    def.ty.clone(),
                    Rc::new(call_expr),
                )
            }
            _ => self._map_expr(expr),
        }
    }
}
