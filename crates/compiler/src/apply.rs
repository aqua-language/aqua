use std::rc::Rc;

use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Stmt;
use crate::ast::Trait;
use crate::ast::Type;
use crate::infer::float_default;
use crate::infer::int_default;
use crate::infer::type_var::TypeVarKind;
use crate::infer::type_var::TypeVarValue;
use crate::infer::Context;
use crate::infer::Goal;
use crate::lexer::Span;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

pub struct Instantiate<'a>(&'a Map<Name, Type>);

impl<'a> Instantiate<'a> {
    pub fn new(sub: &'a Map<Name, Type>) -> Instantiate<'a> {
        Instantiate(sub)
    }
}

impl<'a> Mapper for Instantiate<'a> {
    fn map_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Generic(x) => self.0.get(x).unwrap().clone(),
            _ => self._map_type(ty),
        }
    }
}

impl Type {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> Type {
        Instantiate::new(sub).map_type(self)
    }
    pub fn annotate(&self, ctx: &mut Context) -> Type {
        Annotate::new(ctx).map_type(self)
    }
}

impl Trait {
    pub fn instantiate(&self, sub: &Map<Name, Type>) -> Trait {
        Instantiate::new(sub).map_bound(self)
    }
}

#[derive(Debug)]
pub struct Defaults<'a>(&'a mut Context);

impl Defaults<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Defaults<'a> {
        Defaults(ctx)
    }
}

impl<'a> Visitor for Defaults<'a> {
    fn visit_type(&mut self, ty: &Type) {
        match ty {
            Type::Var(x) => {
                if let TypeVarValue::Unknown(k) = self.0.get_type(*x) {
                    match k {
                        TypeVarKind::Int => self.0.union_value(*x, int_default()),
                        TypeVarKind::Float => self.0.union_value(*x, float_default()),
                        _ => self._visit_type(ty),
                    }
                }
            }
            _ => self._visit_type(ty),
        }
    }
}

pub struct Annotate<'a>(&'a mut Context);

impl Annotate<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Annotate<'a> {
        Annotate(ctx)
    }
}

impl<'a> Mapper for Annotate<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        if let Type::Unknown = t {
            self.0.fresh(TypeVarKind::General)
        } else {
            self._map_type(t)
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Var(s) => Stmt::Var(Rc::new(self.map_stmt_var(s))),
            s => s.clone(),
        }
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Int(s, Type::Unknown, e) => {
                let t = self.0.fresh(TypeVarKind::Int);
                Expr::Int(*s, t, e.clone())
            }
            Expr::Float(s, Type::Unknown, e) => {
                let t = self.0.fresh(TypeVarKind::Float);
                Expr::Float(*s, t, e.clone())
            }
            _ => self._map_expr(e),
        }
    }

    fn map_pattern(&mut self, p: &Pat) -> Pat {
        if let Pat::Int(s, Type::Unknown, e) = p {
            let t = self.0.fresh(TypeVarKind::Int);
            Pat::Int(*s, t, e.clone())
        } else {
            self._map_pattern(p)
        }
    }
}

pub struct Apply<'a>(&'a mut Context);

impl Apply<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Apply<'a> {
        Apply(ctx)
    }
}

impl<'a> Mapper for Apply<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Var(x) => match self.0.get_type(*x) {
                TypeVarValue::Unknown(_) => t.clone(),
                TypeVarValue::Known(t) => self.map_type(&t),
            },
            _ => self._map_type(t),
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Var(v) => Stmt::Var(Rc::new(self.map_stmt_var(v))),
            s => s.clone(),
        }
    }
}

pub struct Canonicalize<'a>(&'a mut Context);

impl Canonicalize<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Canonicalize<'a> {
        Canonicalize(ctx)
    }
}

impl<'a> Mapper for Canonicalize<'a> {
    fn map_type(&mut self, t: &Type) -> Type {
        if let Type::Var(x) = t {
            Type::Var(self.0.type_scope().table.find(*x))
        } else {
            self._map_type(t)
        }
    }
}

pub struct GatherGoals<'a> {
    ctx: &'a mut Context,
    span: Span,
}

impl GatherGoals<'_> {
    pub fn new<'a>(ctx: &'a mut Context, span: Span) -> GatherGoals<'a> {
        GatherGoals { ctx, span }
    }
}

impl Visitor for GatherGoals<'_> {
    fn visit_type(&mut self, t: &Type) {
        if let Type::Assoc(b, _, _) = t {
            self.ctx.type_scope().goals.push(Goal::new(
                self.span,
                b.clone(),
            ));
        }
        self._visit_type(t)
    }

    fn visit_expr(&mut self, e: &Expr) {
        self.span = e.span_of();
        if let Expr::TraitMethod(s, _, b, _, _) = e {
            self.ctx.type_scope().goals.push(Goal::new(*s, b.clone()));
        }
        self._visit_expr(e)
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        self.span = s.span_of();
        match s {
            Stmt::Var(s) => self.visit_stmt_var(s),
            Stmt::Expr(s) => self.visit_expr(s),
            _ => {}
        }
    }
}

pub struct Expand;

impl Expand {
    pub fn new() -> Expand {
        Expand
    }
}

impl Mapper for Expand {
    fn map_type(&mut self, t0: &Type) -> Type {
        if let Type::Assoc(b, x, _) = t0 {
            if let Some(t1) = b.as_type(x) {
                t1.clone()
            } else {
                t0.clone()
            }
        } else {
            self._map_type(t0)
        }
    }

    fn map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.map_stmt_var(s))),
            Stmt::Expr(s) => Stmt::Expr(Rc::new(self.map_expr(s))),
            _ => s.clone(),
        }
    }
}
