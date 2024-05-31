use std::rc::Rc;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Map;
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
pub(crate) trait MappingVisitor {
    fn visit_program(&mut self, program: &Program) -> Program {
        let stmts = self.visit_iter(&program.stmts, Self::visit_stmt);
        Program::new(stmts)
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.visit_stmt_var(s))),
            Stmt::Def(s) => Stmt::Def(Rc::new(self.visit_stmt_def(s))),
            Stmt::Trait(s) => Stmt::Trait(Rc::new(self.visit_stmt_trait(s))),
            Stmt::Impl(s) => Stmt::Impl(Rc::new(self.visit_stmt_impl(s))),
            Stmt::Struct(s) => Stmt::Struct(Rc::new(self.visit_stmt_struct(s))),
            Stmt::Enum(s) => Stmt::Enum(Rc::new(self.visit_stmt_enum(s))),
            Stmt::Type(s) => Stmt::Type(Rc::new(self.visit_stmt_type(s))),
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.visit_expr(e))),
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
        let generics = self.visit_iter(&s.generics, Self::visit_generic);
        let params = self.visit_pairs(&s.params, Self::visit_name, Self::visit_type);
        let ty = self.visit_type(&s.ty);
        let where_clause = self.visit_iter(&s.where_clause, Self::visit_bound);
        let body = self.visit_stmt_def_body(&s.body);
        StmtDef::new(span, name, generics, params, ty, where_clause, body)
    }

    fn visit_generic(&mut self, g: &Name) -> Name {
        self.visit_name(g)
    }

    fn visit_stmt_def_body(&mut self, b: &StmtDefBody) -> StmtDefBody {
        match b {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(self.visit_expr(e)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }

    fn visit_bound(&mut self, b: &Bound) -> Bound {
        match b {
            Bound::Path(span, path) => {
                let span = self.visit_span(span);
                let path = self.visit_path(path);
                Bound::Path(span, path)
            }
            Bound::Trait(span, x, ts, xts) => {
                let span = self.visit_span(span);
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                let xts = self.visit_pairs(xts, Self::visit_name, Self::visit_type);
                Bound::Trait(span, x, ts, xts)
            }
            Bound::Type(span, t) => {
                let span = self.visit_span(span);
                let t = self.visit_type(t);
                Bound::Type(span, Rc::new(t))
            }
            Bound::Err(s) => Bound::Err(self.visit_span(s)),
        }
    }

    fn visit_stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = self.visit_iter(&s.generics, Self::visit_generic);
        let where_clause = self.visit_iter(&s.where_clause, Self::visit_bound);
        let defs = self.visit_rc_iter(&s.defs, Self::visit_trait_def);
        let types = self.visit_rc_iter(&s.types, Self::visit_trait_type);
        StmtTrait::new(span, name, generics, where_clause, defs, types)
    }

    fn visit_trait_def(&mut self, d: &StmtTraitDef) -> StmtTraitDef {
        let span = self.visit_span(&d.span);
        let name = self.visit_name(&d.name);
        let generics = self.visit_iter(&d.generics, Self::visit_generic);
        let params = self.visit_pairs(&d.params, Self::visit_name, Self::visit_type);
        let ty = self.visit_type(&d.ty);
        let where_clause = self.visit_iter(&d.where_clause, Self::visit_bound);
        StmtTraitDef::new(span, name, generics, params, ty, where_clause)
    }

    fn visit_trait_type(&mut self, t: &StmtTraitType) -> StmtTraitType {
        let span = self.visit_span(&t.span);
        let name = self.visit_name(&t.name);
        let generics = self.visit_iter(&t.generics, Self::visit_generic);
        StmtTraitType::new(span, name, generics)
    }

    fn visit_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let span = self.visit_span(&s.span);
        let generics = self.visit_iter(&s.generics, Self::visit_generic);
        let where_clause = self.visit_iter(&s.where_clause, Self::visit_bound);
        let head = self.visit_bound(&s.head);
        let defs = self.visit_rc_iter(&s.defs, Self::visit_stmt_def);
        let types = self.visit_rc_iter(&s.types, Self::visit_stmt_type);
        StmtImpl::new(span, generics, head, where_clause, defs, types)
    }

    fn visit_stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = self.visit_iter(&s.generics, Self::visit_generic);
        let fields = self.visit_pairs(&s.fields, Self::visit_name, Self::visit_type);
        StmtStruct::new(span, name, generics, fields)
    }

    fn visit_stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let span = self.visit_span(&s.span);
        let name = self.visit_name(&s.name);
        let generics = self.visit_iter(&s.generics, Self::visit_generic);
        let variants = self.visit_pairs(&s.variants, Self::visit_name, Self::visit_type);
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
        let span = self.visit_span(&expr.span_of());
        let ty = self.visit_type(&expr.type_of());
        match expr {
            Expr::Path(_, _, path) => {
                let path = self.visit_path(path);
                Expr::Path(span, ty, path)
            }
            Expr::Unresolved(_, _, x, ts) => {
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                Expr::Unresolved(span, ty, x, ts)
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
                let ts = self.visit_iter(ts, Self::visit_type);
                let xes = self.visit_pairs(xes, Self::visit_name, Self::visit_expr);
                Expr::Struct(span, ty, x, ts, xes)
            }
            Expr::Tuple(_, _, es) => {
                let es = self.visit_iter(es, Self::visit_expr);
                Expr::Tuple(span, ty, es)
            }
            Expr::Record(_, _, xes) => {
                let xes = self.visit_pairs(xes, Self::visit_name, Self::visit_expr);
                Expr::Record(span, ty, xes)
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                let x0 = self.visit_name(x0);
                let ts = self.visit_iter(ts, Self::visit_type);
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
                let ts = self.visit_iter(ts, Self::visit_type);
                Expr::Def(span, ty, x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = self.visit_expr(e);
                let es = self.visit_iter(es, Self::visit_expr);
                Expr::Call(span, ty, Rc::new(e), es)
            }
            Expr::Block(_, _, b) => {
                let b = self.visit_block(b);
                Expr::Block(span, ty, b)
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, b, x, ts) => {
                let b = self.visit_bound(b);
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                Expr::Assoc(span, ty, b, x, ts)
            }
            Expr::Match(_, _, _, _) => todo!(),
            Expr::Array(_, _, es) => {
                let es = self.visit_iter(es, Self::visit_expr);
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
                let ps = self.visit_pairs(ps, Self::visit_name, Self::visit_type);
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
        let segments = self.visit_iter(&path.segments, Self::visit_segment);
        Path::new(segments)
    }

    fn visit_segment(&mut self, seg: &Segment) -> Segment {
        let span = self.visit_span(&seg.span);
        let name = self.visit_name(&seg.name);
        let types = self.visit_iter(&seg.ts, Self::visit_type);
        let named_types = self.visit_pairs(&seg.xts, Self::visit_name, Self::visit_type);
        Segment::new(span, name, types, named_types)
    }

    fn visit_name(&mut self, name: &Name) -> Name {
        *name
    }

    fn visit_block(&mut self, b: &Block) -> Block {
        let span = self.visit_span(&b.span);
        let stmts = self.visit_iter(&b.stmts, Self::visit_stmt);
        let expr = self.visit_expr(&b.expr);
        Block::new(span, stmts, expr)
    }

    fn visit_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Path(path) => {
                let path = self.visit_path(path);
                Type::Path(path)
            }
            Type::Cons(x, ts) => {
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                Type::Cons(x, ts)
            }
            Type::Alias(x, ts) => {
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                Type::Alias(x, ts)
            }
            Type::Assoc(b, x, ts) => {
                let b = self.visit_bound(b);
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                Type::Assoc(b, x, ts)
            }
            Type::Var(x, k) => {
                let x = self.visit_name(x);
                Type::Var(x, *k)
            }
            Type::Generic(x) => {
                let x = self.visit_name(x);
                Type::Generic(x)
            }
            Type::Fun(ts, t) => {
                let ts = self.visit_iter(ts, Self::visit_type);
                let t = self.visit_type(t);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = self.visit_iter(ts, Self::visit_type);
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = self.visit_pairs(xts, Self::visit_name, Self::visit_type);
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
        let t = self.visit_type(&p.type_of());
        let s = self.visit_span(&p.span_of());
        match p {
            Pat::Path(_, _, path, ppfs) => {
                let path = self.visit_path(path);
                let ppfs = ppfs
                    .as_ref()
                    .map(|ppfs| self.visit_iter(ppfs, Self::visit_path_pat_field));
                Pat::Path(s, t, path, ppfs)
            }
            Pat::Var(_, _, x) => {
                let x = self.visit_name(x);
                Pat::Var(s, t, x)
            }
            Pat::Tuple(_, _, ps) => {
                let ps = self.visit_iter(ps, Self::visit_pattern);
                Pat::Tuple(s, t, ps)
            }
            Pat::Struct(_, _, x, ts, xps) => {
                let x = self.visit_name(x);
                let ts = self.visit_iter(ts, Self::visit_type);
                let xps = self.visit_pairs(xps, Self::visit_name, Self::visit_pattern);
                Pat::Struct(s, t, x, ts, xps)
            }
            Pat::Record(_, _, xps) => {
                let xps = self.visit_pairs(xps, Self::visit_name, Self::visit_pattern);
                Pat::Record(s, t, xps)
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                let x0 = self.visit_name(x0);
                let ts = self.visit_iter(ts, Self::visit_type);
                let x1 = self.visit_name(x1);
                let p = self.visit_pattern(p);
                Pat::Enum(s, t, x0, ts, x1, Rc::new(p))
            }
            Pat::Int(_, _, v) => Pat::Int(s, t, *v),
            Pat::String(_, _, v) => Pat::String(s, t, *v),
            Pat::Char(_, _, v) => Pat::Char(s, t, *v),
            Pat::Bool(_, _, v) => Pat::Bool(s, t, *v),
            Pat::Wildcard(_, _) => Pat::Wildcard(s, t),
            Pat::Or(_, _, p0, p1) => {
                let p0 = self.visit_pattern(p0);
                let p1 = self.visit_pattern(p1);
                Pat::Or(s, t, Rc::new(p0), Rc::new(p1))
            }
            Pat::Err(_, _) => Pat::Err(s, t),
        }
    }

    fn visit_path_pat_field(&mut self, pf: &PathPatField) -> PathPatField {
        match pf {
            PathPatField::Named(x, p) => {
                let x = self.visit_name(x);
                let p = self.visit_pattern(p);
                PathPatField::Named(x, p)
            }
            PathPatField::Unnamed(p) => {
                let p = self.visit_pattern(p);
                PathPatField::Unnamed(p)
            }
        }
    }

    fn visit_rc_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a Rc<T>>,
        f: impl Fn(&mut Self, &T) -> T,
    ) -> Vec<Rc<T>> {
        iter.into_iter().map(|x| Rc::new(f(self, &x))).collect()
    }

    fn visit_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a T>,
        f: impl Fn(&mut Self, &T) -> T,
    ) -> Vec<T> {
        iter.into_iter().map(|x| f(self, x)).collect()
    }

    fn visit_pairs<'a, K: 'a, V: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a (K, V)>,
        f: impl Fn(&mut Self, &K) -> K,
        g: impl Fn(&mut Self, &V) -> V,
    ) -> Map<K, V> {
        self.visit_iter(iter, |ctx, (k, v)| (f(ctx, k), g(ctx, v)))
            .into()
    }
}
