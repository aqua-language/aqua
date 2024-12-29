use crate::ast::Aggr;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::ImplVar;
use crate::ast::Index;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::Ast;
use crate::ast::QueryOp;
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
use crate::ast::TypeVar;
use crate::pass::infer::solver::Constraint;
use crate::print::Print;

struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    indent_level: usize,
    verbose: bool,
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.f
    }

    fn indent_mut(&mut self) -> &mut usize {
        &mut self.indent_level
    }
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer {
            f,
            indent_level: 0,
            verbose: false,
        }
    }

    fn verbose(&mut self) -> &mut Self {
        self.verbose = true;
        self
    }

    fn param(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(":")?;
        self.space()?;
        self.ty(t)
    }

    fn program(&mut self, p: &Ast) -> std::fmt::Result {
        self.stmts(&p.stmts)
    }

    fn stmts(&mut self, ss: &[Stmt]) -> std::fmt::Result {
        self.newline_sep(ss, Self::stmt)
    }

    fn stmt(&mut self, s: &Stmt) -> std::fmt::Result {
        match s {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(s) => self.stmt_def(s),
            Stmt::Impl(s) => self.stmt_impl(s),
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Struct(s) => self.stmt_struct(s),
            Stmt::Enum(s) => self.stmt_enum(s),
            Stmt::Type(s) => self.stmt_type(s),
            Stmt::Trait(s) => self.stmt_trait(s),
            Stmt::Err(_) => self.kw("<err>"),
        }
    }

    fn type_annotation(&mut self, t: &Type) -> std::fmt::Result {
        if *t != Type::Unknown {
            self.punct(":")?;
            self.space()?;
            self.ty(&t)?;
        }
        Ok(())
    }

    fn stmt_var(&mut self, s: &StmtVar) -> std::fmt::Result {
        self.kw("var")?;
        self.space()?;
        self.name(&s.name)?;
        self.type_annotation(&s.ty)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.expr(&s.expr)?;
        self.punct(";")
    }

    fn stmt_def(&mut self, s: &StmtDef) -> std::fmt::Result {
        self.kw("def")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.paren(|this| this.comma_sep(&s.params, Self::param))?;
        self.punct(":")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.where_clause(&s.where_clause)?;
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e)?;
                if !e.is_braced() {
                    self.punct(";")
                } else {
                    Ok(())
                }
            }
            ExprBody::Builtin(_) => self.punct(";"),
        }
    }

    fn expr_body(&mut self, e: &ExprBody) -> std::fmt::Result {
        match e {
            ExprBody::UserDefined(e) => self.expr(e),
            ExprBody::Builtin(_) => Ok(()),
        }
    }

    fn stmt_impl(&mut self, s: &StmtImpl) -> std::fmt::Result {
        self.kw("impl")?;
        self.generics(&s.generics)?;
        self.space()?;
        self.imp(&s.head)?;
        self.where_clause(&s.where_clause)?;
        self.space()?;
        self.brace(|this| {
            if !s.types.is_empty() {
                this.indented(|this| {
                    this.newline()?;
                    this.newline_sep(&s.types, |this, s| this.stmt_type(s))
                })?;
            }
            if !s.defs.is_empty() {
                this.indented(|this| {
                    this.newline()?;
                    this.newline_sep(&s.defs, |this, s| this.stmt_def(s))
                })?;
            }
            if !s.defs.is_empty() || !s.types.is_empty() {
                this.newline()?;
            }
            Ok(())
        })
    }

    fn where_clause(&mut self, ts: &[Impl]) -> std::fmt::Result {
        self.if_nonempty(ts, |this, ts| {
            this.space()?;
            this.kw("where")?;
            this.space()?;
            this.comma_sep(ts, Self::imp)
        })
    }

    fn stmt_expr(&mut self, s: &Expr) -> std::fmt::Result {
        self.expr(s)?;
        self.punct(";")
    }

    fn stmt_struct(&mut self, s: &StmtStruct) -> std::fmt::Result {
        self.kw("struct")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.fields(s.fields.as_ref(), Self::annotate)?;
        self.punct(";")
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> std::fmt::Result {
        self.kw("enum")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.space()?;
        self.brace(|this| {
            this.if_nonempty(&s.variants, |this, s| {
                this.indented(|this| {
                    this.newline()?;
                    this.newline_sep(s, Self::variant)
                })?;
                this.newline()
            })
        })
    }

    fn variant(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.paren(|this| this.ty(t))
    }

    fn stmt_type(&mut self, s: &StmtType) -> std::fmt::Result {
        self.kw("type")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.ty_body(&s.body)?;
        self.punct(";")
    }

    fn ty_body(&mut self, t: &TypeBody) -> std::fmt::Result {
        match t {
            TypeBody::UserDefined(t) => self.ty(t),
            TypeBody::Builtin(_) => Ok(()),
        }
    }

    fn stmt_trait(&mut self, s: &StmtTrait) -> std::fmt::Result {
        self.kw("trait")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.where_clause(&s.where_clause)?;
        self.space()?;
        self.brace(|this| {
            if !s.types.is_empty() {
                this.indented(|this| {
                    this.newline()?;
                    this.newline_sep(&s.types, |this, s| this.stmt_type_decl(s))
                })?;
            }
            if !s.defs.is_empty() {
                this.indented(|this| {
                    this.newline()?;
                    this.newline_sep(&s.defs, |this, s| this.stmt_def_decl(s))
                })?;
            }
            if !s.defs.is_empty() || !s.types.is_empty() {
                this.newline()?;
            }
            Ok(())
        })
    }

    fn stmt_def_decl(&mut self, s: &StmtTraitDef) -> std::fmt::Result {
        self.kw("def")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.paren(|this| this.comma_sep(&s.params, Self::param))?;
        self.punct(":")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.punct(";")
    }

    fn stmt_type_decl(&mut self, s: &StmtTraitType) -> std::fmt::Result {
        self.kw("type")?;
        self.space()?;
        self.name(&s.name)?;
        self.generics(&s.generics)?;
        self.punct(";")
    }

    fn generics(&mut self, gs: &[Name]) -> std::fmt::Result {
        self.if_nonempty(gs, |this, gs| {
            this.brack(|this| this.comma_sep(gs, Self::name))
        })
    }

    fn type_args(&mut self, ts: &[Type]) -> std::fmt::Result {
        self.if_nonempty(ts, |this, ts| {
            this.brack(|this| this.comma_sep(ts, Self::ty))
        })
    }

    fn expr_args(&mut self, es: &[Expr]) -> std::fmt::Result {
        self.paren(|this| this.comma_sep(es, Self::expr))
    }

    fn _expr(&mut self, e: &Expr) -> std::fmt::Result {
        match e {
            Expr::Path(_, _, p) => {
                self.path(p)?;
            }
            Expr::Int(_, _, v) => {
                self.lit(v)?;
            }
            Expr::Float(_, _, v) => {
                self.lit(v)?;
            }
            Expr::Bool(_, _, v) => {
                self.lit(v)?;
            }
            Expr::String(_, _, s) => {
                self.str(s)?;
            }
            Expr::Field(_, _, e, x) => {
                self.expr(e)?;
                self.punct(".")?;
                self.name(x)?;
            }
            Expr::Tuple(_, _, es) => {
                self.paren(|this| this.comma_sep_trailing(es, Self::expr))?;
            }
            Expr::Struct(_, _, name, ts, xes) => {
                self.name(name)?;
                self.type_args(ts)?;
                self.fields(xes.as_ref(), Self::expr_field)?;
            }
            Expr::Enum(_, _, name, ts, x, e) => {
                self.name(name)?;
                self.type_args(ts)?;
                self.punct("::")?;
                self.name(x)?;
                self.paren(|this| this.expr(e))?;
            }
            Expr::Var(_, _, x) => {
                self.name(x)?;
            }
            Expr::Def(_, _, name, ts) => {
                self.name(name)?;
                self.type_args(ts)?;
            }
            Expr::Call(_, _, e, es) => {
                self.expr(e)?;
                self.paren(|this| this.comma_sep(es, Self::expr))?;
            }
            Expr::Block(_, _, b) => {
                self.block(b)?;
            }
            Expr::Query(_, _, x, t, e, qs) => {
                self.kw("from")?;
                self.space()?;
                self.name(x)?;
                self.punct(":")?;
                self.ty(t)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e)?;
                if !qs.is_empty() {
                    self.newline()?;
                    self.newline_sep(qs, Self::query_clause)?;
                }
            }
            Expr::QueryInto(_, _, x0, t0, e, qs, x1, ts, es) => {
                self.kw("from")?;
                self.space()?;
                self.name(x0)?;
                self.punct(":")?;
                self.ty(t0)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e)?;
                if !qs.is_empty() {
                    self.newline()?;
                    self.newline_sep(qs, Self::query_clause)?;
                }
                self.newline()?;
                self.kw("into")?;
                self.space()?;
                self.name(x1)?;
                self.type_args(ts)?;
                self.paren(|this| this.comma_sep(es, Self::expr))?;
            }
            Expr::Assoc(_, _, b, x1, ts1) => {
                self.imp(b)?;
                self.punct("::")?;
                self.name(x1)?;
                self.type_args(ts1)?;
            }
            Expr::Index(_, _, e, i) => {
                self.expr(e)?;
                self.punct(".")?;
                self.index(i)?;
            }
            Expr::Array(_, _, es) => {
                self.brack(|this| this.comma_sep(es, Self::expr))?;
            }
            Expr::Err(_, _) => {
                self.kw("<err>")?;
            }
            Expr::Assign(_, _, e0, e1) => {
                self.expr(e0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Return(_, _, e) => {
                self.kw("return")?;
                self.space()?;
                self.expr(e)?;
            }
            Expr::Continue(_, _) => {
                self.kw("continue")?;
            }
            Expr::Break(_, _) => {
                self.kw("break")?;
            }
            Expr::Lambda(_, _, ps, _, e) => {
                if ps.len() == 1 {
                    if ps[0].1 == Type::Unknown {
                        self.name(&ps[0].0)?;
                    } else {
                        self.paren(|this| this.param(&ps[0]))?;
                    }
                } else {
                    self.paren(|this| this.comma_sep(ps, Self::param))?;
                }
                self.space()?;
                self.punct("=>")?;
                self.space()?;
                self.expr(e)?;
            }
            Expr::Match(_, _, e, arms) => {
                self.kw("match")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.comma_scope(arms, Self::arm)?;
            }
            Expr::While(_, _, e0, e1) => {
                self.kw("while")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Record(_, _, xts) => {
                self.kw("record")?;
                self.fields(xts.as_ref(), Self::expr_field)?;
            }
            Expr::For(_, _, x, e0, e1) => {
                self.kw("for")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Char(_, _, c) => {
                self.char(*c)?;
            }
            Expr::InfixBinaryOp(_, _, op, e0, e1) => {
                self.expr(e0)?;
                self.space()?;
                self.lit(op)?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::PrefixUnaryOp(_, _, op, es) => {
                self.lit(op)?;
                self.expr(es)?;
            }
            Expr::PostfixUnaryOp(_, _, op, es) => {
                self.expr(es)?;
                self.lit(op)?;
            }
            Expr::Annotate(_, t, e) => {
                self.expr(e)?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
            }
            Expr::Paren(_, _, e) => {
                self.paren(|this| this.expr(e))?;
            }
            Expr::Dot(_, _, e, x, ts, es) => {
                self.expr(e)?;
                self.punct(".")?;
                self.name(x)?;
                self.type_args(ts)?;
                self.paren(|this| this.comma_sep(es, Self::expr))?;
            }
            Expr::IfElse(_, _, e0, e1, e2) => {
                self.kw("if")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.expr(e1)?;
                self.space()?;
                self.kw("else")?;
                self.space()?;
                self.expr(e2)?;
            }
            Expr::IntSuffix(_, _, v, x) => {
                self.lit(v)?;
                self.lit(x)?;
            }
            Expr::FloatSuffix(_, _, v, x) => {
                self.lit(v)?;
                self.lit(x)?;
            }
            Expr::LetIn(_, _, x, t1, e0, e1) => {
                self.kw("let")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.punct(":")?;
                self.ty(t1)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Update(_, _, e0, x, e1) => {
                self.expr(e0)?;
                self.space()?;
                self.kw("with")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Anonymous(_, _) => {
                self.punct("_")?;
            }
            Expr::Closure(_, _, xts0, xts1, t, e) => {
                self.paren(|this| {
                    this.comma_sep(xts0, Self::param)?;
                    if !xts1.is_empty() {
                        this.punct("|")?;
                        this.space()?;
                        this.comma_sep(xts1, Self::param)?;
                    }
                    Ok(())
                })?;
                self.space()?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
                self.space()?;
                self.punct("=>")?;
                self.space()?;
                self.expr(e)?;
            }
            Expr::Ref(_, _, _) => todo!(),
            Expr::RefMut(_, _, _) => todo!(),
            Expr::Place(_, _, _) => todo!(),
            Expr::Deref(_, _, _) => todo!(),
            Expr::Unit(_, _) => {
                self.lit("()")?;
            }
        }
        Ok(())
    }

    fn block(&mut self, b: &Block) -> std::fmt::Result {
        self.brace(|this| {
            this.indented(|this| {
                if !b.stmts.is_empty() {
                    this.newline()?;
                    this.newline_sep(&b.stmts, |this, s| this.stmt(s))?;
                }
                if let Some(e) = &b.expr {
                    this.newline()?;
                    this.expr(e)?;
                }
                Ok(())
            })?;
            this.newline()
        })
    }

    fn arm(&mut self, (p, e): &(Pat, Expr)) -> std::fmt::Result {
        self.pat(&p)?;
        self.space()?;
        self.punct("=>")?;
        self.space()?;
        self.expr(&e)
    }

    fn expr(&mut self, expr: &Expr) -> std::fmt::Result {
        if self.verbose {
            self.paren(|this| {
                this._expr(expr)?;
                this.punct(":")?;
                this.ty(expr.ty())
            })?;
        } else {
            self._expr(expr)?;
        }
        Ok(())
    }

    fn expr_field(&mut self, (x, e): &(Name, Expr)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(" = ")?;
        self.expr(e)
    }

    fn pat_field(&mut self, (p, e): &(Name, Pat)) -> std::fmt::Result {
        self.name(p)?;
        self.punct("=")?;
        self.pat(e)
    }

    fn annotate(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(":")?;
        self.ty(t)
    }

    fn query_clause(&mut self, q: &QueryOp) -> std::fmt::Result {
        match q {
            QueryOp::From(_, x, t, e) => {
                self.kw("from")?;
                self.space()?;
                self.name(x)?;
                self.type_annotation(t)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e)?;
            }
            QueryOp::Union(_, e) => {
                self.kw("union")?;
                self.space()?;
                self.expr(e)?;
            }
            QueryOp::Limit(_, e) => {
                self.kw("limit")?;
                self.space()?;
                self.expr(e)?;
            }
            QueryOp::Where(_, e) => {
                self.kw("where")?;
                self.space()?;
                self.expr(e)?;
            }
            QueryOp::Select(_, xes) => {
                self.kw("select")?;
                self.space()?;
                self.comma_scope(xes.as_ref(), Self::expr_field)?;
            }
            QueryOp::GroupOverCompute(_, x, e0, e1, aggrs) => {
                self.kw("group")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e0)?;
                self.newline()?;
                self.indented(|this| {
                    this.tab()?;
                    this.kw("over")?;
                    this.space()?;
                    this.expr(e1)?;
                    this.newline()?;
                    this.kw("compute")?;
                    this.indented(|this| this.newline_comma_sep(aggrs, Self::aggr))
                })?;
            }
            QueryOp::OverCompute(_, e, aggrs) => {
                self.kw("over")?;
                self.space()?;
                self.expr(e)?;
                self.indented(|this| {
                    this.kw("compute")?;
                    this.newline_comma_sep(aggrs, Self::aggr)
                })?;
            }
            QueryOp::Var(_, x, t, e) => {
                self.kw("var")?;
                self.space()?;
                self.name(x)?;
                self.type_annotation(t)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e)?;
            }
            QueryOp::JoinOn(_, x, t, e0, e1) => {
                self.kw("join")?;
                self.space()?;
                self.name(x)?;
                self.type_annotation(t)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.kw("on")?;
                self.space()?;
                self.expr(e1)?;
            }
            QueryOp::JoinOverOn(_, x, e0, e1, e2) => {
                self.kw("join")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.kw("over")?;
                self.space()?;
                self.expr(e1)?;
                self.space()?;
                self.kw("on")?;
                self.space()?;
                self.expr(e2)?;
            }
            QueryOp::Err(_) => {
                self.kw("<err>")?;
            }
            QueryOp::Drop(_, x) => {
                self.kw("drop")?;
                self.space()?;
                self.name(x)?;
            }
        }
        Ok(())
    }

    fn aggr(&mut self, a: &Aggr) -> std::fmt::Result {
        self.name(&a.x0)?;
        self.punct("=")?;
        self.name(&a.x1)?;
        self.space()?;
        self.kw("of")?;
        self.space()?;
        self.expr(&a.e1)?;
        if let Some(e2) = &a.e2 {
            self.space()?;
            self.kw("if")?;
            self.space()?;
            self.expr(e2)?;
        }
        Ok(())
    }

    fn tr(&mut self, tr: &Trait) -> std::fmt::Result {
        self.name(&tr.x)?;
        if !tr.ts.is_empty() {
            self.brack(|this| this.if_nonempty(&tr.ts, |this, ts| this.comma_sep(ts, Self::ty)))?;
        }
        Ok(())
    }

    fn imp(&mut self, b: &Impl) -> std::fmt::Result {
        match b {
            Impl::Path(_, path) => {
                self.path(path)?;
            }
            Impl::Trait(tr) => {
                self.tr(tr)?;
            }
            Impl::Type(t) => {
                self.ty(t)?;
            }
            Impl::Err => {
                self.kw("<err>")?;
            }
            Impl::Var(x) => {
                self.punct("?")?;
                self.lit(x)?;
            }
            Impl::Unknown => self.kw("_")?,
        }
        Ok(())
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Struct(x, ts) | Type::Enum(x, ts) | Type::Builtin(x, ts) | Type::Alias(x, ts) => {
                self.name(x)?;
                self.type_args(ts)?;
            }
            Type::Assoc(b, x1, ts1) => {
                self.imp(b)?;
                self.punct("::")?;
                self.name(x1)?;
                self.type_args(ts1)?;
            }
            Type::Var(x) => {
                self.punct("'")?;
                self.lit(x)?;
            }
            Type::Unknown => {
                self.punct("_")?;
            }
            Type::Err => {
                self.kw("<err>")?;
            }
            Type::Generic(x) => {
                self.name(x)?;
            }
            Type::Function(ts, t) => {
                if ts.len() == 1 {
                    self.ty(&ts[0])?;
                } else {
                    self.paren(|this| this.comma_sep(ts, Self::ty))?;
                }
                self.space()?;
                self.punct("=>")?;
                self.space()?;
                self.ty(t)?;
            }
            Type::Tuple(ts) => {
                self.paren(|this| this.comma_sep_trailing(ts, Self::ty))?;
            }
            Type::Record(xts) => {
                self.kw("record")?;
                self.fields(xts.as_ref(), Self::annotate)?;
            }
            Type::Path(path) => {
                self.path(path)?;
            }
            Type::Array(t, n) => {
                self.brack(|this| {
                    this.ty(t)?;
                    this.if_some(n, |this, n| {
                        this.punct(";")?;
                        this.lit(n)
                    })
                })?;
            }
            Type::Never => {
                self.punct("!")?;
            }
            Type::Paren(t) => {
                self.paren(|this| this.ty(t))?;
            }
            Type::Ref(loans, t) => {
                self.punct("&")?;
                self.comma_sep(loans, Self::loan)?;
                self.ty(t)?;
            }
            Type::RefMut(_, _) => todo!(),
            Type::Unit => {
                self.lit("()")?;
            }
        }
        Ok(())
    }

    fn loan(&mut self, loan: &Loan) -> std::fmt::Result {
        if loan.mutable {
            self.punct("mut")?;
        }
        self.place(&loan.place)
    }

    fn place(&mut self, place: &Place) -> std::fmt::Result {
        self.local(&place.local)?;
        for elem in &place.elems {
            match elem {
                PlaceElem::Index(i) => {
                    self.punct(".")?;
                    self.lit(i)?;
                }
                PlaceElem::Deref => {
                    self.punct(".")?;
                    self.lit("*")?;
                }
            }
        }
        Ok(())
    }

    fn local(&mut self, local: &Local) -> std::fmt::Result {
        if local.mutable {
            self.kw("mut")?;
        }
        self.name(&local.name)?;
        self.punct(":")?;
        self.ty(&local.ty)
    }

    fn pat(&mut self, p: &Pat) -> std::fmt::Result {
        if self.verbose {
            self.paren(|this| {
                this._pat(p)?;
                this.punct(":")?;
                this.ty(p.ty())
            })
        } else {
            self._pat(p)
        }
    }

    fn _pat(&mut self, p: &Pat) -> std::fmt::Result {
        match p {
            Pat::Path(_, _, path, args) => {
                self.path(path)?;
                self.if_some(args, |this, args| {
                    this.paren(|this| {
                        this.sep(",", true, args, |this, p| match p {
                            PathPatField::Named(x, p) => {
                                this.name(x)?;
                                this.punct("=")?;
                                this.pat(p)
                            }
                            PathPatField::Unnamed(p) => this.pat(p),
                        })
                    })
                })?;
            }
            Pat::Var(_, _, x) => {
                self.name(x)?;
            }
            Pat::Int(_, _, v) => {
                write!(self.f, "{}", v)?;
            }
            Pat::Bool(_, _, v) => {
                write!(self.f, "{}", v)?;
            }
            Pat::String(_, _, v) => {
                write!(self.f, r#""{}""#, v)?;
            }
            Pat::Wildcard(_, _) => {
                self.punct("_")?;
            }
            Pat::Tuple(_, _, ps) => {
                self.paren(|this| this.comma_sep_trailing(ps, Self::pat))?;
            }
            Pat::Struct(_, _, name, ts, xps) => {
                self.name(name)?;
                self.type_args(ts)?;
                self.fields(xps, Self::pat_field)?;
            }
            Pat::Enum(_, _, name, ts, x1, p) => {
                self.name(name)?;
                self.type_args(ts)?;
                self.punct("::")?;
                self.name(x1)?;
                self.paren(|this| this.pat(p))?;
            }
            Pat::Err(_, _) => {
                self.kw("<err>")?;
            }
            Pat::Record(_, _, xps) => {
                self.kw("record")?;
                self.fields(xps, Self::pat_field)?;
            }
            Pat::Or(_, _, p0, p1) => {
                self.pat(p0)?;
                self.punct(" or ")?;
                self.pat(p1)?;
            }
            Pat::Char(_, _, v) => {
                write!(self.f, "'{}'", v)?;
            }
            Pat::Annotate(_, t, e) => {
                self.pat(e)?;
                if !self.verbose {
                    self.punct(":")?;
                    self.space()?;
                    self.ty(t)?;
                }
            }
            Pat::Paren(_, _, p) => {
                self.paren(|this| this.pat(p))?;
            }
            Pat::Unit(_, _) => {
                self.lit("()")?;
            }
        }
        Ok(())
    }

    fn path(&mut self, p: &Path) -> std::fmt::Result {
        self.sep("::", false, &p.segments, Self::segment)
    }

    fn segment(&mut self, seg: &Segment) -> std::fmt::Result {
        self.name(&seg.x)?;
        if !seg.ts.is_empty() || !seg.xts.is_empty() {
            self.brack(|this| {
                this.if_nonempty(&seg.ts, |this, ts| this.comma_sep(ts, Self::ty))?;
                this.if_nonempty(&seg.xts, |this, xts| {
                    if !seg.ts.is_empty() {
                        this.punct(",")?;
                        this.space()?;
                    }
                    this.comma_sep(xts, |this, (x, t)| {
                        this.name(x)?;
                        this.punct("=")?;
                        this.ty(t)
                    })
                })
            })?;
        }
        Ok(())
    }

    fn fields<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        if !items.is_empty() {
            self.paren(|this| this.comma_sep(items, |this, item| f(this, item)))?;
        }
        Ok(())
    }

    fn constraint(&mut self, c: &Constraint) -> std::fmt::Result {
        match c {
            Constraint::AssocDef(_, t, i, x, ts) => {
                self.lit("function")?;
                self.punct(":")?;
                self.space()?;
                self.imp(i)?;
                self.punct("::")?;
                self.name(x)?;
                self.type_args(ts)?;
                self.space()?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
            }
            Constraint::AssocType(_, t, i, x, ts) => {
                self.lit("type")?;
                self.punct(":")?;
                self.space()?;
                self.imp(i)?;
                self.punct("::")?;
                self.name(x)?;
                self.type_args(ts)?;
                self.space()?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
            }
            Constraint::WhereClause(_, i) => {
                self.lit("where")?;
                self.punct(":")?;
                self.space()?;
                self.imp(i)?;
            }
            Constraint::Field(_, t0, t1, x) => {
                self.ty(t0)?;
                self.punct(".")?;
                self.name(x)?;
                self.space()?;
                self.punct(":")?;
                self.space()?;
                self.ty(t1)?;
            }
        }
        Ok(())
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Stmt>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).verbose().stmts(self.0)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Expr>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).verbose().expr_args(self.0)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Type>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).verbose().type_args(self.0)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).expr(self)
    }
}

