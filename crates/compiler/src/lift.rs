//! Lift defs to the top-level
//! * Assume defs can only capture defs, and not vars or generics.

use std::rc::Rc;

use runtime::HashMap;

use crate::ast::Block;
use crate::ast::Trait;
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
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

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

    pub fn lift(&mut self, program: &Program) -> Program {
        self.map_program(program)
    }
}

impl Visitor for Context {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.stack.bind(s.name);
            }
            Stmt::Trait(s) => {
                self.stack.bind(s.name);
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
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.scopes.push(Scope::new());
    }

    fn exit_scope(&mut self) {
        self.stack.scopes.pop();
    }

    fn map_program(&mut self, program: &Program) -> Program {
        self.visit_program(program);
        for stmt in &program.stmts {
            let stmt = self.map_stmt(stmt);
            self.top.push(stmt);
        }
        let stmts = std::mem::take(&mut self.top);
        Program::new(program.span, stmts)
    }

    fn map_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let ty = self.map_type(&s.ty);
        let expr = self.map_expr(&s.expr);
        let name = self.stack.get(&s.name);
        StmtVar::new(span, name, ty, expr)
    }

    fn map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        let name = self.stack.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let params = self.map_params(&s.params).into();
        let ty = self.map_type(&s.ty);
        let where_clause = self.map_bounds(&s.where_clause);
        let body = self.map_stmt_def_body(&s.body);
        StmtDef::new(s.span, name, generics, params, ty, where_clause, body)
    }

    fn map_param(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let name = self.stack.bind(*x);
        let ty = self.map_type(t);
        (name, ty)
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let generics = self.map_generics(&s.generics);
        let head = self.map_bound(&s.head);
        let where_clause = self.map_bounds(&s.where_clause);
        let defs = self.map_rc_iter(&s.defs, Self::_map_stmt_def).into();
        let types = self.map_rc_iter(&s.types, Self::_map_stmt_type).into();
        StmtImpl::new(s.span, generics, head, where_clause, defs, types)
    }

    fn map_bound(&mut self, b: &Trait) -> Trait {
        match b {
            Trait::Path(..) => unreachable!(),
            Trait::Cons(x, ts, xts) => {
                let ts = ts.iter().map(|t| self.map_type(t)).collect();
                let xts = xts.iter().map(|(n, t)| (*n, self.map_type(t))).collect();
                Trait::Cons(*x, ts, xts)
            }
            Trait::Type(t) => Trait::Type(Rc::new(self.map_type(t))),
            Trait::Err => Trait::Err,
            Trait::Var(_) => todo!(),
        }
    }

    fn map_stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let name = self.stack.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let fields = self.map_type_fields(&s.fields).into();
        StmtStruct::new(s.span, name, generics, fields)
    }

    fn map_stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let name = self.stack.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let variants = self.map_type_variants(&s.variants).into();
        StmtEnum::new(s.span, name, generics, variants)
    }

    fn map_stmt_type(&mut self, s: &StmtType) -> StmtType {
        let name = self.stack.get(&s.name);
        let generics = self.map_generics(&s.generics);
        let ty = self.map_stmt_type_body(&s.body);
        StmtType::new(s.span, name, generics, ty)
    }

    fn map_generic(&mut self, x: &Name) -> Name {
        self.stack.bind(*x)
    }

    fn map_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Cons(x, ts) => {
                let x = self.stack.get(x);
                let ts = self.map_types(ts);
                Type::Cons(x, ts)
            }
            Type::Alias(x, ts) => {
                let x = self.stack.get(x);
                let ts = ts.iter().map(|t| self.map_type(t)).collect();
                Type::Alias(x, ts)
            }
            Type::Generic(x) => {
                let x = self.stack.get(x);
                Type::Generic(x)
            }
            _ => self._map_type(ty),
        }
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Struct(s, t, x, ts, xes) => {
                let t = self.map_type(t);
                let x = self.stack.get(x);
                let ts = self.map_types(ts);
                let xes = self.map_expr_fields(xes).into();
                Expr::Struct(*s, t, x, ts, xes)
            }
            Expr::Enum(s, t, x0, ts, x1, e) => {
                let t = self.map_type(t);
                let x0 = self.stack.get(x0);
                let ts = self.map_types(ts);
                let e = self.map_expr(e);
                Expr::Enum(*s, t, x0, ts, *x1, Rc::new(e))
            }
            Expr::Var(s, t, x) => {
                let t = self.map_type(t);
                let x = self.stack.get(x);
                Expr::Var(*s, t, x)
            }
            Expr::Def(s, t, x, ts) => {
                let t = self.map_type(t);
                let x = self.stack.get(x);
                let ts = self.map_types(ts);
                Expr::Def(*s, t, x, ts)
            }
            _ => self._map_expr(e),
        }
    }

    fn map_block(&mut self, b: &Block) -> Block {
        self.visit_stmts(&b.stmts);
        let stmts = b
            .stmts
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Def(_)
                | Stmt::Trait(_)
                | Stmt::Impl(_)
                | Stmt::Struct(_)
                | Stmt::Enum(_)
                | Stmt::Type(_) => {
                    let stmt = self.map_stmt(stmt);
                    self.top.push(stmt);
                    None
                }
                _ => Some(self.map_stmt(stmt)),
            })
            .collect();
        let expr = self.map_expr(&b.expr);
        Block::new(b.span, stmts, expr)
    }
}
