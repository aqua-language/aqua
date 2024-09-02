use std::rc::Rc;

use smol_str::format_smolstr;

use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::ExprBody;
use crate::ast::StmtVar;
use crate::span::Span;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

#[derive(Debug)]
pub struct Context {
    stack: Vec<Scope>,
    uid_counter: u32,
}

#[derive(Debug)]
pub struct Scope {
    stmts: Vec<Stmt>,
    aliases: Map<Name, Name>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            stmts: vec![],
            aliases: Map::new(),
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack: vec![Scope::new()],
            uid_counter: 0,
        }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.stack.last_mut().unwrap().stmts.push(stmt);
    }

    pub fn rebind(&mut self, old: Name, new: Name) {
        self.stack.last_mut().unwrap().aliases.insert(old, new);
    }

    // Dealias the name `alias` to the most recent name it was bound to.
    // For example, if we have: var x = 1; var y = x; { var z = y; z + 1 },
    // then, dealias(z) gives x.
    pub fn dealias(&self, alias: Name) -> Name {
        let mut x = alias;
        let mut n = self.stack.len();
        // This loop searches for aliases by unwinding the stack of scopes.
        // If `x` is an alias, then we dealias it and keep searching from the current scope.
        while let Some((i, newer)) = self
            .stack
            .iter()
            .take(n)
            .enumerate()
            .rev()
            .find_map(|(i, s)| s.aliases.get(&x).map(|newer| (i, newer)))
        {
            n = i;
            x = *newer;
        }
        x
    }

    fn fresh_name(&mut self, s: Span) -> Name {
        let sym = format_smolstr!("x{}", self.uid_counter);
        self.uid_counter += 1;
        Name::new(s, sym)
    }

    pub fn flatten(&mut self, program: &Program) -> Program {
        self.visit_program(program);
        let stmts = std::mem::take(&mut self.stack.last_mut().unwrap().stmts);
        Program::new(program.span, stmts)
    }
}

impl Visitor for Context {
    fn visit_top_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.visit_stmt_var(s),
            Stmt::Expr(e) => {
                self.map_expr(e);
            }
            Stmt::Def(s) => self.visit_stmt_def(s),
            Stmt::Impl(s) => self.push_stmt(Stmt::Impl(s.clone())),
            Stmt::Trait(s) => self.push_stmt(Stmt::Trait(s.clone())),
            Stmt::Struct(s) => self.push_stmt(Stmt::Struct(s.clone())),
            Stmt::Enum(s) => self.push_stmt(Stmt::Enum(s.clone())),
            Stmt::Type(s) => self.push_stmt(Stmt::Type(s.clone())),
            Stmt::Err(_) => {}
        }
    }

    // `var x = 1; var y = x;` => `var x0 = 1;
    fn visit_stmt_var(&mut self, s: &StmtVar) {
        let Expr::Var(_, _, name) = self.map_expr(&s.expr) else {
            unreachable!();
        };
        self.rebind(s.name, name);
    }

    fn visit_stmt_def(&mut self, stmt: &StmtDef) {
        match &stmt.body {
            ExprBody::UserDefined(e) => {
                self.stack.push(Scope::new());
                let e = self.map_expr(e);
                let s = e.span_of();
                let t = e.type_of();
                let stmts = self.stack.pop().unwrap().stmts;
                let e = Expr::Block(s, t.clone(), Block::new(s, stmts, e));
                let mut stmt = stmt.clone();
                stmt.body = ExprBody::UserDefined(Rc::new(e));
                self.push_stmt(Stmt::Def(Rc::new(stmt)));
            }
            ExprBody::Builtin(_) => self.push_stmt(Stmt::Def(Rc::new(stmt.clone()))),
        }
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.push(Scope::new());
    }

    fn exit_scope(&mut self) {
        let stmts = self.stack.pop().unwrap().stmts;
        self.stack.last_mut().unwrap().stmts.extend(stmts);
    }

    fn map_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        self.visit_stmts(stmts);
        vec![]
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            // { 1 } => 1
            Expr::Block(_, _, b) => {
                let b = self.map_block(b);
                self.stack.last_mut().unwrap().stmts.extend(b.stmts);
                b.expr.as_ref().clone()
            }
            Expr::Var(_, _, x) => {
                let x = self.dealias(*x);
                Expr::Var(e.span_of(), e.type_of().clone(), x)
            }
            Expr::Tuple(_, _, es) if es.len() == 0 => e.clone(),
            _ => {
                let e = self._map_expr(e);
                let s = e.span_of();
                let t = e.type_of().clone();
                let x = self.fresh_name(e.span_of());

                self.stack
                    .last_mut()
                    .unwrap()
                    .stmts
                    .push(Stmt::Var(Rc::new(StmtVar::new(s, x, t.clone(), e))));

                Expr::Var(s, t, x)
            }
        }
    }
}