impl std::fmt::Display for ExprBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).expr_body(self)
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).block(self)
    }
}

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).program(self)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Printer::new(f).path(self)
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt(self)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).ty(self)
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).name(self)
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).index(self)
    }
}

impl std::fmt::Display for Pat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).pat(self)
    }
}

impl std::fmt::Display for Impl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).imp(self)
    }
}

impl std::fmt::Display for QueryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).query_clause(self)
    }
}

impl std::fmt::Display for StmtImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_impl(self)
    }
}

impl std::fmt::Display for StmtDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_def(self)
    }
}

impl std::fmt::Display for StmtVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_var(self)
    }
}

impl std::fmt::Display for StmtStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_struct(self)
    }
}

impl std::fmt::Display for StmtTraitDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_def_decl(self)
    }
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ImplVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for StmtTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).stmt_trait(self)
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).constraint(self)
    }
}

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).tr(self)
    }
}

struct Verbose<T>(T);

impl Ast {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Expr {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Pat {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Stmt {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Type {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Path {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl StmtTraitDef {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl StmtTrait {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl StmtImpl {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl Impl {
    pub fn verbose(&self) -> impl std::fmt::Display + '_ {
        Verbose(self)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().program(self.0)
    }
}

impl std::fmt::Display for Verbose<&StmtImpl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().stmt_impl(self.0)
    }
}

impl std::fmt::Display for Verbose<&StmtTrait> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().stmt_trait(self.0)
    }
}

impl std::fmt::Display for Verbose<&Expr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().expr(self.0)
    }
}

impl std::fmt::Display for Verbose<&Pat> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().pat(self.0)
    }
}

impl std::fmt::Display for Verbose<&Stmt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().stmt(self.0)
    }
}

impl std::fmt::Display for Verbose<&Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.verbose = true;
        p.ty(self.0)
    }
}

impl std::fmt::Display for Verbose<&Path> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().path(self.0)
    }
}

impl std::fmt::Display for Verbose<&StmtTraitDef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().stmt_def_decl(self.0)
    }
}

impl std::fmt::Display for Verbose<&Impl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Printer::new(f).verbose().imp(self.0)
    }
}
