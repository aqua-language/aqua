use std::rc::Rc;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::TraitBound;
use crate::ast::TraitDef;
use crate::ast::TraitType;
use crate::ast::Type;
use crate::lexer::Span;

pub(crate) trait Visitor {
    fn visit_program(&mut self, program: &Program) -> Program {
        program.stmts.iter().for_each(|s| self.pre_visit_stmt(s));
        let stmts = program.stmts.iter().map(|s| self.visit_stmt(s)).collect();
        Program::new(stmts)
    }

    fn pre_visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.pre_visit_stmt_var(s),
            Stmt::Def(s) => self.pre_visit_stmt_def(s),
            Stmt::Trait(s) => self.pre_visit_stmt_trait(s),
            Stmt::Impl(s) => self.pre_visit_stmt_impl(s),
            Stmt::Struct(s) => self.pre_visit_stmt_struct(s),
            Stmt::Enum(s) => self.pre_visit_stmt_enum(s),
            Stmt::Type(s) => self.pre_visit_stmt_type(s),
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }

    fn pre_visit_stmt_var(&mut self, _: &StmtVar) {}
    fn pre_visit_stmt_def(&mut self, _: &StmtDef) {}
    fn pre_visit_stmt_trait(&mut self, _: &StmtTrait) {}
    fn pre_visit_stmt_impl(&mut self, _: &StmtImpl) {}
    fn pre_visit_stmt_struct(&mut self, _: &StmtStruct) {}
    fn pre_visit_stmt_enum(&mut self, _: &StmtEnum) {}
    fn pre_visit_stmt_type(&mut self, _: &StmtType) {}

    fn visit_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(self.visit_stmt_var(s)),
            Stmt::Def(s) => Stmt::Def(self.visit_stmt_def(s)),
            Stmt::Trait(s) => Stmt::Trait(self.visit_stmt_trait(s)),
            Stmt::Impl(s) => Stmt::Impl(self.visit_stmt_impl(s)),
            Stmt::Struct(s) => Stmt::Struct(self.visit_stmt_struct(s)),
            Stmt::Enum(s) => Stmt::Enum(self.visit_stmt_enum(s)),
            Stmt::Type(s) => Stmt::Type(self.visit_stmt_type(s)),
            Stmt::Expr(e) => Stmt::Expr(self.visit_expr(e)),
            Stmt::Err(s) => Stmt::Err(self.visit_span(s)),
        }
    }

    fn visit_span(&mut self, s: &Span) -> Span {
        *s
    }

    fn visit_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let ty = self.visit_type(&s.ty);
        let expr = self.visit_expr(&s.expr);
        StmtVar::new(span, name, ty, expr)
    }

    fn visit_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let params = s.params.iter().map(|xt| self.visit_param(xt)).collect();
        let ty = self.visit_type(&s.ty);
        let where_clause = s.where_clause.iter().map(|b| self.visit_bound(b)).collect();
        let body = self.visit_stmt_def_body(&s.body);
        StmtDef::new(span, name, generics, params, ty, where_clause, body)
    }

    fn visit_generic(&mut self, g: &Name) -> Name {
        self.visit_name(g)
    }

    fn visit_param(&mut self, p: &Param) -> Param {
        let span = self.visit_span(&p.span);
        let name = self.visit_name(&p.name);
        let ty = self.visit_type(&p.ty);
        Param::new(span, name, ty)
    }

    fn visit_stmt_def_body(&mut self, b: &StmtDefBody) -> StmtDefBody {
        match b {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(self.visit_expr(e)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }

    fn visit_bound(&mut self, b: &Bound) -> Bound {
        match b {
            Bound::Unresolved(span, path) => {
                let span = self.visit_span(span);
                let path = self.visit_path(path);
                Bound::Unresolved(span, path)
            }
            Bound::Trait(span, bound) => {
                let span = self.visit_span(span);
                let bound = self.visit_trait_bound(bound);
                Bound::Trait(span, bound)
            }
            Bound::Err(s) => Bound::Err(self.visit_span(s)),
        }
    }

    fn visit_trait_bound(&mut self, b: &TraitBound) -> TraitBound {
        let name = self.visit_name(&b.name);
        let ts = b.ts.iter().map(|t| self.visit_type(t)).collect();
        let xts = b
            .xts
            .iter()
            .map(|(x, t)| (self.visit_name(x), self.visit_type(t)))
            .collect();
        TraitBound::new(name, ts, xts)
    }

    fn visit_stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let where_clause = s.where_clause.iter().map(|b| self.visit_bound(b)).collect();
        let defs = s.defs.iter().map(|d| self.visit_trait_def(d)).collect();
        let types = s.types.iter().map(|t| self.visit_trait_type(t)).collect();
        StmtTrait::new(span, name, generics, where_clause, defs, types)
    }

    fn visit_trait_def(&mut self, d: &TraitDef) -> TraitDef {
        let span = self.visit_span(&d.span);
        let name = self.visit_name(&d.name);
        let generics = d.generics.iter().map(|g| self.visit_generic(g)).collect();
        let params = d.params.iter().map(|xt| self.visit_param(xt)).collect();
        let ty = self.visit_type(&d.ty);
        let where_clause = d.where_clause.iter().map(|b| self.visit_bound(b)).collect();
        TraitDef::new(span, name, generics, params, ty, where_clause)
    }

    fn visit_trait_type(&mut self, t: &TraitType) -> TraitType {
        let span = self.visit_span(&t.span);
        let name = self.visit_name(&t.name);
        let generics = t.generics.iter().map(|g| self.visit_generic(g)).collect();
        TraitType::new(span, name, generics)
    }

    fn visit_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let span = self.visit_span(&s.span);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let where_clause = s.where_clause.iter().map(|b| self.visit_bound(b)).collect();
        let head = self.visit_bound(&s.head);
        let defs = s.defs.iter().map(|d| self.visit_stmt_def(d)).collect();
        let types = s.types.iter().map(|t| self.visit_stmt_type(t)).collect();
        StmtImpl::new(span, generics, head, where_clause, defs, types)
    }

    fn visit_stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let fields = s
            .fields
            .iter()
            .map(|(x, t)| (self.visit_name(x), self.visit_type(t)))
            .collect();
        StmtStruct::new(span, name, generics, fields)
    }

    fn visit_stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let variants = s
            .variants
            .iter()
            .map(|(x, t)| (self.visit_name(x), self.visit_type(t)))
            .collect();
        StmtEnum::new(span, name, generics, variants)
    }

    fn visit_stmt_type(&mut self, s: &StmtType) -> StmtType {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = s.generics.iter().map(|g| self.visit_generic(g)).collect();
        let body = self.visit_stmt_type_body(&s.body);
        StmtType::new(span, name, generics, body)
    }

    fn visit_stmt_type_body(&mut self, b: &StmtTypeBody) -> StmtTypeBody {
        match b {
            StmtTypeBody::UserDefined(t) => StmtTypeBody::UserDefined(self.visit_type(t)),
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Expr {
        let span = self.visit_span(&expr.span());
        let ty = self.visit_type(&expr.ty());
        match expr {
            Expr::Unresolved(_, _, path) => {
                let path = self.visit_path(path);
                Expr::Unresolved(span, ty, path)
            }
            Expr::Int(_, _, v) => {
                let v = v.clone();
                Expr::Int(span, ty, v)
            }
            Expr::Float(_, _, v) => {
                let v = v.clone();
                Expr::Float(span, ty, v)
            }
            Expr::Bool(_, _, v) => {
                let v = v.clone();
                Expr::Bool(span, ty, v)
            }
            Expr::String(_, _, v) => {
                let v = v.clone();
                Expr::String(span, ty, v)
            }
            Expr::Char(_, _, v) => {
                let v = v.clone();
                Expr::Char(span, ty, v)
            }
            Expr::Struct(_, _, x, ts, xes) => {
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                let xes = xes
                    .iter()
                    .map(|(x, e)| (self.visit_name(x), self.visit_expr(e)))
                    .collect();
                Expr::Struct(span, ty, x, ts, xes)
            }
            Expr::Tuple(_, _, es) => {
                let es = es.iter().map(|e| self.visit_expr(e)).collect();
                Expr::Tuple(span, ty, es)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes
                    .iter()
                    .map(|(x, e)| (self.visit_name(x), self.visit_expr(e)))
                    .collect();
                Expr::Record(span, ty, xes)
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                let x0 = self.visit_name(x0);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                let x1 = self.visit_name(x1);
                let e = self.visit_expr(e);
                Expr::Enum(span, ty, x0, ts, x1, Rc::new(e))
            }
            Expr::Field(_, _, e, x) => {
                let e = self.visit_expr(e);
                let x = self.visit_name(x);
                Expr::Field(span, ty, Rc::new(e), x)
            }
            Expr::Index(_, _, e, i) => {
                let e = self.visit_expr(e);
                let i = *i;
                Expr::Index(span, ty, Rc::new(e), i)
            }
            Expr::Var(_, _, x) => {
                let x = self.visit_name(x);
                Expr::Var(span, ty, x)
            }
            Expr::Def(_, _, x, ts) => {
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Expr::Def(span, ty, x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = self.visit_expr(e);
                let es = es.iter().map(|e| self.visit_expr(e)).collect();
                Expr::Call(span, ty, Rc::new(e), es)
            }
            Expr::Block(_, _, b) => {
                let b = self.visit_block(b);
                Expr::Block(span, ty, b)
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, tb, x, ts) => {
                let tb = self.visit_trait_bound(tb);
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Expr::Assoc(span, ty, tb, x, ts)
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::Array(_, _, es) => {
                let es = es.iter().map(|e| self.visit_expr(e)).collect();
                Expr::Array(span, ty, es)
            }
            Expr::Assign(_, _, e0, e1) => {
                let e0 = self.visit_expr(e0);
                let e1 = self.visit_expr(e1);
                Expr::Assign(span, ty, Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, e) => {
                let e = self.visit_expr(e);
                Expr::Return(span, ty, Rc::new(e))
            }
            Expr::Continue(_, _) => Expr::Continue(span, ty),
            Expr::Break(_, _) => Expr::Break(span, ty),
            Expr::While(_, _, e, b) => {
                let e = self.visit_expr(e);
                let b = self.visit_block(b);
                Expr::While(span, ty, Rc::new(e), b)
            }
            Expr::Fun(_, _, ps, t, e) => {
                let ps = ps.iter().map(|p| self.visit_param(p)).collect();
                let t = self.visit_type(t);
                let e = self.visit_expr(e);
                Expr::Fun(span, ty, ps, t, Rc::new(e))
            }
            Expr::For(_, _, x, e, b) => {
                let x = self.visit_name(x);
                let e = self.visit_expr(e);
                let b = self.visit_block(b);
                Expr::For(span, ty, x, Rc::new(e), b)
            }
            Expr::Err(_, _) => Expr::Err(span, ty),
            Expr::Value(_, _) => todo!(),
        }
    }

    fn visit_path(&mut self, path: &Path) -> Path {
        let segments = path
            .segments
            .iter()
            .map(|seg| self.visit_segment(seg))
            .collect();
        Path::new(segments)
    }

    fn visit_segment(&mut self, seg: &Segment) -> Segment {
        let span = seg.span;
        let name = seg.name.clone();
        let types = seg.types.iter().map(|t| self.visit_type(t)).collect();
        let named_types = seg
            .named_types
            .iter()
            .map(|(x, t)| (self.visit_name(x), self.visit_type(t)))
            .collect();
        Segment::new(span, name, types, named_types)
    }

    fn visit_name(&mut self, name: &Name) -> Name {
        name.clone()
    }

    fn visit_block(&mut self, b: &Block) -> Block {
        let span = self.visit_span(&b.span);
        b.stmts.iter().for_each(|s| self.pre_visit_stmt(s));
        let stmts = b.stmts.iter().map(|s| self.visit_stmt(s)).collect();
        let expr = self.visit_expr(&b.expr);
        Block::new(span, stmts, expr)
    }

    fn visit_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Unresolved(path) => {
                let path = self.visit_path(path);
                Type::Unresolved(path)
            }
            Type::Cons(x, ts) => {
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Type::Cons(x, ts)
            }
            Type::Alias(x, ts) => {
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Type::Alias(x, ts)
            }
            Type::Assoc(tb, x, ts) => {
                let tb = self.visit_trait_bound(tb);
                let x = self.visit_name(x);
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Type::Assoc(tb, x, ts)
            }
            Type::Var(x) => {
                let x = self.visit_name(x);
                Type::Var(x)
            }
            Type::Generic(x) => {
                let x = self.visit_name(x);
                Type::Generic(x)
            }
            Type::Fun(ts, t) => {
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                let t = self.visit_type(t);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| self.visit_type(t)).collect();
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts
                    .iter()
                    .map(|(x, t)| (self.visit_name(x), self.visit_type(t)))
                    .collect();
                Type::Record(xts)
            }
            Type::Array(t, i) => {
                let t = self.visit_type(t);
                let i = *i;
                Type::Array(Rc::new(t), i)
            }
            Type::Never => Type::Never,
            Type::Hole => Type::Hole,
            Type::Err => Type::Err,
        }
    }

    fn visit_pattern(&mut self, p: &Pat) -> Pat {
        match p {
            Pat::Unresolved(_, _, _, _) => todo!(),
            Pat::Var(_, _, _) => todo!(),
            Pat::Tuple(_, _, _) => todo!(),
            Pat::Struct(_, _, _, _, _) => todo!(),
            Pat::Record(_, _, _) => todo!(),
            Pat::Enum(_, _, _, _, _, _) => todo!(),
            Pat::Int(_, _, _) => todo!(),
            Pat::String(_, _, _) => todo!(),
            Pat::Char(_, _, _) => todo!(),
            Pat::Bool(_, _, _) => todo!(),
            Pat::Wildcard(_, _) => todo!(),
            Pat::Or(_, _, _, _) => todo!(),
            Pat::Err(_, _) => todo!(),
        }
    }
}
