use std::rc::Rc;

use crate::ast::Block;
use crate::ast::Bound;
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
use crate::ast::Type;
use crate::lexer::Span;

#[allow(unused)]
pub(crate) trait Mapper {
    #[inline(always)]
    fn enter_scope(&mut self) {}

    #[inline(always)]
    fn exit_scope(&mut self) {}

    fn map_program(&mut self, program: &Program) -> Program {
        self._map_program(program)
    }
    #[inline(always)]
    fn _map_program(&mut self, program: &Program) -> Program {
        let stmts = self.map_stmts(&program.stmts);
        Program::new(stmts)
    }

    #[inline(always)]
    fn map_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        self._map_stmts(stmts)
    }
    #[inline(always)]
    fn _map_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        self.map_iter(stmts, Self::map_stmt)
    }

    fn map_stmt(&mut self, stmt: &Stmt) -> Stmt {
        self._map_stmt(stmt)
    }
    #[inline(always)]
    fn _map_stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.map_stmt_var(s))),
            Stmt::Def(s) => Stmt::Def(Rc::new(self.map_stmt_def(s))),
            Stmt::Trait(s) => Stmt::Trait(Rc::new(self.map_stmt_trait(s))),
            Stmt::Impl(s) => Stmt::Impl(Rc::new(self.map_stmt_impl(s))),
            Stmt::Struct(s) => Stmt::Struct(Rc::new(self.map_stmt_struct(s))),
            Stmt::Enum(s) => Stmt::Enum(Rc::new(self.map_stmt_enum(s))),
            Stmt::Type(s) => Stmt::Type(Rc::new(self.map_stmt_type(s))),
            Stmt::Expr(e) => Stmt::Expr(Rc::new(self.map_expr(e))),
            Stmt::Err(s) => Stmt::Err(self.map_span(s)),
        }
    }

    fn map_span(&mut self, span: &Span) -> Span {
        self._map_span(span)
    }
    #[inline(always)]
    fn _map_span(&mut self, s: &Span) -> Span {
        *s
    }

    fn map_stmt_var(&mut self, stmt: &StmtVar) -> StmtVar {
        self._map_stmt_var(stmt)
    }
    #[inline(always)]
    fn _map_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let ty = self.map_type(&s.ty);
        let expr = self.map_expr(&s.expr);
        StmtVar::new(span, name, ty, expr)
    }

    fn map_stmt_def(&mut self, stmt: &StmtDef) -> StmtDef {
        self._map_stmt_def(stmt)
    }
    #[inline(always)]
    fn _map_stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let params = self.map_params(&s.params).into();
        let ty = self.map_type(&s.ty);
        let where_clause = self.map_bounds(&s.where_clause);
        let body = self.map_stmt_def_body(&s.body);
        self.exit_scope();
        StmtDef::new(span, name, generics, params, ty, where_clause, body)
    }

    #[inline(always)]
    fn map_params(&mut self, ps: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self._map_params(ps)
    }
    #[inline(always)]
    fn _map_params(&mut self, ps: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self.map_iter(ps, Self::map_param)
    }

    fn map_param(&mut self, xt: &(Name, Type)) -> (Name, Type) {
        self._map_param(xt)
    }
    #[inline(always)]
    fn _map_param(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    #[inline(always)]
    fn map_generics(&mut self, gs: &[Name]) -> Vec<Name> {
        self._map_generics(gs)
    }
    #[inline(always)]
    fn _map_generics(&mut self, gs: &[Name]) -> Vec<Name> {
        self.map_iter(gs, Self::map_generic)
    }

    fn map_generic(&mut self, g: &Name) -> Name {
        self._map_generic(g)
    }
    #[inline(always)]
    fn _map_generic(&mut self, g: &Name) -> Name {
        self.map_name(g)
    }

    fn map_stmt_def_body(&mut self, b: &StmtDefBody) -> StmtDefBody {
        self._map_stmt_def_body(b)
    }
    #[inline(always)]
    fn _map_stmt_def_body(&mut self, b: &StmtDefBody) -> StmtDefBody {
        match b {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(self.map_expr(e)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }

    #[inline(always)]
    fn map_bounds(&mut self, bs: &[Bound]) -> Vec<Bound> {
        self._map_bounds(bs)
    }
    #[inline(always)]
    fn _map_bounds(&mut self, bs: &[Bound]) -> Vec<Bound> {
        self.map_iter(bs, Self::map_bound)
    }

    fn map_bound(&mut self, b: &Bound) -> Bound {
        self._map_bound(b)
    }
    #[inline(always)]
    fn _map_bound(&mut self, b: &Bound) -> Bound {
        match b {
            Bound::Path(span, path) => {
                let span = self.map_span(span);
                let path = self.map_path(path);
                Bound::Path(span, path)
            }
            Bound::Trait(span, x, ts, xts) => {
                let span = self.map_span(span);
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                let xts = self.map_iter(xts, Self::map_assoc_type).into();
                Bound::Trait(span, x, ts, xts)
            }
            Bound::Type(span, t) => {
                let span = self.map_span(span);
                let t = self.map_type(t);
                Bound::Type(span, Rc::new(t))
            }
            Bound::Err(s) => Bound::Err(self.map_span(s)),
        }
    }

    fn map_assoc_type(&mut self, xt: &(Name, Type)) -> (Name, Type) {
        self._map_assoc_type(xt)
    }
    #[inline(always)]
    fn _map_assoc_type(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    fn map_stmt_trait(&mut self, stmt: &StmtTrait) -> StmtTrait {
        self._map_stmt_trait(stmt)
    }
    #[inline(always)]
    fn _map_stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let where_clause = self.map_bounds(&s.where_clause);
        let defs = self.map_trait_defs(&s.defs);
        let types = self.map_trait_types(&s.types);
        self.exit_scope();
        StmtTrait::new(span, name, generics, where_clause, defs, types)
    }

    fn map_trait_defs(&mut self, ds: &[Rc<StmtTraitDef>]) -> Vec<Rc<StmtTraitDef>> {
        self._map_trait_defs(ds)
    }
    #[inline(always)]
    fn _map_trait_defs(&mut self, ds: &[Rc<StmtTraitDef>]) -> Vec<Rc<StmtTraitDef>> {
        self.map_rc_iter(ds, Self::map_stmt_trait_def)
    }

    fn map_stmt_trait_def(&mut self, d: &StmtTraitDef) -> StmtTraitDef {
        self._map_trait_def(d)
    }
    #[inline(always)]
    fn _map_trait_def(&mut self, d: &StmtTraitDef) -> StmtTraitDef {
        self.enter_scope();
        let span = self.map_span(&d.span);
        let name = self.map_name(&d.name);
        let generics = self.map_generics(&d.generics);
        let params = self.map_iter(&d.params, Self::map_trait_def_param).into();
        let ty = self.map_type(&d.ty);
        let where_clause = self.map_bounds(&d.where_clause);
        self.exit_scope();
        StmtTraitDef::new(span, name, generics, params, ty, where_clause)
    }

    fn map_trait_def_param(&mut self, xt: &(Name, Type)) -> (Name, Type) {
        self._map_trait_def_param(xt)
    }
    #[inline(always)]
    fn _map_trait_def_param(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    fn map_trait_types(&mut self, ts: &[Rc<StmtTraitType>]) -> Vec<Rc<StmtTraitType>> {
        self._map_trait_types(ts)
    }
    #[inline(always)]
    fn _map_trait_types(&mut self, ts: &[Rc<StmtTraitType>]) -> Vec<Rc<StmtTraitType>> {
        self.map_rc_iter(ts, Self::map_trait_type)
    }

    fn map_trait_type(&mut self, t: &StmtTraitType) -> StmtTraitType {
        self._map_trait_type(t)
    }
    #[inline(always)]
    fn _map_trait_type(&mut self, t: &StmtTraitType) -> StmtTraitType {
        self.enter_scope();
        let span = self.map_span(&t.span);
        let name = self.map_name(&t.name);
        let generics = self.map_generics(&t.generics);
        self.exit_scope();
        StmtTraitType::new(span, name, generics)
    }

    fn map_stmt_impl(&mut self, stmt: &StmtImpl) -> StmtImpl {
        self._map_stmt_impl(stmt)
    }
    #[inline(always)]
    fn _map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let generics = self.map_generics(&s.generics);
        let where_clause = self.map_bounds(&s.where_clause);
        let head = self.map_bound(&s.head);
        let defs = self.map_rc_iter(&s.defs, Self::map_stmt_def);
        let types = self.map_rc_iter(&s.types, Self::map_stmt_type);
        self.exit_scope();
        StmtImpl::new(span, generics, head, where_clause, defs, types)
    }

    fn map_stmt_struct(&mut self, stmt: &StmtStruct) -> StmtStruct {
        self._map_stmt_struct(stmt)
    }
    #[inline(always)]
    fn _map_stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let fields = self.map_type_fields(&s.fields).into();
        self.exit_scope();
        StmtStruct::new(span, name, generics, fields)
    }

    #[inline(always)]
    fn map_type_fields(&mut self, fs: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self._map_type_fields(fs)
    }
    #[inline(always)]
    fn _map_type_fields(&mut self, fs: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self.map_iter(fs, Self::map_type_field)
    }

    fn map_type_field(&mut self, f: &(Name, Type)) -> (Name, Type) {
        self._map_type_field(f)
    }
    #[inline(always)]
    fn _map_type_field(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    fn map_stmt_enum(&mut self, stmt: &StmtEnum) -> StmtEnum {
        self._map_stmt_enum(stmt)
    }
    #[inline(always)]
    fn _map_stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let variants = self.map_iter(&s.variants, Self::map_type_variant).into();
        self.exit_scope();
        StmtEnum::new(span, name, generics, variants)
    }

    fn map_type_variants(&mut self, vs: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self._map_type_variants(vs)
    }
    #[inline(always)]
    fn _map_type_variants(&mut self, vs: &[(Name, Type)]) -> Vec<(Name, Type)> {
        self.map_iter(vs, Self::map_type_variant)
    }

    fn map_type_variant(&mut self, v: &(Name, Type)) -> (Name, Type) {
        self._map_type_variant(v)
    }
    #[inline(always)]
    fn _map_type_variant(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    fn map_stmt_type(&mut self, stmt: &StmtType) -> StmtType {
        self._map_stmt_type(stmt)
    }
    #[inline(always)]
    fn _map_stmt_type(&mut self, s: &StmtType) -> StmtType {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let body = self.map_stmt_type_body(&s.body);
        self.exit_scope();
        StmtType::new(span, name, generics, body)
    }

    fn map_stmt_type_body(&mut self, b: &StmtTypeBody) -> StmtTypeBody {
        self._map_stmt_type_body(b)
    }
    #[inline(always)]
    fn _map_stmt_type_body(&mut self, b: &StmtTypeBody) -> StmtTypeBody {
        match b {
            StmtTypeBody::UserDefined(t) => StmtTypeBody::UserDefined(self.map_type(t)),
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
        }
    }

    fn map_expr(&mut self, expr: &Expr) -> Expr {
        self._map_expr(expr)
    }

    #[inline(always)]
    fn _map_expr(&mut self, expr: &Expr) -> Expr {
        let span = self.map_span(&expr.span_of());
        let ty = self.map_type(&expr.type_of());
        match expr {
            Expr::Path(_, _, path) => {
                let path = self.map_path(path);
                Expr::Path(span, ty, path)
            }
            Expr::Unresolved(_, _, x, ts) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Expr::Unresolved(span, ty, x, ts)
            }
            Expr::Int(_, _, v) => Expr::Int(span, ty, *v),
            Expr::Float(_, _, v) => Expr::Float(span, ty, *v),
            Expr::Bool(_, _, v) => Expr::Bool(span, ty, *v),
            Expr::String(_, _, v) => Expr::String(span, ty, *v),
            Expr::Char(_, _, v) => Expr::Char(span, ty, *v),
            Expr::Struct(_, _, x, ts, xes) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                let xes = self.map_expr_fields(xes).into();
                Expr::Struct(span, ty, x, ts, xes)
            }
            Expr::Tuple(_, _, es) => {
                let es = self.map_exprs(es);
                Expr::Tuple(span, ty, es)
            }
            Expr::Record(_, _, xes) => {
                let xes = self.map_iter(xes, Self::map_expr_field).into();
                Expr::Record(span, ty, xes)
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                let x0 = self.map_name(x0);
                let ts = self.map_types(ts);
                let x1 = self.map_name(x1);
                let e = self.map_expr(e);
                Expr::Enum(span, ty, x0, ts, x1, Rc::new(e))
            }
            Expr::Field(_, _, e, x) => {
                let e = self.map_expr(e);
                let x = self.map_name(x);
                Expr::Field(span, ty, Rc::new(e), x)
            }
            Expr::Index(_, _, e, i) => {
                let e = self.map_expr(e);
                let i = *i;
                Expr::Index(span, ty, Rc::new(e), i)
            }
            Expr::Var(_, _, x) => {
                let x = self.map_name(x);
                Expr::Var(span, ty, x)
            }
            Expr::Def(_, _, x, ts) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Expr::Def(span, ty, x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = self.map_expr(e);
                let es = self.map_exprs(es);
                Expr::Call(span, ty, Rc::new(e), es)
            }
            Expr::Block(_, _, b) => {
                let b = self.map_block(b);
                Expr::Block(span, ty, b)
            }
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, b, x, ts) => {
                let b = self.map_bound(b);
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Expr::Assoc(span, ty, b, x, ts)
            }
            Expr::Match(_, _, e, arms) => {
                let e = self.map_expr(e);
                let arms = self.map_iter(arms, Self::map_arm).into();
                Expr::Match(span, ty, Rc::new(e), arms)
            }
            Expr::Array(_, _, es) => {
                let es = self.map_exprs(es);
                Expr::Array(span, ty, es)
            }
            Expr::Assign(_, _, e0, e1) => {
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                Expr::Assign(span, ty, Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, e) => {
                let e = self.map_expr(e);
                Expr::Return(span, ty, Rc::new(e))
            }
            Expr::Continue(_, _) => Expr::Continue(span, ty),
            Expr::Break(_, _) => Expr::Break(span, ty),
            Expr::While(_, _, e, b) => {
                let e = self.map_expr(e);
                let b = self.map_block(b);
                Expr::While(span, ty, Rc::new(e), b)
            }
            Expr::Fun(_, _, ps, t, e) => {
                self.enter_scope();
                let ps = self.map_params(ps).into();
                let t = self.map_type(t);
                let e = self.map_expr(e);
                self.exit_scope();
                Expr::Fun(span, ty, ps, t, Rc::new(e))
            }
            Expr::For(_, _, x, e, b) => {
                self.enter_scope();
                let x = self.map_name(x);
                let e = self.map_expr(e);
                let b = self.map_block(b);
                self.exit_scope();
                Expr::For(span, ty, x, Rc::new(e), b)
            }
            Expr::Err(_, _) => Expr::Err(span, ty),
            Expr::Value(_, _) => todo!(),
            Expr::QueryInto(_, _, qs, x, ts, es) => {
                let qs = self.map_query_stmts(qs);
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                let es = self.map_exprs(es);
                Expr::QueryInto(span, ty, qs, x, ts, es)
            }
            Expr::InfixBinaryOp(_, _, op, e0, e1) => {
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                Expr::InfixBinaryOp(span, ty, *op, Rc::new(e0), Rc::new(e1))
            }
            Expr::PrefixUnaryOp(_, _, op, e) => {
                let e = self.map_expr(e);
                Expr::PrefixUnaryOp(span, ty, *op, Rc::new(e))
            }
            Expr::PostfixUnaryOp(_, _, op, e) => {
                let e = self.map_expr(e);
                Expr::PostfixUnaryOp(span, ty, *op, Rc::new(e))
            }
            Expr::Annotate(_, _, e) => {
                let e = self.map_expr(e);
                Expr::Annotate(span, ty, Rc::new(e))
            }
            Expr::Paren(_, _, e) => {
                let e = self.map_expr(e);
                Expr::Paren(span, ty, Rc::new(e))
            }
            Expr::Dot(_, _, e, x, ts, es) => {
                let e = self.map_expr(e);
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                let es = self.map_exprs(es);
                Expr::Dot(span, ty, Rc::new(e), x, ts, es)
            }
            Expr::IfElse(_, _, e, b0, b1) => {
                let e = self.map_expr(e);
                let b0 = self.map_block(b0);
                let b1 = self.map_block(b1);
                Expr::IfElse(span, ty, Rc::new(e), b0, b1)
            }
            Expr::IntSuffix(_, _, v, x) => Expr::IntSuffix(span, ty, *v, *x),
            Expr::FloatSuffix(_, _, v, x) => Expr::FloatSuffix(span, ty, *v, *x),
        }
    }

    fn map_query_stmts(&mut self, qs: &[Query]) -> Vec<Query> {
        self._map_query_stmts(qs)
    }
    #[inline(always)]
    fn _map_query_stmts(&mut self, qs: &[Query]) -> Vec<Query> {
        self.map_iter(qs, Self::map_query_stmt)
    }

    fn map_query_stmt(&mut self, q: &Query) -> Query {
        self._map_query_stmt(q)
    }
    #[inline(always)]
    fn _map_query_stmt(&mut self, q: &Query) -> Query {
        let s = self.map_span(&q.span_of());
        match q {
            Query::From(_, x, e) => {
                let x = self.map_name(x);
                let e = self.map_expr(e);
                Query::From(s, x, Rc::new(e))
            }
            Query::Var(_, x, e) => {
                let x = self.map_name(x);
                let e = self.map_expr(e);
                Query::Var(s, x, Rc::new(e))
            }
            Query::Where(_, e) => {
                let e = self.map_expr(e);
                Query::Where(s, Rc::new(e))
            }
            Query::Select(_, xes) => {
                let xes = self.map_expr_fields(xes).into();
                Query::Select(s, xes)
            }
            Query::OverCompute(_, e, aggs) => {
                let e = self.map_expr(e);
                Query::OverCompute(s, Rc::new(e), aggs.clone())
            }
            Query::GroupOverCompute(_, x, e0, e1, aggs) => {
                let x = self.map_name(x);
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                Query::GroupOverCompute(s, x, Rc::new(e0), Rc::new(e1), aggs.clone())
            }
            Query::Join(_, x, e0, e1, e2) => {
                let x = self.map_name(x);
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                let e2 = self.map_expr(e2);
                Query::Join(s, x, Rc::new(e0), Rc::new(e1), Rc::new(e2))
            }
            Query::JoinOver(_, x, e0, e1, e2, e3) => {
                let x = self.map_name(x);
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                let e2 = self.map_expr(e2);
                let e3 = self.map_expr(e3);
                Query::JoinOver(s, x, Rc::new(e0), Rc::new(e1), Rc::new(e2), Rc::new(e3))
            }
            Query::Err(_) => Query::Err(s),
        }
    }

    #[inline(always)]
    fn map_exprs(&mut self, exprs: &[Expr]) -> Vec<Expr> {
        self._map_exprs(exprs)
    }
    #[inline(always)]
    fn _map_exprs(&mut self, exprs: &[Expr]) -> Vec<Expr> {
        self.map_iter(exprs, Self::map_expr)
    }

    #[inline(always)]
    fn map_expr_fields(&mut self, xes: &[(Name, Expr)]) -> Vec<(Name, Expr)> {
        self._map_expr_fields(xes)
    }
    #[inline(always)]
    fn _map_expr_fields(&mut self, xes: &[(Name, Expr)]) -> Vec<(Name, Expr)> {
        self.map_iter(xes, Self::map_expr_field)
    }

    #[inline(always)]
    fn map_arms(&mut self, arms: &[(Pat, Expr)]) -> Vec<(Pat, Expr)> {
        self._map_arms(arms)
    }
    #[inline(always)]
    fn _map_arms(&mut self, arms: &[(Pat, Expr)]) -> Vec<(Pat, Expr)> {
        self.map_iter(arms, Self::map_arm)
    }

    fn map_arm(&mut self, arm: &(Pat, Expr)) -> (Pat, Expr) {
        self._map_arm(arm)
    }
    #[inline(always)]
    fn _map_arm(&mut self, (p, e): &(Pat, Expr)) -> (Pat, Expr) {
        self.enter_scope();
        let p = self.map_pattern(p);
        let e = self.map_expr(e);
        self.exit_scope();
        (p, e)
    }

    fn map_expr_field(&mut self, ef: &(Name, Expr)) -> (Name, Expr) {
        self._map_expr_field(ef)
    }
    #[inline(always)]
    fn _map_expr_field(&mut self, (x, e): &(Name, Expr)) -> (Name, Expr) {
        let x = self.map_name(x);
        let e = self.map_expr(e);
        (x, e)
    }

    fn map_path(&mut self, path: &Path) -> Path {
        self._map_path(path)
    }
    #[inline(always)]
    fn _map_path(&mut self, path: &Path) -> Path {
        let segments = self.map_iter(&path.segments, Self::map_segment);
        Path::new(segments)
    }

    fn map_segment(&mut self, seg: &Segment) -> Segment {
        self._map_segment(seg)
    }
    #[inline(always)]
    fn _map_segment(&mut self, seg: &Segment) -> Segment {
        let span = self.map_span(&seg.span);
        let name = self.map_name(&seg.name);
        let types = self.map_types(&seg.ts);
        let named_types = self.map_iter(&seg.xts, Self::map_segment_named_type).into();
        Segment::new(span, name, types, named_types)
    }

    fn map_segment_named_type(&mut self, xt: &(Name, Type)) -> (Name, Type) {
        self._map_segment_named_type(xt)
    }
    #[inline(always)]
    fn _map_segment_named_type(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        let x = self.map_name(x);
        let t = self.map_type(t);
        (x, t)
    }

    fn map_name(&mut self, name: &Name) -> Name {
        self._map_name(name)
    }
    #[inline(always)]
    fn _map_name(&mut self, name: &Name) -> Name {
        *name
    }

    fn map_block(&mut self, b: &Block) -> Block {
        self._map_block(b)
    }
    #[inline(always)]
    fn _map_block(&mut self, b: &Block) -> Block {
        self.enter_scope();
        let span = self.map_span(&b.span);
        let stmts = self.map_stmts(&b.stmts);
        let expr = self.map_expr(&b.expr);
        self.exit_scope();
        Block::new(span, stmts, expr)
    }

    #[inline(always)]
    fn map_types(&mut self, ts: &[Type]) -> Vec<Type> {
        self._map_types(ts)
    }
    #[inline(always)]
    fn _map_types(&mut self, ts: &[Type]) -> Vec<Type> {
        self.map_iter(ts, Self::map_type)
    }

    fn map_type(&mut self, t: &Type) -> Type {
        self._map_type(t)
    }

    #[inline(always)]
    fn _map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Path(path) => {
                let path = self.map_path(path);
                Type::Path(path)
            }
            Type::Cons(x, ts) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Type::Cons(x, ts)
            }
            Type::Alias(x, ts) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Type::Alias(x, ts)
            }
            Type::Assoc(b, x, ts) => {
                let b = self.map_bound(b);
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                Type::Assoc(b, x, ts)
            }
            Type::Var(v) => Type::Var(*v),
            Type::Generic(x) => {
                let x = self.map_name(x);
                Type::Generic(x)
            }
            Type::Fun(ts, t) => {
                let ts = self.map_types(ts);
                let t = self.map_type(t);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = self.map_types(ts);
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = self.map_iter(xts, Self::map_type_field).into();
                Type::Record(xts)
            }
            Type::Array(t, i) => {
                let t = self.map_type(t);
                let i = *i;
                Type::Array(Rc::new(t), i)
            }
            Type::Never => Type::Never,
            Type::Unknown => Type::Unknown,
            Type::Err => Type::Err,
            Type::Paren(t) => {
                let t = self.map_type(t);
                Type::Paren(Rc::new(t))
            }
        }
    }

    #[inline(always)]
    fn map_patterns(&mut self, ps: &[Pat]) -> Vec<Pat> {
        self._map_patterns(ps)
    }
    #[inline(always)]
    fn _map_patterns(&mut self, ps: &[Pat]) -> Vec<Pat> {
        self.map_iter(ps, Self::map_pattern)
    }

    fn map_pattern(&mut self, p: &Pat) -> Pat {
        self._map_pattern(p)
    }
    #[inline(always)]
    fn _map_pattern(&mut self, p: &Pat) -> Pat {
        let t = self.map_type(&p.type_of());
        let s = self.map_span(&p.span_of());
        match p {
            Pat::Path(_, _, path, ppfs) => {
                let path = self.map_path(path);
                let ppfs = ppfs
                    .as_ref()
                    .map(|ppfs| self.map_iter(ppfs, Self::map_path_pat_field));
                Pat::Path(s, t, path, ppfs)
            }
            Pat::Var(_, _, x) => {
                let x = self.map_name(x);
                Pat::Var(s, t, x)
            }
            Pat::Tuple(_, _, ps) => {
                let ps = self.map_patterns(ps);
                Pat::Tuple(s, t, ps)
            }
            Pat::Struct(_, _, x, ts, xps) => {
                let x = self.map_name(x);
                let ts = self.map_types(ts);
                let xps = self.map_iter(xps, Self::map_pattern_field).into();
                Pat::Struct(s, t, x, ts, xps)
            }
            Pat::Record(_, _, xps) => {
                let xps = self.map_iter(xps, Self::map_pattern_field).into();
                Pat::Record(s, t, xps)
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                let x0 = self.map_name(x0);
                let ts = self.map_types(ts);
                let x1 = self.map_name(x1);
                let p = self.map_pattern(p);
                Pat::Enum(s, t, x0, ts, x1, Rc::new(p))
            }
            Pat::Int(_, _, v) => Pat::Int(s, t, *v),
            Pat::String(_, _, v) => Pat::String(s, t, *v),
            Pat::Char(_, _, v) => Pat::Char(s, t, *v),
            Pat::Bool(_, _, v) => Pat::Bool(s, t, *v),
            Pat::Wildcard(_, _) => Pat::Wildcard(s, t),
            Pat::Or(_, _, p0, p1) => {
                let p0 = self.map_pattern(p0);
                let p1 = self.map_pattern(p1);
                Pat::Or(s, t, Rc::new(p0), Rc::new(p1))
            }
            Pat::Err(_, _) => Pat::Err(s, t),
            Pat::Annotate(_, _, p) => {
                let p = self.map_pattern(p);
                Pat::Annotate(s, t, Rc::new(p))
            }
            Pat::Paren(_, _, p) => {
                let p = self.map_pattern(p);
                Pat::Paren(s, t, Rc::new(p))
            }
        }
    }

    fn map_pattern_field(&mut self, pf: &(Name, Pat)) -> (Name, Pat) {
        self._map_pattern_field(pf)
    }
    #[inline(always)]
    fn _map_pattern_field(&mut self, (x, p): &(Name, Pat)) -> (Name, Pat) {
        let x = self.map_name(x);
        let p = self.map_pattern(p);
        (x, p)
    }

    fn map_path_pat_field(&mut self, pf: &PathPatField) -> PathPatField {
        self._map_path_pat_field(pf)
    }
    #[inline(always)]
    fn _map_path_pat_field(&mut self, pf: &PathPatField) -> PathPatField {
        match pf {
            PathPatField::Named(x, p) => {
                let x = self.map_name(x);
                let p = self.map_pattern(p);
                PathPatField::Named(x, p)
            }
            PathPatField::Unnamed(p) => {
                let p = self.map_pattern(p);
                PathPatField::Unnamed(p)
            }
        }
    }

    #[inline(always)]
    fn map_rc_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a Rc<T>>,
        f: impl Fn(&mut Self, &T) -> T,
    ) -> Vec<Rc<T>> {
        iter.into_iter().map(|x| Rc::new(f(self, &x))).collect()
    }

    #[inline(always)]
    fn map_iter<'a, T: 'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a T>,
        f: impl Fn(&mut Self, &T) -> T,
    ) -> Vec<T> {
        iter.into_iter().map(|x| f(self, x)).collect()
    }
}
