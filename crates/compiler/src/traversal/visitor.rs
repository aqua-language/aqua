#![allow(unused)]

use std::rc::Rc;

use crate::ast::Aggr;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::Program;
use crate::ast::Query;
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
use crate::ast::Trait;
use crate::ast::Type;
use crate::span::Span;

pub(crate) trait Visitor {
    fn visit_program(&mut self, program: &Program) {
        self._visit_program(program);
    }
    #[inline(always)]
    fn _visit_program(&mut self, program: &Program) {
        self.visit_top_stmts(&program.stmts);
    }

    fn visit_top_stmts(&mut self, stmts: &[Stmt]) {
        self.visit_iter(stmts, Self::visit_top_stmt);
    }

    fn visit_top_stmt(&mut self, s: &Stmt) {
        self.visit_stmt(s);
    }

    fn visit_stmts(&mut self, stmts: &[Stmt]) {
        self._visit_stmts(stmts);
    }
    #[inline(always)]
    fn _visit_stmts(&mut self, stmts: &[Stmt]) {
        self.visit_iter(stmts, Self::visit_stmt);
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        self._visit_stmt(s);
    }
    #[inline(always)]
    fn _visit_stmt(&mut self, s: &Stmt) {
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

    fn visit_span(&mut self, s: &Span) {
        self._visit_span(s);
    }
    #[inline(always)]
    fn _visit_span(&mut self, _: &Span) {}

    fn visit_stmt_var(&mut self, s: &StmtVar) {
        self._visit_stmt_var(s);
    }
    #[inline(always)]
    fn _visit_stmt_var(&mut self, s: &StmtVar) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_type(&s.ty);
        self.visit_expr(&s.expr);
    }

