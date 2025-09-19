// TODO: Need to replace Places with Variables.
use std::rc::Rc;

use crate::analysis::declare;
use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Index;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtImpl;
use crate::ast::StmtLocal;
use crate::ast::StmtStruct;
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
    stmts: Vec<Stmt>, // Struct statements that are generated
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
        let mut program = self.map_program(program);
        program.stmts.append(&mut self.stmts);
        program
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
            stmts: vec![],
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

    fn push_closure_stmts(
        &mut self,
        span: Span,
        captures: &[Place],
        params: &[Local],
        expr: Expr,
    ) -> Name {
        // Generate a new name
        let uid = self.uid;
        self.uid += 1;
        let name = Name::from(Symbol::from(smol_str::format_smolstr!("C{uid}")));

        // Create a struct statement
        //
        // struct <id>(c: ?)
        let type_fields = captures
            .iter()
            .map(|p| (p.local.name, Type::Unknown))
            .collect();
        let stmt = StmtStruct::new(span, name, vec![], type_fields);
        self.stmts.push(Stmt::Struct(Rc::new(stmt)));

        // Create an impl statement
        //
        // Fn[<id>, (?, ?), ?]
        let impl_trait = Impl::Trait(Trait::new(
            "Fn".into(),
            vec![
                Type::Struct(name.clone(), vec![]),
                Type::Tuple(vec![Type::Unknown; params.len()]),
                Type::Unknown,
            ],
        ));
        // Create locals for the environment and arguments
        // env: <id>, args: (?, ?)
        let env_local = Local::new(
            span,
            Name::new(span, "env"),
            Type::Struct(name.clone(), vec![]),
            false,
        );
        let args_local = Local::new(
            span,
            Name::new(span, "args"),
            Type::Tuple(vec![Type::Unknown; params.len()]),
            false,
        );
        // Create statements for extracting captures
        //
        // val c = env.c;
        let mut stmts = vec![];
        stmts.extend(captures.iter().map(|p| {
            let field = PlaceElem::Field(p.span, Type::Unknown, p.local.name.clone());
            let place = Place::new(p.span, env_local.clone(), vec![field]);
            let expr = Expr::Place(p.span, Type::Unknown, place.clone());
            Stmt::Local(Rc::new(StmtLocal::new(
                place.span,
                p.local.clone(),
                Some(expr),
            )))
        }));
        // Create statements for extracting arguments
        //
        // val a = args.0;
        // val b = args.1;
        stmts.extend(params.iter().enumerate().map(|(i, l)| {
            let index = PlaceElem::Index(l.span, Type::Unknown, Index::new(l.span, i));
            let place = Place::new(l.span, args_local.clone(), vec![index]);
            let expr = Expr::Place(l.span, Type::Unknown, place.clone());
            Stmt::Local(Rc::new(StmtLocal::new(place.span, l.clone(), Some(expr))))
        }));
        // Create the function body
        //
        // { val c = env.c; val a = args.0; val b = args.1; a + b + c }
        let block = Block::new(span, stmts, Some(expr));
        // Create the trait impl
        //
        // impl Fn[<id>, (?, ?), ?] {
        //    def call(env: <id>, args: (?, ?)): ? = {
        //        val a = args.0;
        //        val b = args.1;
        //        val c = env.c;
        //        a + b + c
        //    }
        // }
        let impl_def = StmtDef::new_simple(
            span,
            "call".into(),
            vec![env_local, args_local],
            Type::Unknown,
            Expr::Block(span, Type::Unknown, Rc::new(block)),
        );
        let impl_stmt = StmtImpl::new_simple(span, impl_trait, vec![Rc::new(impl_def)]);
        self.stmts.push(Stmt::Impl(Rc::new(impl_stmt)));
        name
    }

    fn create_closure_expr(&mut self, span: Span, name: Name, captures: &[Place]) -> Expr {
        // Create a struct expression
        //
        // <id>(c: ?)
        let fields = captures
            .iter()
            .map(|p| {
                let place = Place::new(p.span, p.local.clone(), vec![]);
                (p.local.name, Expr::Place(p.span, Type::Unknown, place))
            })
            .collect();
        Expr::Struct(span, Type::Unknown, name, vec![], fields)
    }
}

impl Mapper for Context {
    fn map_stmt_local(&mut self, stmt: &StmtLocal) -> StmtLocal {
        self.bind(stmt.local.clone());
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
            Expr::Lambda(s, _t, params, t1, e) => {
                self.push_closure_scope();
                params.iter().for_each(|l| self.bind(l.clone()));
                let params: Vec<Local> = params.iter().map(|l| self.map_local(l)).collect();
                let _t1 = self.map_type(t1);
                let e = self.map_expr(e);
                self.exit_scope();
                let captures = self.pop_closure_scope();
                let name = self.push_closure_stmts(e.span(), &captures, &params, e);
                self.create_closure_expr(*s, name, &captures)
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
            // def f(a: ?, b: ?) = a + b;
            // val g = f;
            //
            // Becomes:
            //
            // struct <id>;
            // impl Fn<(?, ?), ?> for <id> {
            //     def call(env: <id>, args: (?, ?)): ? = {
            //         val a = args.0;
            //         val b = args.1;
            //         val f = env.f;
            //         f(a, b)
            //     }
            // };
            Expr::Def(s, t, x, _) => {
                // Lower the closure
                let def = self.decls.defs.get(x).unwrap().clone();
                let locals = def
                    .params
                    .iter()
                    .map(|l| Expr::Place(*s, Type::Unknown, Place::new(*s, l.clone(), vec![])))
                    .collect();
                let expr = Expr::Call(*s, t.clone(), Rc::new(expr.clone()), locals);
                let name = self.push_closure_stmts(*s, &[], &def.params, expr);
                self.create_closure_expr(*s, name, &[])
            }
            _ => self._map_expr(expr),
        }
    }
}
