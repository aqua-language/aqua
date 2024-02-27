#![allow(unused)]
use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Query;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::TraitDef;
use crate::ast::TraitType;
use crate::ast::Type;
use crate::ast::UnresolvedPath;
use crate::print::Print;

pub struct Wrapper<T>(T);

impl<'a> std::fmt::Display for Wrapper<&'a Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Rust::new(f);
        p.type_info = true;
        p.program(self.0)
    }
}

impl<'a, 'b> Print<'b> for Rust<'a, 'b> {
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.f
    }

    fn get_indent(&mut self) -> &mut usize {
        &mut self.indent_level
    }

    fn should_indent(&mut self) -> bool {
        self.noindent
    }
}

impl Program {
    pub fn rust(&self) -> Wrapper<&Self> {
        Wrapper(self)
    }
}

pub struct Rust<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    noindent: bool,
    indent_level: usize,
    type_info: bool,
}

impl<'a, 'b> Rust<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Rust<'a, 'b> {
        Rust {
            f,
            noindent: false,
            indent_level: 0,
            type_info: false,
        }
    }

    fn program(&mut self, p: &Program) -> std::fmt::Result {
        self.newline_sep(&p.stmts, Self::stmt)
    }

    fn param(&mut self, p: &Param) -> std::fmt::Result {
        self.name(&p.name)?;
        self.punct(":")?;
        self.space()?;
        self.ty(&p.ty)
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
            Stmt::Err(_) => todo!(),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) -> std::fmt::Result {
        self.kw("let")?;
        self.space()?;
        self.name(&s.name)?;
        self.punct(":")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.expr(&s.expr)?;
        self.punct(";")
    }

    fn stmt_def(&mut self, s: &StmtDef) -> std::fmt::Result {
        self.kw("fn")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.paren(|this| this.comma_sep(&s.params, Self::param))?;
        self.punct(":")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.where_clause(&s.where_clause)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.body(&s.body)?;
        self.punct(";")
    }

    fn body(&mut self, e: &Body) -> std::fmt::Result {
        match e {
            Body::Expr(e) => self.expr(e),
            Body::Builtin => self.kw("<builtin>"),
        }
    }

    fn stmt_impl(&mut self, s: &StmtImpl) -> std::fmt::Result {
        self.kw("impl")?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.bound(&s.head)?;
        self.where_clause(&s.where_clause)?;
        self.space()?;
        self.brace(|this| {
            this.indented(|this| {
                this.newline_sep(&s.defs, Self::stmt_def)?;
                this.newline_sep(&s.types, Self::stmt_type)
            })
        })
    }

    fn where_clause(&mut self, ts: &[Bound]) -> std::fmt::Result {
        if !ts.is_empty() {
            self.space()?;
            self.kw("where")?;
            self.space()?;
            self.comma_sep(ts, Self::bound)?;
        }
        Ok(())
    }

    fn stmt_expr(&mut self, s: &Expr) -> std::fmt::Result {
        self.expr(s)?;
        self.punct(";")
    }

    fn stmt_struct(&mut self, s: &StmtStruct) -> std::fmt::Result {
        self.kw("struct")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.fields(&s.fields, Self::annotate)?;
        self.punct(";")
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> std::fmt::Result {
        self.kw("enum")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.scope(&s.variants, Self::variant)
    }

    fn variant(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.paren(|this| this.ty(t))
    }

    fn stmt_type(&mut self, s: &StmtType) -> std::fmt::Result {
        self.kw("type")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.punct(";")
    }

    fn stmt_trait(&mut self, s: &StmtTrait) -> std::fmt::Result {
        self.kw("trait")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.brace(|this| {
            this.indented(|this| {
                this.newline_sep(&s.defs, Self::stmt_def_decl)?;
                this.newline_sep(&s.types, Self::stmt_type_decl)
            })
        })
    }

    fn stmt_def_decl(&mut self, s: &TraitDef) -> std::fmt::Result {
        self.kw("def")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.paren(|this| this.comma_sep(&s.params, Self::param))?;
        self.punct(":")?;
        self.space()?;
        self.ty(&s.ty)?;
        self.punct(";")
    }

    fn stmt_type_decl(&mut self, s: &TraitType) -> std::fmt::Result {
        self.kw("type")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.punct(";")
    }

    fn type_args(&mut self, ts: &[Type]) -> std::fmt::Result {
        if !ts.is_empty() {
            self.brack(|this| this.comma_sep(ts, Self::ty))?;
        }
        Ok(())
    }

    fn _expr(&mut self, expr: &Expr) -> std::fmt::Result {
        match expr {
            Expr::Unresolved(_, _, p) => unreachable!(),
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
                self.fields(xes, Self::assign)?;
            }
            Expr::Enum(_, _, name, ts, x1, e) => {
                self.name(name)?;
                self.type_args(ts)?;
                self.punct("::")?;
                self.name(x1)?;
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
            Expr::Block(_, _, ss, e) => {
                self.brace(|this| {
                    this.indented(|this| {
                        this.newline_sep(ss, |this, s| this.stmt(s))?;
                        this.newline()?;
                        this.expr(e)
                    })?;
                    this.newline()
                })?;
            }
            Expr::Query(_, _, qs) => {
                self.newline_sep(qs, Self::query_stmt)?;
            }
            Expr::Assoc(_, _, x0, ts0, x1, ts1) => {
                self.name(x0)?;
                self.type_args(ts0)?;
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
            Expr::Fun(_, _, ps, t, e) => {
                self.kw("fun")?;
                self.paren(|this| this.comma_sep(ps, Self::param))?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e)?;
            }
            Expr::Match(_, _, e, arms) => {
                self.kw("match")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.scope(arms, Self::arm)?;
            }
            Expr::While(_, _, e0, e1) => {
                self.kw("while")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.expr(e1)?;
            }
            Expr::Record(_, _, xts) => {
                self.fields(xts, Self::assign)?;
            }
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => unreachable!(),
            Expr::Postfix(_, _, _, _) => unreachable!(),
            Expr::Prefix(_, _, _, _) => unreachable!(),
        }
        Ok(())
    }

    fn arm(&mut self, (p, e): &(Pat, Expr)) -> std::fmt::Result {
        self.pat(p)?;
        self.space()?;
        self.punct("=>")?;
        self.space()?;
        self.expr(e)
    }

    fn expr(&mut self, expr: &Expr) -> std::fmt::Result {
        if self.type_info {
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

    fn scan(&mut self, (x, e): &(Name, Expr)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(" in ")?;
        self.expr(e)
    }

    fn assign(&mut self, (x, e): &(Name, Expr)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(" = ")?;
        self.expr(e)
    }

    fn bind(&mut self, (p, e): &(Name, Pat)) -> std::fmt::Result {
        self.name(p)?;
        self.punct("=")?;
        self.pat(e)
    }

    fn annotate(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(":")?;
        self.ty(t)
    }

    fn query_stmt(&mut self, q: &Query) -> std::fmt::Result {
        match q {
            Query::From(_, _, xes) => {
                self.kw("from")?;
                self.space()?;
                self.comma_scope(xes, Self::scan)?;
            }
            Query::Where(_, _, e) => {
                self.kw("where")?;
                self.space()?;
                self.expr(e)?;
            }
            Query::Select(_, _, xes) => {
                self.kw("select")?;
                self.space()?;
                self.comma_scope(xes, Self::assign)?;
            }
            Query::Join(_, _, xes, e) => {
                self.kw("join")?;
                self.space()?;
                self.comma_scope(xes, Self::scan)?;
                self.space()?;
                self.punct("on")?;
                self.space()?;
                self.expr(e)?;
            }
            Query::Group(_, _, xs, qs) => {
                self.kw("group")?;
                self.space()?;
                self.comma_sep(xs, Self::name)?;
                self.space()?;
                self.comma_scope(qs, Self::query_stmt)?;
            }
            Query::Over(_, _, e, qs) => {
                self.kw("over")?;
                self.space()?;
                self.expr(e)?;
                self.comma_scope(qs, Self::query_stmt)?;
            }
            Query::Order(_, _, os) => {
                self.kw("order")?;
                self.space()?;
                self.comma_sep(os, Self::ordering)?;
            }
            Query::With(_, _, xes) => {
                self.kw("with")?;
                self.space()?;
                self.comma_scope(xes, Self::assign)?;
            }
            Query::Into(_, _, e) => {
                self.kw("into")?;
                self.space()?;
                self.comma_sep(e, Self::expr)?;
            }
            Query::Compute(_, _, aggs) => {
                self.kw("compute")?;
                self.space()?;
                self.comma_scope(aggs, Self::agg)?;
            }
        }
        Ok(())
    }

    fn ordering(&mut self, (x, d): &(Name, bool)) -> std::fmt::Result {
        self.name(x)?;
        if *d {
            self.space()?;
            self.kw("desc")?;
        }
        Ok(())
    }

    fn agg(&mut self, (x, e0, e1): &(Name, Expr, Expr)) -> std::fmt::Result {
        self.name(x)?;
        self.punct("=")?;
        self.expr(e0)?;
        self.space()?;
        self.kw("of")?;
        self.space()?;
        self.expr(e1)
    }

    fn bound(&mut self, b: &Bound) -> std::fmt::Result {
        match b {
            Bound::Unresolved(..) => unreachable!(),
            Bound::Trait(_, x, ts) => {
                self.name(x)?;
                self.type_args(ts)?;
            }
            Bound::Err(_) => unreachable!(),
        }
        Ok(())
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Cons(name, ts) => {
                self.name(name)?;
                self.type_args(ts)?;
            }
            Type::Assoc(x0, ts0, x1, ts1) => {
                self.name(x0)?;
                self.type_args(ts0)?;
                self.punct("::")?;
                self.name(x1)?;
                self.type_args(ts1)?;
            }
            Type::Var(x) => {
                self.punct("?")?;
                self.name(x)?;
            }
            Type::Hole => {
                self.punct("_")?;
            }
            Type::Err => {
                self.kw("<err>")?;
            }
            Type::Generic(x) => {
                self.name(x)?;
            }
            Type::Fun(ts, t) => {
                self.kw("fun")?;
                self.paren(|this| this.comma_sep(ts, Self::ty))?;
                self.punct(":")?;
                self.space()?;
                self.ty(t)?;
            }
            Type::Tuple(ts) => {
                self.paren(|this| this.comma_sep_trailing(ts, Self::ty))?;
            }
            Type::Record(xts) => {
                self.fields(xts, Self::annotate)?;
            }
            Type::Alias(..) => unreachable!(),
            Type::Unresolved(..) => unreachable!(),
            Type::Array(t, n) => {
                self.brack(|ctx| {
                    ctx.ty(t)?;
                    if let Some(n) = n {
                        ctx.punct(";")?;
                        ctx.lit(n)?;
                    }
                    Ok(())
                })?;
            }
            Type::Never => {
                self.punct("!")?;
            }
        }
        Ok(())
    }

    fn pat(&mut self, p: &Pat) -> std::fmt::Result {
        if self.type_info {
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
            Pat::Unresolved(..) => unreachable!(),
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
            Pat::Struct(_, _, x, ts, xps) => {
                self.name(x)?;
                self.type_args(ts)?;
                self.fields(xps, Self::bind)?;
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                self.name(x0)?;
                self.type_args(ts)?;
                self.punct("::")?;
                self.name(x1)?;
                self.paren(|this| this.pat(p))?;
            }
            Pat::Err(_, _) => {
                self.kw("<err>")?;
            }
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
}
