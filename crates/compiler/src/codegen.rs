#![allow(unused)]
use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Query;
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

    fn param(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(":")?;
        self.space()?;
        self.ty(t)
    }

    fn stmt(&mut self, s: &Stmt) -> std::fmt::Result {
        match s {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(s) => self.stmt_def(s),
            Stmt::Impl(s) => unreachable!(),
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
        if let StmtDefBody::UserDefined(e) = &s.body {
            self.kw("fn")?;
            self.space()?;
            self.name(&s.name)?;
            assert!(s.generics.is_empty());
            self.paren(|this| this.comma_sep(&s.params, Self::param))?;
            self.punct(":")?;
            self.space()?;
            self.ty(&s.ty)?;
            assert!(s.where_clause.is_empty());
            self.space()?;
            self.brace(|this| this.expr(e))?;
        }
        Ok(())
    }

    fn body(&mut self, e: &StmtDefBody) -> std::fmt::Result {
        match e {
            StmtDefBody::UserDefined(e) => self.expr(e),
            StmtDefBody::Builtin(_) => self.kw("<builtin>"),
        }
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
        self.fields(s.fields.as_ref(), Self::annotate)?;
        self.punct(";")
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> std::fmt::Result {
        self.kw("enum")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.scope(s.variants.as_ref(), Self::variant)
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
        self.ty_body(&s.body)?;
        self.punct(";")
    }

    fn ty_body(&mut self, t: &StmtTypeBody) -> std::fmt::Result {
        match t {
            StmtTypeBody::UserDefined(t) => self.ty(t),
            StmtTypeBody::Builtin(b) => self.kw("<builtin>"),
        }
    }

    fn stmt_trait(&mut self, s: &StmtTrait) -> std::fmt::Result {
        self.kw("trait")?;
        self.space()?;
        self.name(&s.name)?;
        assert!(s.generics.is_empty());
        self.space()?;
        self.brace(|this| {
            this.indented(|this| {
                this.newline_sep(&s.defs, |ctx, s| ctx.stmt_def_decl(s))?;
                this.newline_sep(&s.types, |ctx, s| ctx.stmt_type_decl(s))
            })
        })
    }

    fn stmt_def_decl(&mut self, s: &StmtTraitDef) -> std::fmt::Result {
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

    fn stmt_type_decl(&mut self, s: &StmtTraitType) -> std::fmt::Result {
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

    fn _expr(&mut self, e: &Expr) -> std::fmt::Result {
        match e {
            Expr::Path(_, _, p) => unreachable!(),
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
                self.fields(xes.as_ref(), Self::assign)?;
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
            Expr::Block(_, _, b) => {
                self.block(b)?;
            }
            Expr::Query(..) => unreachable!(),
            Expr::Assoc(..) => unreachable!(),
            Expr::Index(_, _, e, i) => {
                self.expr(e)?;
                self.punct(".")?;
                self.index(i)?;
            }
            Expr::Array(_, _, es) => {
                self.brack(|this| this.comma_sep(es, Self::expr))?;
            }
            Expr::Err(..) => unreachable!(),
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
            Expr::While(_, _, e, b) => {
                self.kw("while")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.block(b)?;
            }
            Expr::Record(_, _, xts) => {
                self.fields(xts.as_ref(), Self::assign)?;
            }
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
            Expr::Unresolved(_, _, _, _) => unreachable!(),
        }
        Ok(())
    }

    fn block(&mut self, b: &Block) -> std::fmt::Result {
        self.brace(|this| {
            this.indented(|this| {
                this.newline_sep(&b.stmts, |this, s| this.stmt(s))?;
                this.newline()?;
                this.expr(&b.expr)
            })?;
            this.newline()
        })
    }

    fn arm(&mut self, arm: &Arm) -> std::fmt::Result {
        self.pat(&arm.p)?;
        self.space()?;
        self.punct("=>")?;
        self.space()?;
        self.expr(&arm.e)
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

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Cons(name, ts) => {
                self.name(name)?;
                self.type_args(ts)?;
            }
            Type::Assoc(..) => unreachable!(),
            Type::Var(_, _) => unreachable!(),
            Type::Hole => unreachable!(),
            Type::Err => unreachable!(),
            Type::Generic(x) => unreachable!(),
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
                self.fields(xts.as_ref(), Self::annotate)?;
            }
            Type::Alias(..) => unreachable!(),
            Type::Path(..) => unreachable!(),
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
            Pat::Path(..) => unreachable!(),
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
                self.fields(xps.as_ref(), Self::bind)?;
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
            Pat::Record(_, _, xps) => {
                self.fields(xps.as_ref(), Self::bind)?;
            }
            Pat::Or(_, _, p0, p1) => {
                self.pat(p0)?;
                self.punct("|")?;
                self.pat(p1)?;
            }
            Pat::Char(_, _, c) => {
                write!(self.f, "{:?}", c)?;
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
