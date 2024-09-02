#![allow(unused)]

use std::rc::Rc;

use runtime::prelude::Aggregator;
use runtime::prelude::Assigner;
use runtime::prelude::Encoding;
use runtime::prelude::Set;

use crate::ast::Aggr;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::Program;
use crate::ast::Query;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::builtins::value::Dataflow;
use crate::builtins::value::Fun;
use crate::builtins::value::Record;
use crate::builtins::value::Stream;
use crate::builtins::value::Tuple;
use crate::builtins::value::Value;
use crate::builtins::value::Variant;
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

    fn visit_stmt_def_body(&mut self, b: &ExprBody) {
        self._visit_stmt_def_body(b);
    }
    #[inline(always)]
    fn _visit_stmt_def_body(&mut self, b: &ExprBody) {
        match b {
            ExprBody::UserDefined(e) => self.visit_expr(e),
            ExprBody::Builtin(_) => {}
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

    fn visit_stmt_type_body(&mut self, b: &TypeBody) {
        self._visit_stmt_type_body(b);
    }
    #[inline(always)]
    fn _visit_stmt_type_body(&mut self, b: &TypeBody) {
        match b {
            TypeBody::UserDefined(t) => self.visit_type(t),
            TypeBody::Builtin(_) => {}
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
            Expr::Query(_, _, x0, t0, e, qs) => {
                self.visit_name(x0);
                self.visit_type(t0);
                self.visit_expr(e);
                self.visit_query_stmts(qs);
            }
            Expr::QueryInto(_, _, x0, t0, e, qs, x1, ts, es) => {
                self.visit_name(x0);
                self.visit_type(t0);
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
            Expr::Value(_, v) => {
                self.visit_value(v);
            }
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
            Expr::Anonymous(_, _) => {}
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

    fn visit_value(&mut self, v: &Value) {
        self._visit_value(v);
    }

    fn _visit_value(&mut self, v: &Value) {
        match v {
            Value::Aggregator(v) => self._visit_value_aggregator(v),
            Value::Array(v) => self._visit_value_array(v),
            Value::Blob(_) => {}
            Value::Bool(_) => {}
            Value::Char(_) => {}
            Value::Dict(v) => self._visit_value_dict(v),
            Value::Assigner(_) => {}
            Value::Duration(_) => {}
            Value::Encoding(_) => {}
            Value::F32(_) => {}
            Value::F64(_) => {}
            Value::File(_) => {}
            Value::Fun(v) => self._visit_value_fun(v),
            Value::I128(_) => {}
            Value::I16(_) => {}
            Value::I32(_) => {}
            Value::I64(_) => {}
            Value::I8(_) => {}
            Value::Option(v) => self._visit_value_option(v),
            Value::Path(_) => {}
            Value::Reader(_) => {}
            Value::Record(v) => self._visit_value_record(v),
            Value::Result(v) => self._visit_value_result(v),
            Value::Set(v) => self._visit_value_set(v),
            Value::SocketAddr(_) => {}
            Value::Stream(v) => self._visit_value_stream(v),
            Value::Dataflow(v) => self._visit_value_dataflow(v),
            Value::String(_) => {}
            Value::Time(_) => {}
            Value::TimeSource(_) => {}
            Value::Tuple(v) => self._visit_value_tuple(v),
            Value::U128(_) => {}
            Value::U16(_) => {}
            Value::U32(_) => {}
            Value::U64(_) => {}
            Value::U8(_) => {}
            Value::Usize(_) => {}
            Value::Variant(v) => self._visit_value_variant(v),
            Value::Vec(v) => self._visit_value_vec(v),
            Value::Writer(_) => {}
            Value::Instance(_) => {}
            Value::Ordering(_) => {}
            Value::Backend(_) => {}
            Value::Range(v) => self._visit_value_range(v),
        }
    }

    fn _visit_value_range(&mut self, v: &runtime::builtins::range::Range<Rc<Value>>) {
        if let Some(v) = &v.start {
            self.visit_value(v);
        }
        if let Some(v) = &v.end {
            self.visit_value(v);
        }
    }

    fn _visit_value_aggregator(
        &mut self,
        a: &Aggregator<Rc<Value>, Rc<Value>, Rc<Value>, Rc<Value>>,
    ) {
        match a {
            Aggregator::Incremental {
                lift,
                combine,
                lower,
            } => {
                self.visit_value(lift);
                self.visit_value(combine);
                self.visit_value(lower);
            }
            Aggregator::Holistic { compute } => {
                self.visit_value(compute);
            }
        }
    }

    fn _visit_value_array(&mut self, a: &crate::builtins::types::array::Array) {
        for v in &a.0 {
            self.visit_value(v);
        }
    }

    fn _visit_value_vec(&mut self, v: &runtime::builtins::vec::Vec<Value>) {
        for v in v.0.iter() {
            self.visit_value(v);
        }
    }

    fn _visit_value_dict(&mut self, v: &runtime::builtins::dict::Dict<Value, Value>) {
        for (k, v) in v.0.iter() {
            self.visit_value(k);
            self.visit_value(v);
        }
    }

    fn _visit_value_option(&mut self, o: &runtime::builtins::option::Option<Rc<Value>>) {
        if let Some(v) = &o.0 {
            self.visit_value(v);
        }
    }

    fn _visit_value_record(&mut self, v: &Record) {
        for v in v.0.values() {
            self.visit_value(v);
        }
    }

    fn _visit_value_variant(&mut self, v: &Variant) {
        self.visit_value(&v.v);
    }

    fn _visit_value_tuple(&mut self, v: &Tuple) {
        for v in &v.0 {
            self.visit_value(v);
        }
    }

    fn _visit_value_result(&mut self, v: &runtime::builtins::result::Result<Rc<Value>>) {
        if let Ok(v) = &v.0 {
            self.visit_value(v)
        }
    }

    fn _visit_value_set(&mut self, s: &Set<Value>) {
        for v in s.0.iter() {
            self.visit_value(v);
        }
    }

    fn _visit_value_dataflow(&mut self, d: &Dataflow) {
        match d {
            Dataflow::Collocate(d1, d2) => {
                self._visit_value_dataflow(d1);
                self._visit_value_dataflow(d2);
            }
            Dataflow::Sink(s, _, _) => {
                self._visit_value_stream(s);
            }
        }
    }

    fn _visit_value_stream(&mut self, s: &Stream) {
        match s {
            Stream::Source(_, _, f) => {
                self._visit_value_fun(f);
            }
            Stream::Map(s, f) => {
                self._visit_value_stream(s);
                self._visit_value_fun(f);
            }
            Stream::Filter(_, _) => todo!(),
            Stream::Flatten(_) => todo!(),
            Stream::FlatMap(_, _) => todo!(),
            Stream::Keyby(_, _) => todo!(),
            Stream::Unkey(_) => todo!(),
            Stream::Window(_, _, _) => todo!(),
            Stream::Merge(_, _) => todo!(),
            Stream::Sink(_, _, _) => todo!(),
        }
    }

    fn _visit_value_fun(&mut self, f: &Fun) {
        f.params.values().for_each(|t| self.visit_type(t));
        self.visit_stmt_def_body(&f.body);
    }
}

pub(crate) trait AcceptVisitor {
    fn visit(&self, visitor: &mut impl Visitor);
}

impl AcceptVisitor for Program {
    fn visit(&self, mut visitor: &mut impl Visitor) {
        visitor.visit_program(self);
    }
}

impl AcceptVisitor for Stmt {
    fn visit(&self, mut visitor: &mut impl Visitor) {
        visitor.visit_stmt(self);
    }
}

impl AcceptVisitor for Expr {
    fn visit(&self, mut visitor: &mut impl Visitor) {
        visitor.visit_expr(self);
    }
}

impl AcceptVisitor for Type {
    fn visit(&self, mut visitor: &mut impl Visitor) {
        visitor.visit_type(self);
    }
}
