use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::print::Print;

use super::Codegen;

// This wrapper causes the underlying structure to be printed as Rust code.
pub struct Rust<T>(T);

impl<'a> Codegen<'a> {
    pub fn rust(&self) -> Rust<&Self> {
        Rust(self)
    }
}

impl Program {
    pub fn rust(&self) -> Rust<&Self> {
        Rust(self)
    }
}

impl StmtDef {
    pub fn rust(&self) -> Rust<&Self> {
        Rust(self)
    }
}

impl StmtStruct {
    pub fn rust(&self) -> Rust<&Self> {
        Rust(self)
    }
}

impl StmtEnum {
    pub fn rust(&self) -> Rust<&Self> {
        Rust(self)
    }
}

impl<'a> std::fmt::Display for Rust<&'a Codegen<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in self.0.decls.defs.values() {
            writeln!(f, "{}", stmt.rust())?;
        }
        for stmt in self.0.decls.structs.values() {
            writeln!(f, "{}", stmt.rust())?;
        }
        for stmt in self.0.decls.enums.values() {
            writeln!(f, "{}", stmt.rust())?;
        }
        writeln!(f, "{}", self.0.dataflow)
    }
}

impl<'a> std::fmt::Display for Rust<&'a Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.type_info = true;
        p.program(self.0)
    }
}

impl<'a> std::fmt::Display for Rust<&'a StmtDef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.type_info = true;
        p.stmt_def(self.0)
    }
}

impl<'a> std::fmt::Display for Rust<&'a StmtStruct> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.type_info = true;
        p.stmt_struct(self.0)
    }
}

impl<'a> std::fmt::Display for Rust<&'a StmtEnum> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.type_info = true;
        p.stmt_enum(self.0)
    }
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
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

struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    noindent: bool,
    indent_level: usize,
    type_info: bool,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer {
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
            Stmt::Impl(_) => unreachable!(),
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Struct(s) => self.stmt_struct(s),
            Stmt::Enum(s) => self.stmt_enum(s),
            Stmt::Type(s) => self.stmt_type(s),
            Stmt::Trait(_) => unreachable!(),
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
        assert!(s.generics.is_empty());
        assert!(s.where_clause.is_empty());
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.kw("fn")?;
                self.space()?;
                self.name(&s.name)?;
                self.paren(|this| this.comma_sep(&s.params, Self::param))?;
                self.space()?;
                self.punct("->")?;
                self.space()?;
                self.ty(&s.ty)?;
                self.space()?;
                self.brace(|this| {
                    this.space()?;
                    this.expr(e)?;
                    this.space()
                })?;
            }
            ExprBody::Builtin(b) => {
                self.kw("const")?;
                self.space()?;
                self.name(&s.name)?;
                self.punct(":")?;
                self.kw("fn")?;
                self.paren(|this| this.comma_sep(s.params.values(), Self::ty))?;
                self.space()?;
                self.punct("->")?;
                self.space()?;
                self.ty(&s.ty)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit(b.rust)?;
                // assert!(s.where_clause.is_empty());
                // self.space()?;
                // self.punct(";")?;
            }
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

    fn ty_body(&mut self, t: &TypeBody) -> std::fmt::Result {
        match t {
            TypeBody::UserDefined(t) => self.ty(t),
            TypeBody::Builtin(_) => todo!(),
        }
    }

    fn type_args(&mut self, ts: &[Type]) -> std::fmt::Result {
        if !ts.is_empty() {
            self.brack(|this| this.comma_sep(ts, Self::ty))?;
        }
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> std::fmt::Result {
        match e {
            Expr::Path(_, _, _) => unreachable!(),
            Expr::Int(_, _, v) => {
                self.lit(v)?;
            }
            Expr::Float(_, _, v) => {
                self.lit(v)?;
            }
            Expr::Bool(_, _, v) => {
                self.lit(v)?;
            }
            Expr::Char(_, _, v) => {
                self.char(*v)?;
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
            Expr::QueryInto(..) => unreachable!(),
            Expr::TraitMethod(..) => unreachable!(),
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
                self.bars(|this| this.comma_sep(ps, Self::param))?;
                self.punct("->")?;
                self.space()?;
                self.ty(t)?;
                self.space()?;
                self.brace(|this| {
                    this.space()?;
                    this.expr(e)?;
                    this.space()
                })?;
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
            Expr::Unresolved(_, _, _, _) => unreachable!(),
            Expr::InfixBinaryOp(_, _, _, _, _) => unreachable!(),
            Expr::PrefixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::PostfixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::Annotate(_, _, _) => unreachable!(),
            Expr::Paren(_, _, _) => unreachable!(),
            Expr::Dot(_, _, _, _, _, _) => unreachable!(),
            Expr::IfElse(_, _, e, b0, b1) => {
                self.kw("if")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.block(b0)?;
                self.space()?;
                self.kw("else")?;
                self.space()?;
                self.block(b1)?;
            }
            Expr::IntSuffix(_, _, _, _) => unreachable!(),
            Expr::FloatSuffix(_, _, _, _) => unreachable!(),
            Expr::LetIn(_, _, _, _, _, _) => todo!(),
            Expr::Update(_, _, _, _, _) => todo!(),
            Expr::Anonymous(_, _) => unreachable!(),
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

    fn arm(&mut self, pe: &(Pat, Expr)) -> std::fmt::Result {
        self.pat(&pe.0)?;
        self.space()?;
        self.punct("=>")?;
        self.space()?;
        self.expr(&pe.1)
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
            Type::Assoc(..) => unreachable!("{t}"),
            Type::Var(_) => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Err => unreachable!(),
            Type::Generic(_) => unreachable!(),
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
            Type::Paren(t) => {
                self.paren(|this| this.ty(t))?;
            }
        }
        Ok(())
    }

    fn pat(&mut self, p: &Pat) -> std::fmt::Result {
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
            Pat::Annotate(_, _, _) => unreachable!(),
            Pat::Paren(_, _, _) => unreachable!(),
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