    fn visit_stmt_def(&mut self, s: &StmtDef) {
        self._visit_stmt_def(s);
    }
    #[inline(always)]
    fn _visit_stmt_def(&mut self, s: &StmtDef) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_generics(&s.generics);
        self.visit_params(&s.params);
        self.visit_type(&s.ty);
        self.visit_bounds(&s.where_clause);
        self.visit_stmt_def_body(&s.body);
    }

    #[inline(always)]
    fn visit_bounds(&mut self, bs: &[Trait]) {
        self._visit_bounds(bs);
    }
    #[inline(always)]
    fn _visit_bounds(&mut self, bs: &[Trait]) {
        self.visit_iter(bs, Self::visit_bound);
    }

    #[inline(always)]
    fn visit_params(&mut self, ps: &[(Name, Type)]) {
        self._visit_params(ps);
    }
    #[inline(always)]
    fn _visit_params(&mut self, ps: &[(Name, Type)]) {
        self.visit_iter(ps, Self::visit_param);
    }

    fn visit_param(&mut self, p: &(Name, Type)) {
        self._visit_param(p);
    }
    #[inline(always)]
    fn _visit_param(&mut self, p: &(Name, Type)) {
        self.visit_name(&p.0);
        self.visit_type(&p.1);
    }

    #[inline(always)]
    fn visit_generics(&mut self, generics: &[Name]) {
        self._visit_generics(generics);
    }
    #[inline(always)]
    fn _visit_generics(&mut self, generics: &[Name]) {
        self.visit_iter(generics, Self::visit_generic);
    }

    fn visit_generic(&mut self, g: &Name) {
        self._visit_generic(g);
    }
    #[inline(always)]
    fn _visit_generic(&mut self, g: &Name) {
        self.visit_name(g)
    }

    fn visit_stmt_def_body(&mut self, b: &StmtDefBody) {
        self._visit_stmt_def_body(b);
    }
    #[inline(always)]
    fn _visit_stmt_def_body(&mut self, b: &StmtDefBody) {
        match b {
            StmtDefBody::UserDefined(e) => self.visit_expr(e),
            StmtDefBody::Builtin(_) => {}
        }
    }

    fn visit_bound(&mut self, b: &Trait) {
        self._visit_bound(b);
    }
    #[inline(always)]
    fn _visit_bound(&mut self, b: &Trait) {
        match b {
            Trait::Path(span, path) => {
                self.visit_span(span);
                self.visit_path(path);
            }
            Trait::Cons(x, ts, xts) => {
                self.visit_name(x);
                self.visit_types(ts);
                self.visit_assoc_types(xts);
            }
            Trait::Type(t) => {
                self.visit_type(t);
            }
            Trait::Err => {}
            Trait::Var(_) => {}
        }
    }

    #[inline(always)]
    fn visit_assoc_types(&mut self, xts: &[(Name, Type)]) {
        self._visit_assoc_types(xts);
    }
    #[inline(always)]
    fn _visit_assoc_types(&mut self, xts: &[(Name, Type)]) {
        self.visit_iter(xts, Self::visit_assoc_type);
    }

    fn visit_assoc_type(&mut self, xt: &(Name, Type)) {
        self._visit_assoc_type(xt);
    }
    #[inline(always)]
    fn _visit_assoc_type(&mut self, xt: &(Name, Type)) {
        self.visit_name(&xt.0);
        self.visit_type(&xt.1);
    }

    fn visit_stmt_trait(&mut self, s: &StmtTrait) {
        self._visit_stmt_trait(s);
    }
    #[inline(always)]
    fn _visit_stmt_trait(&mut self, s: &StmtTrait) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_generics(&s.generics);
        self.visit_bounds(&s.where_clause);
        s.defs.iter().for_each(|d| self.visit_stmt_trait_def(d));
        s.types.iter().for_each(|t| self.visit_trait_type(t));
    }

    fn visit_stmt_trait_def(&mut self, d: &StmtTraitDef) {
        self._visit_stmt_trait_def(d);
    }
    #[inline(always)]
    fn _visit_stmt_trait_def(&mut self, d: &StmtTraitDef) {
        self.visit_span(&d.span);
        self.visit_name(&d.name);
        self.visit_generics(&d.generics);
        self.visit_trait_def_params(&d.params);
        self.visit_type(&d.ty);
        self.visit_iter(&d.where_clause, Self::visit_bound);
    }

    #[inline(always)]
    fn visit_trait_def_params(&mut self, ps: &[(Name, Type)]) {
        self._visit_trait_def_params(ps);
    }
    #[inline(always)]
    fn _visit_trait_def_params(&mut self, ps: &[(Name, Type)]) {
        self.visit_iter(ps, Self::visit_trait_def_param);
    }

    fn visit_trait_def_param(&mut self, p: &(Name, Type)) {
        self._visit_trait_def_param(p);
    }
    #[inline(always)]
    fn _visit_trait_def_param(&mut self, p: &(Name, Type)) {
        self.visit_name(&p.0);
        self.visit_type(&p.1);
    }

    fn visit_trait_type(&mut self, t: &StmtTraitType) {
        self._visit_trait_type(t);
    }
    #[inline(always)]
    fn _visit_trait_type(&mut self, t: &StmtTraitType) {
        self.visit_span(&t.span);
        self.visit_name(&t.name);
        self.visit_generics(&t.generics);
    }

    fn visit_stmt_impl(&mut self, s: &StmtImpl) {
        self._visit_stmt_impl(s);
    }
    #[inline(always)]
    fn _visit_stmt_impl(&mut self, s: &StmtImpl) {
        self.visit_span(&s.span);
        self.visit_generics(&s.generics);
        self.visit_bounds(&s.where_clause);
        self.visit_bound(&s.head);
        self.visit_rc_iter(&s.defs, Self::visit_stmt_impl_def);
        self.visit_rc_iter(&s.types, Self::visit_stmt_type);
    }

    fn visit_stmt_impl_def(&mut self, d: &StmtDef) {
        self._visit_stmt_def(d);
    }

    fn visit_stmt_impl_type(&mut self, s: &StmtType) {
        self._visit_stmt_type(s);
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

    fn visit_stmt_struct(&mut self, s: &StmtStruct) {
        self._visit_stmt_struct(s);
    }
    #[inline(always)]
    fn _visit_stmt_struct(&mut self, s: &StmtStruct) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_generics(&s.generics);
        self.visit_type_fields(&s.fields);
    }

    #[inline(always)]
    fn visit_type_fields(&mut self, xts: &[(Name, Type)]) {
        self._visit_type_fields(xts);
    }
    #[inline(always)]
    fn _visit_type_fields(&mut self, xts: &[(Name, Type)]) {
        self.visit_iter(xts, Self::visit_type_field);
    }

    fn visit_type_field(&mut self, xt: &(Name, Type)) {
        self._visit_type_field(xt);
    }
    #[inline(always)]
    fn _visit_type_field(&mut self, xt: &(Name, Type)) {
        self.visit_name(&xt.0);
        self.visit_type(&xt.1);
    }

    fn visit_stmt_enum(&mut self, s: &StmtEnum) {
        self._visit_stmt_enum(s);
    }
    #[inline(always)]
    fn _visit_stmt_enum(&mut self, s: &StmtEnum) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_generics(&s.generics);
        self.visit_type_variants(&s.variants);
    }

    #[inline(always)]
    fn visit_type_variants(&mut self, xts: &[(Name, Type)]) {
        self._visit_type_variants(xts);
    }
    #[inline(always)]
    fn _visit_type_variants(&mut self, xts: &[(Name, Type)]) {
        self.visit_iter(xts, Self::visit_type_variant);
    }

    fn visit_type_variant(&mut self, xt: &(Name, Type)) {
        self._visit_type_variant(xt);
    }
    #[inline(always)]
    fn _visit_type_variant(&mut self, xt: &(Name, Type)) {
        self.visit_name(&xt.0);
        self.visit_type(&xt.1);
    }

    fn visit_stmt_type(&mut self, s: &StmtType) {
        self._visit_stmt_type(s);
    }
    #[inline(always)]
    fn _visit_stmt_type(&mut self, s: &StmtType) {
        self.visit_span(&s.span);
        self.visit_name(&s.name);
        self.visit_generics(&s.generics);
        self.visit_stmt_type_body(&s.body);
    }

    fn visit_stmt_type_body(&mut self, b: &StmtTypeBody) {
        self._visit_stmt_type_body(b);
    }
    #[inline(always)]
    fn _visit_stmt_type_body(&mut self, b: &StmtTypeBody) {
        match b {
            StmtTypeBody::UserDefined(t) => self.visit_type(t),
            StmtTypeBody::Builtin(_) => {}
        }
    }

    #[inline(always)]
    fn visit_exprs(&mut self, es: &[Expr]) {
        self._visit_exprs(es);
    }
    #[inline(always)]
    fn _visit_exprs(&mut self, es: &[Expr]) {
        self.visit_iter(es, Self::visit_expr);
    }

    fn visit_expr(&mut self, e: &Expr) {
        self._visit_expr(e);
    }
    #[inline(always)]
    fn _visit_expr(&mut self, expr: &Expr) {
        self.visit_span(&expr.span_of());
        self.visit_type(&expr.type_of());
        match expr {
            Expr::Path(_, _, path) => {
                self.visit_path(path);
            }
            Expr::Unresolved(_, _, x, ts) => {
                self.visit_name(x);
                self.visit_types(ts);
            }
            Expr::Int(_, _, _v) => {}
            Expr::Float(_, _, _v) => {}
            Expr::Bool(_, _, _v) => {}
            Expr::String(_, _, _v) => {}
            Expr::Char(_, _, _v) => {}
            Expr::Struct(_, _, x, ts, xes) => {
                self.visit_name(x);
                self.visit_types(ts);
                self.visit_expr_fields(xes);
            }
            Expr::Tuple(_, _, es) => {
                self.visit_exprs(es);
            }
            Expr::Record(_, _, xes) => {
                self.visit_expr_fields(xes);
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                self.visit_name(x0);
                self.visit_types(ts);
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
                self.visit_types(ts);
            }
            Expr::Call(_, _, e, es) => {
                self.visit_expr(e);
                self.visit_exprs(es);
            }
            Expr::Block(_, _, b) => {
                self.visit_block(b);
            }
            Expr::Query(_, _, x, e, qs) => {
                self.visit_name(x);
                self.visit_expr(e);
                self.visit_query_stmts(qs);
            }
            Expr::QueryInto(_, _, x0, e, qs, x1, ts, es) => {
                self.visit_name(x0);
                self.visit_expr(e);
                self.visit_query_stmts(qs);
                self.visit_name(x1);
                self.visit_types(ts);
                self.visit_exprs(es);
            }
            Expr::TraitMethod(_, _, b, x, ts) => {
                self.visit_bound(b);
                self.visit_name(x);
                self.visit_types(ts);
            }
            Expr::Match(_, _, e, pes) => {
                self.visit_expr(e);
                self.visit_arms(pes);
            }
            Expr::Array(_, _, es) => {
                self.visit_exprs(es);
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
                self.visit_params(xts);
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
            Expr::InfixBinaryOp(_, _, _op, e0, e1) => {
                self.visit_expr(e0);
                self.visit_expr(e1);
            }
            Expr::PrefixUnaryOp(_, _, _op, e) => {
                self.visit_expr(e);
            }
            Expr::PostfixUnaryOp(_, _, _op, e) => {
                self.visit_expr(e);
            }
            Expr::Annotate(_, _, e) => {
                self.visit_expr(e);
            }
            Expr::Paren(_, _, e) => {
                self.visit_expr(e);
            }
            Expr::Dot(_, _, e, x, ts, es) => {
                self.visit_expr(e);
                self.visit_name(x);
                self.visit_types(ts);
                self.visit_exprs(es);
            }
            Expr::IfElse(_, _, e, b0, b1) => {
                self.visit_expr(e);
                self.visit_block(b0);
                self.visit_block(b1);
            }
            Expr::IntSuffix(_, _, _v, _x) => {}
            Expr::FloatSuffix(_, _, _v, _x) => {}
            Expr::LetIn(_, _, x, t1, e0, e1) => {
                self.visit_name(x);
                self.visit_type(t1);
                self.visit_expr(e0);
                self.visit_expr(e1);
            }
            Expr::Update(_, _, e0, x, e1) => {
                self.visit_expr(e0);
                self.visit_name(x);
                self.visit_expr(e1);
            }
        }
    }

    fn visit_arms(&mut self, pes: &[(Pat, Expr)]) {
        self._visit_arms(pes);
    }
    #[inline(always)]
    fn _visit_arms(&mut self, pes: &[(Pat, Expr)]) {
        self.visit_iter(pes, Self::visit_arm);
    }

    fn visit_arm(&mut self, pe: &(Pat, Expr)) {
        self._visit_arm(pe);
    }
    #[inline(always)]
    fn _visit_arm(&mut self, pe: &(Pat, Expr)) {
        self.visit_pattern(&pe.0);
        self.visit_expr(&pe.1);
    }

    fn visit_query_stmts(&mut self, qs: &[Query]) {
        self._visit_query_stmts(qs);
    }
    #[inline(always)]
    fn _visit_query_stmts(&mut self, qs: &[Query]) {
        self.visit_iter(qs, Self::visit_query_stmt);
    }

    fn visit_query_stmt(&mut self, q: &Query) {
        self._visit_query_stmt(q);
    }
    fn _visit_query_stmt(&mut self, q: &Query) {
        let s = self.visit_span(&q.span_of());
        match q {
            Query::From(_, x, e) => {
                self.visit_name(x);
                self.visit_expr(e);
            }
            Query::Var(_, x, e) => {
                self.visit_name(x);
                self.visit_expr(e);
            }
            Query::Where(_, e) => {
                self.visit_expr(e);
            }
            Query::Select(_, xes) => {
                self.visit_expr_fields(xes);
            }
            Query::OverCompute(_, e, aggs) => {
                self.visit_expr(e);
                self.visit_aggs(aggs);
            }
            Query::GroupOverCompute(_, x, e0, e1, aggs) => {
                self.visit_name(x);
                self.visit_expr(e0);
                self.visit_expr(e1);
                self.visit_aggs(aggs);
            }
            Query::JoinOn(_, x, e0, e1) => {
                self.visit_name(x);
                self.visit_expr(e0);
                self.visit_expr(e1);
            }
            Query::JoinOverOn(_, x, e0, e1, e2) => {
                self.visit_name(x);
                self.visit_expr(e0);
                self.visit_expr(e1);
                self.visit_expr(e2);
            }
            Query::Err(_) => {}
        }
    }

    fn visit_aggs(&mut self, aggs: &[Aggr]) {
        self._visit_aggs(aggs);
    }
    #[inline(always)]
    fn _visit_aggs(&mut self, aggs: &[Aggr]) {
        self.visit_iter(aggs, Self::visit_agg);
    }

    fn visit_agg(&mut self, agg: &Aggr) {
        self._visit_agg(agg);
    }
    fn _visit_agg(&mut self, agg: &Aggr) {
        self.visit_name(&agg.x);
        self.visit_expr(&agg.e0);
        self.visit_expr(&agg.e1);
    }

    #[inline(always)]
    fn visit_expr_fields(&mut self, xes: &[(Name, Expr)]) {
        self._visit_expr_fields(xes);
    }
    #[inline(always)]
    fn _visit_expr_fields(&mut self, xes: &[(Name, Expr)]) {
        self.visit_iter(xes, Self::visit_expr_field);
    }

    fn visit_expr_field(&mut self, xe: &(Name, Expr)) {
        self._visit_expr_field(xe);
    }
    #[inline(always)]
    fn _visit_expr_field(&mut self, xe: &(Name, Expr)) {
        self.visit_name(&xe.0);
        self.visit_expr(&xe.1);
    }

    fn visit_path(&mut self, path: &Path) {
        self._visit_path(path);
    }
    #[inline(always)]
    fn _visit_path(&mut self, path: &Path) {
        self.visit_iter(&path.segments, Self::visit_segment);
    }

    fn visit_segment(&mut self, seg: &Segment) {
        self._visit_segment(seg);
    }
    #[inline(always)]
    fn _visit_segment(&mut self, seg: &Segment) {
        self.visit_span(&seg.span);
        self.visit_name(&seg.name);
        self.visit_types(&seg.ts);
        self.visit_iter(&seg.xts, |ctx, xt| {
            ctx.visit_name(&xt.0);
            ctx.visit_type(&xt.1);
        });
    }

    fn visit_name(&mut self, name: &Name) {
        self._visit_name(name);
    }
    #[inline(always)]
    fn _visit_name(&mut self, name: &Name) {
        self.visit_span(&name.span);
    }

    fn visit_block(&mut self, b: &Block) {
        self._visit_block(b);
    }
    #[inline(always)]
    fn _visit_block(&mut self, b: &Block) {
        self.visit_span(&b.span);
        self.visit_stmts(&b.stmts);
        self.visit_expr(&b.expr);
    }

    #[inline(always)]
    fn visit_types(&mut self, ts: &[Type]) {
        self._visit_types(ts);
    }
    #[inline(always)]
    fn _visit_types(&mut self, ts: &[Type]) {
        self.visit_iter(ts, Self::visit_type);
    }

    fn visit_type(&mut self, t: &Type) {
        self._visit_type(t);
    }
    #[inline(always)]
    fn _visit_type(&mut self, t: &Type) {
        match t {
            Type::Path(path) => {
                self.visit_path(path);
            }
            Type::Cons(x, ts) => {
                self.visit_name(x);
                self.visit_types(ts);
            }
            Type::Alias(x, ts) => {
                self.visit_name(x);
                self.visit_types(ts);
            }
            Type::Assoc(b, x, ts) => {
                self.visit_bound(b);
                self.visit_name(x);
                self.visit_types(ts);
            }
            Type::Var(_) => {}
            Type::Generic(x) => {
                self.visit_name(x);
            }
            Type::Fun(ts, t) => {
                self.visit_types(ts);
                self.visit_type(t);
            }
            Type::Tuple(ts) => {
                self.visit_types(ts);
            }
            Type::Record(xts) => {
                self.visit_type_fields(xts);
            }
            Type::Array(t, _) => {
                self.visit_type(t);
            }
            Type::Never => {}
            Type::Unknown => {}
            Type::Err => {}
            Type::Paren(t) => {
                self.visit_type(t);
            }
        }
    }

    #[inline(always)]
    fn visit_patterns(&mut self, ps: &[Pat]) {
        self._visit_patterns(ps);
    }
    #[inline(always)]
    fn _visit_patterns(&mut self, ps: &[Pat]) {
        self.visit_iter(ps, Self::visit_pattern);
    }

    fn visit_pattern(&mut self, p: &Pat) {
        self._visit_pattern(p);
    }
    #[inline(always)]
    fn _visit_pattern(&mut self, p: &Pat) {
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
                self.visit_patterns(ts);
            }
            Pat::Struct(_, _, x, ts, xps) => {
                self.visit_name(x);
                self.visit_types(ts);
                self.visit_pattern_fields(xps);
            }
            Pat::Record(_, _, xps) => {
                self.visit_pattern_fields(xps);
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                self.visit_name(x0);
                self.visit_types(ts);
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
            Pat::Annotate(_, _, p) => {
                self.visit_pattern(p);
            }
            Pat::Paren(_, _, p) => {
                self.visit_pattern(p);
            }
        }
    }

    #[inline(always)]
    fn visit_pattern_fields(&mut self, xps: &[(Name, Pat)]) {
        self._visit_pattern_fields(xps);
    }
    #[inline(always)]
    fn _visit_pattern_fields(&mut self, xps: &[(Name, Pat)]) {
        self.visit_iter(xps, Self::visit_pattern_field);
    }

    fn visit_pattern_field(&mut self, xp: &(Name, Pat)) {
        self._visit_pattern_field(xp);
    }
    #[inline(always)]
    fn _visit_pattern_field(&mut self, xp: &(Name, Pat)) {
        self.visit_name(&xp.0);
        self.visit_pattern(&xp.1);
    }

    fn visit_path_pat_field(&mut self, pf: &PathPatField) {
        self._visit_path_pat_field(pf);
    }
    #[inline(always)]
    fn _visit_path_pat_field(&mut self, pf: &PathPatField) {
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

pub(crate) trait AcceptVisitor {
    fn visit(&self, visitor: impl Visitor);
}

impl AcceptVisitor for Program {
    fn visit(&self, mut visitor: impl Visitor) {
        visitor.visit_program(self);
    }
}

impl AcceptVisitor for Stmt {
    fn visit(&self, mut visitor: impl Visitor) {
        visitor.visit_stmt(self);
    }
}

impl AcceptVisitor for Expr {
    fn visit(&self, mut visitor: impl Visitor) {
        visitor.visit_expr(self);
    }
}

impl AcceptVisitor for Type {
    fn visit(&self, mut visitor: impl Visitor) {
        visitor.visit_type(self);
    }
}
