use std::rc::Rc;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::Program;
use crate::ast::Segment;
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
use crate::lexer::Span;

#[allow(unused)]
pub(crate) trait UnitVisitor {
    fn visit_program(&mut self, program: &Program) {
        program.stmts.iter().for_each(|s| self.visit_stmt(s));
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.visit_stmt_var(s),
            Stmt::Def(s) => self.visit_stmt_def(s),
            Stmt::Trait(s) => self.visit_stmt_trait(s),
            Stmt::Impl(s) => self.visit_stmt_impl(s),
            Stmt::Struct(s) => self.visit_stmt_struct(s),
            Stmt::Enum(s) => self.visit_stmt_enum(s),
            Stmt::Type(s) => self.visit_stmt_type(s),
            Stmt::Expr(e) => self.visit_expr(e),
            Stmt::Err(s) => self.visit_span(s),
        }
    }

    fn visit_span(&mut self, _: &Span) {}

    fn visit_stmt_var(&mut self, s: &StmtVar) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_type(&s.ty);
        self.visit_expr(&s.expr);
    }

    fn visit_stmt_def(&mut self, s: &StmtDef) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_iter(&s.generics, Self::visit_generic);
        self.visit_pairs(&s.params, Self::visit_name, Self::visit_type);
        self.visit_type(&s.ty);
        self.visit_iter(&s.where_clause, Self::visit_bound);
        self.visit_stmt_def_body(&s.body);
    }

    fn visit_generic(&mut self, g: &Name) {
        self.visit_name(g)
    }

    fn visit_stmt_def_body(&mut self, b: &StmtDefBody) {
        match b {
            StmtDefBody::UserDefined(e) => self.visit_expr(e),
            StmtDefBody::Builtin(_) => {}
        }
    }

    fn visit_bound(&mut self, b: &Bound) {
        match b {
            Bound::Path(span, path) => {
                self.visit_span(span);
                self.visit_path(path);
            }
            Bound::Trait(span, x, ts, xts) => {
                self.visit_span(span);
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
                self.visit_pairs(xts, Self::visit_name, Self::visit_type);
            }
            Bound::Type(span, t) => {
                self.visit_span(span);
                self.visit_type(t);
            }
            Bound::Err(s) => self.visit_span(s),
        }
    }

    fn visit_stmt_trait(&mut self, s: &StmtTrait) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        s.generics.iter().for_each(|g| self.visit_generic(g));
        self.visit_iter(&s.where_clause, Self::visit_bound);
        s.defs.iter().for_each(|d| self.visit_trait_def(d));
        s.types.iter().for_each(|t| self.visit_trait_type(t));
    }

    fn visit_trait_def(&mut self, d: &StmtTraitDef) {
        self.visit_span(&d.span);
        self.visit_name(&d.name);
        self.visit_iter(&d.generics, Self::visit_generic);
        self.visit_pairs(&d.params, Self::visit_name, Self::visit_type);
        self.visit_type(&d.ty);
        self.visit_iter(&d.where_clause, Self::visit_bound);
    }

    fn visit_trait_type(&mut self, t: &StmtTraitType) {
        self.visit_span(&t.span);
        self.visit_name(&t.name);
        self.visit_iter(&t.generics, Self::visit_generic);
    }

    fn visit_stmt_impl(&mut self, s: &StmtImpl) {
        self.visit_span(&s.span);
        self.visit_iter(&s.generics, Self::visit_generic);
        self.visit_iter(&s.where_clause, Self::visit_bound);
        self.visit_bound(&s.head);
        self.visit_rc_iter(&s.defs, Self::visit_stmt_def);
        self.visit_rc_iter(&s.types, Self::visit_stmt_type);
    }

    fn visit_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a T>,
        f: impl Fn(&mut Self, &T),
    ) {
        iter.into_iter().for_each(|x| f(self, x));
    }

    fn visit_rc_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a Rc<T>>,
        f: impl Fn(&mut Self, &T),
    ) {
        iter.into_iter().for_each(|x| f(self, x));
    }

    fn visit_pairs<'a, K: 'a, V: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a (K, V)>,
        f: impl Fn(&mut Self, &K),
        g: impl Fn(&mut Self, &V),
    ) {
        self.visit_iter(iter, |ctx, (k, v)| {
            f(ctx, k);
            g(ctx, v);
        });
    }

    fn visit_stmt_struct(&mut self, s: &StmtStruct) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_iter(&s.generics, Self::visit_generic);
        self.visit_pairs(&s.fields, Self::visit_name, Self::visit_type);
    }

    fn visit_stmt_enum(&mut self, s: &StmtEnum) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_iter(&s.generics, Self::visit_generic);
        self.visit_pairs(&s.variants, Self::visit_name, Self::visit_type);
    }

    fn visit_stmt_type(&mut self, s: &StmtType) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_iter(&s.generics, Self::visit_generic);
        self.visit_stmt_type_body(&s.body);
    }

    fn visit_stmt_type_body(&mut self, b: &StmtTypeBody) {
        match b {
            StmtTypeBody::UserDefined(t) => self.visit_type(t),
            StmtTypeBody::Builtin(_) => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        self.visit_span(&expr.span_of());
        self.visit_type(&expr.type_of());
        match expr {
            Expr::Path(_, _, path) => {
                self.visit_path(path);
            }
            Expr::Unresolved(_, _, x, ts) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Expr::Int(_, _, _v) => {}
            Expr::Float(_, _, _v) => {}
            Expr::Bool(_, _, _v) => {}
            Expr::String(_, _, _v) => {}
            Expr::Char(_, _, _v) => {}
            Expr::Struct(_, _, x, ts, xes) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
                self.visit_pairs(xes, Self::visit_name, Self::visit_expr);
            }
            Expr::Tuple(_, _, es) => {
                self.visit_iter(es, Self::visit_expr);
            }
            Expr::Record(_, _, xes) => {
                self.visit_pairs(xes, Self::visit_name, Self::visit_expr);
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                self.visit_name(x0);
                self.visit_iter(ts, Self::visit_type);
                self.visit_name(x1);
                self.visit_expr(e);
            }
            Expr::Field(_, _, e, x) => {
                self.visit_expr(e);
                self.visit_name(x);
            }
            Expr::Index(_, _, e, _i) => {
                self.visit_expr(e);
            }
            Expr::Var(_, _, x) => {
                self.visit_name(x);
            }
            Expr::Def(_, _, x, ts) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Expr::Call(_, _, e, es) => {
                self.visit_expr(e);
                self.visit_iter(es, Self::visit_expr);
            }
            Expr::Block(_, _, b) => {
                self.visit_block(b);
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, b, x, ts) => {
                self.visit_bound(b);
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::Array(_, _, es) => {
                self.visit_iter(es, Self::visit_expr);
            }
            Expr::Assign(_, _, e0, e1) => {
                self.visit_expr(e0);
                self.visit_expr(e1);
            }
            Expr::Return(_, _, e) => {
                self.visit_expr(e);
            }
            Expr::Continue(_, _) => {}
            Expr::Break(_, _) => {}
            Expr::While(_, _, e, b) => {
                self.visit_expr(e);
                self.visit_block(b);
            }
            Expr::Fun(_, _, xts, t, e) => {
                self.visit_pairs(xts, Self::visit_name, Self::visit_type);
                self.visit_type(t);
                self.visit_expr(e);
            }
            Expr::For(_, _, x, e, b) => {
                self.visit_name(x);
                self.visit_expr(e);
                self.visit_block(b);
            }
            Expr::Err(_, _) => {}
            Expr::Value(_, _) => {}
        }
    }

    fn visit_path(&mut self, path: &Path) {
        self.visit_iter(&path.segments, Self::visit_segment);
    }

    fn visit_segment(&mut self, seg: &Segment) {
        self.visit_span(&seg.span);
        self.visit_name(&seg.name);
        self.visit_iter(&seg.ts, Self::visit_type);
        self.visit_pairs(&seg.xts, Self::visit_name, Self::visit_type);
    }

    fn visit_name(&mut self, name: &Name) {
        self.visit_span(&name.span);
    }

    fn visit_block(&mut self, b: &Block) {
        self.visit_span(&b.span);
        self.visit_iter(&b.stmts, Self::visit_stmt);
        self.visit_expr(&b.expr);
    }

    fn visit_type(&mut self, t: &Type) {
        match t {
            Type::Path(path) => {
                self.visit_path(path);
            }
            Type::Cons(x, ts) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Type::Alias(x, ts) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Type::Assoc(b, x, ts) => {
                self.visit_bound(b);
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
            }
            Type::Var(x, _k) => {
                self.visit_name(x);
            }
            Type::Generic(x) => {
                self.visit_name(x);
            }
            Type::Fun(ts, t) => {
                self.visit_iter(ts, Self::visit_type);
                self.visit_type(t);
            }
            Type::Tuple(ts) => {
                self.visit_iter(ts, Self::visit_type);
            }
            Type::Record(xts) => {
                self.visit_pairs(xts, Self::visit_name, Self::visit_type);
            }
            Type::Array(t, _) => {
                self.visit_type(t);
            }
            Type::Never => {}
            Type::Hole => {}
            Type::Err => {}
        }
    }

    fn visit_pattern(&mut self, p: &Pat) {
        self.visit_span(&p.span_of());
        self.visit_type(&p.type_of());
        match p {
            Pat::Path(_, _, path, ppfs) => {
                self.visit_path(path);
                if let Some(ppfs) = ppfs {
                    self.visit_iter(ppfs, Self::visit_path_pat_field);
                }
            }
            Pat::Var(_, _, x) => {
                self.visit_name(x);
            }
            Pat::Tuple(_, _, ts) => {
                self.visit_iter(ts, Self::visit_pattern);
            }
            Pat::Struct(_, _, x, ts, xps) => {
                self.visit_name(x);
                self.visit_iter(ts, Self::visit_type);
                self.visit_pairs(xps, Self::visit_name, Self::visit_pattern);
            }
            Pat::Record(_, _, xps) => {
                self.visit_pairs(xps, Self::visit_name, Self::visit_pattern);
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                self.visit_name(x0);
                self.visit_iter(ts, Self::visit_type);
                self.visit_name(x1);
                self.visit_pattern(p);
            }
            Pat::Int(_, _, _v) => {}
            Pat::String(_, _, _v) => {}
            Pat::Char(_, _, _v) => {}
            Pat::Bool(_, _, _v) => {}
            Pat::Wildcard(_, _) => {}
            Pat::Or(_, _, p0, p1) => {
                self.visit_pattern(p0);
                self.visit_pattern(p1);
            }
            Pat::Err(_, _) => {}
        }
    }

    fn visit_path_pat_field(&mut self, pf: &PathPatField) {
        match pf {
            PathPatField::Named(x, p) => {
                self.visit_name(x);
                self.visit_pattern(p);
            }
            PathPatField::Unnamed(p) => {
                self.visit_pattern(p);
            }
        }
    }
}
