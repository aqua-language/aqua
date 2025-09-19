//! Lift functions, structs, enums, and traits to the top-level scope.
//! * Assume defs can only capture defs, and not vars or generics.

use std::rc::Rc;

use std::collections::HashMap;

use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::Type;
use crate::report::source::Cache;
use crate::report::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    unique: HashMap<Name, usize>,
    stack: Vec<Map<Name, Name>>,
    stmts: Vec<Stmt>,
    report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast, _: &mut Cache) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            unique: HashMap::default(),
            stack: vec![Map::new()],
            stmts: vec![],
            report: Report::new(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    fn bind(&mut self, old: Name) -> Name {
        let uid = *self
            .unique
            .entry(old)
            .and_modify(|uid| *uid += 1)
            .or_insert(0);
        let new = if uid == 0 { old } else { old.with_suffix(uid) };
        self.stack.last_mut().unwrap().insert(old, new);
        new
    }

    fn get(&self, x: &Name) -> Name {
        *self
            .stack
            .iter()
            .rev()
            .find_map(|scope| scope.get(x))
            .expect("Should be resolved")
    }
}

impl Visitor for Context {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Local(_) => {}
            Stmt::Def(s) => {
                self.bind(s.name);
            }
            Stmt::Trait(s) => {
                self.bind(s.name);
            }
            Stmt::Impl(_) => {}
            Stmt::Struct(s) => {
                self.bind(s.name);
            }
            Stmt::Enum(s) => {
                self.bind(s.name);
            }
            Stmt::Type(s) => {
                self.bind(s.name);
            }
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }
}

impl Mapper for Context {
    fn map_program(&mut self, program: &Ast) -> Ast {
        self.visit_program(program);
        for stmt in &program.stmts {
            let stmt = self.map_stmt(stmt);
            self.stmts.push(stmt);
        }
        let stmts = std::mem::take(&mut self.stmts);
        Ast::new(program.span, stmts)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        let name = self.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let params = self.map_locals(&s.params).into();
        let ty = self.map_type(&s.ty);
        let effect = s.effect.clone();
        let where_clause = self.map_impls(&s.where_clause);
        let body = self.map_stmt_def_body(&s.body);
        StmtDef::new(
            s.span,
            name,
            generics,
            params,
            ty,
            effect,
            where_clause,
            body,
        )
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let generics = self.map_generics(&s.generics);
        let head = self.map_impl(&s.head);
        let where_clause = self.map_impls(&s.where_clause);
        let defs = self.map_rc_iter(&s.defs, Self::_map_stmt_def).into();
        let types = self.map_rc_iter(&s.types, Self::_map_stmt_type).into();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let name = self.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let fields = self.map_type_fields(&s.fields).into();
        StmtStruct::new(s.span, name, generics, fields)
    }

    fn map_stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let name = self.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let variants = self.map_type_variants(&s.variants).into();
        StmtEnum::new(s.span, name, generics, variants)
    }

    fn map_stmt_type(&mut self, s: &StmtType) -> StmtType {
        let name = self.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let ty = self.map_stmt_type_body(&s.body);
        StmtType::new(s.span, name, generics, ty)
    }

    fn map_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Builtin(x, ts) => {
                let x = self.get(x);
                let ts = self.map_types(ts);
                Type::Builtin(x, ts)
            }
            Type::Struct(x, ts) => {
                let x = self.get(x);
                let ts = self.map_types(ts);
                Type::Struct(x, ts)
            }
            Type::Enum(x, ts) => {
                let x = self.get(x);
                let ts = self.map_types(ts);
                Type::Enum(x, ts)
            }
            _ => self._map_type(ty),
        }
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Struct(s, t, x, ts, xes) => {
                let t = self.map_type(t);
                let x = self.get(x);
                let ts = self.map_types(ts);
                let xes = self.map_expr_fields(xes).into();
                Expr::Struct(*s, t, x, ts, xes)
            }
            Expr::Enum(s, t, x0, ts, x1, e) => {
                let t = self.map_type(t);
                let x0 = self.get(x0);
                let ts = self.map_types(ts);
                let e = self.map_expr(e);
                Expr::Enum(*s, t, x0, ts, *x1, Rc::new(e))
            }
            Expr::Def(s, t, x, ts) => {
                let t = self.map_type(t);
                let x = self.get(x);
                let ts = self.map_types(ts);
                Expr::Def(*s, t, x, ts)
            }
            _ => self._map_expr(e),
        }
    }

    fn map_block(&mut self, b: &Block) -> Block {
        self.stack.push(Map::new());
        self.visit_stmts(&b.stmts);
        let stmts = b
            .stmts
            .iter()
            .filter_map(|stmt| {
                let stmt = self.map_stmt(stmt);
                if stmt.is_local() {
                    Some(stmt)
                } else {
                    self.stmts.push(stmt);
                    None
                }
            })
            .collect();
        let expr = b.expr.as_ref().map(|e| self.map_expr(e));
        self.stack.pop().unwrap();
        Block::new(b.span, stmts, expr)
    }
}
