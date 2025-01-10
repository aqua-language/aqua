use std::fmt::Display;

use crate::analysis::declare;
use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::StmtLocal;
use crate::ast::Type;
use crate::backend::codegen::Codegen;
use crate::builtins::value::Function;
use crate::print::Print;

// This wrapper causes the underlying structure to be printed as Java code.
struct Wrapper<T>(T, usize);

impl Ast {
    pub fn to_java(&self) -> impl Display + '_ {
        Wrapper(self, 0)
    }
}

impl declare::Context {
    pub fn to_java(&self) -> impl Display + '_ {
        Wrapper(self, 0)
    }
}

impl Function {
    pub fn to_java(&self, indent: usize) -> impl Display + '_ {
        Wrapper(self, indent)
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a Ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.indent_level = self.1;
        p.program(self.0)
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a declare::Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.indent_level = self.1;
        for stmt in self.0.defs.values() {
            p.stmt_def(stmt)?;
        }
        p.newline()?;
        for stmt in self.0.structs.values() {
            p.stmt_struct(stmt)?;
        }
        p.newline()?;
        for stmt in self.0.enums.values() {
            p.stmt_enum(stmt)?;
        }
        p.newline()
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.indent_level = self.1;
        p.fun(self.0)
    }
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.f
    }

    fn indent_mut(&mut self) -> &mut usize {
        &mut self.indent_level
    }
}

pub struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    indent_level: usize,
}

impl<'a, 'b> Printer<'a, 'b> {
    pub fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer { f, indent_level: 0 }
    }
}

impl<'a, 'b> Codegen<'b> for Printer<'a, 'b> {
    fn program(&mut self, p: &Ast) -> std::fmt::Result {
        self.newline_sep(&p.stmts, Self::stmt)
    }

    fn local(&mut self, l: &Local) -> std::fmt::Result {
        self.name(&l.name)?;
        self.punct(":")?;
        self.space()?;
        self.ty(&l.ty)
    }

    fn stmt(&mut self, s: &Stmt) -> std::fmt::Result {
        match s {
            Stmt::Local(s) => self.stmt_var(s),
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

    fn stmt_var(&mut self, s: &StmtLocal) -> std::fmt::Result {
        self.ty(&s.local.ty)?;
        self.space()?;
        self.name(&s.local.name)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.expr(&s.expr)?;
        self.punct(";")
    }

    fn stmt_def(&mut self, s: &StmtDef) -> std::fmt::Result {
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.ty(&s.ty)?;
                self.space()?;
                self.name(&s.name)?;
                self.paren(|this| this.comma_sep(&s.params, Self::local))?;
                self.space()?;
                self.brace(|this| {
                    this.space()?;
                    this.kw("return")?;
                    this.space()?;
                    this.expr(e)?;
                    this.space()
                })?;
            }
            ExprBody::Builtin(b) => {
                self.kw("static")?;
                self.space()?;
                self.kw("final")?;
                self.space()?;
                self.ty(&s.ty())?;
                self.space()?;
                self.name(&s.name)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit(b.codegen.as_ref().unwrap().java)?;
                self.punct(";")?;
            }
        }
        Ok(())
    }

    fn stmt_expr(&mut self, s: &Expr) -> std::fmt::Result {
        self.expr(s)?;
        self.punct(";")
    }

    fn stmt_struct(&mut self, s: &StmtStruct) -> std::fmt::Result {
        self.kw("public")?;
        self.space()?;
        self.kw("static")?;
        self.space()?;
        self.kw("class")?;
        self.space()?;
        self.name(&s.name)?;
        self.space()?;
        self.brace(|this| this.fields(s.fields.as_ref(), Self::type_field))
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> std::fmt::Result {
        self.kw("enum")?;
        self.space()?;
        self.name(&s.name)?;
        self.space()?;
        self.scope(s.variants.as_ref(), |this, (x, t)| {
            this.name(x)?;
            this.paren(|this| this.ty(t))
        })
    }

    fn stmt_type(&mut self, s: &StmtType) -> std::fmt::Result {
        self.kw("type")?;
        self.space()?;
        self.name(&s.name)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.lit(&s.body.as_bit().unwrap().codegen.as_ref().unwrap().java)?;
        self.punct(";")
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
                self.kw("new")?;
                self.space()?;
                self.lit("Tuple")?;
                self.lit(es.len())?;
                self.paren(|this| this.comma_sep_trailing(es, Self::expr))?;
            }
            Expr::Struct(_, _, x, _, xes) => {
                self.name(x)?;
                self.space()?;
                self.fields(xes.as_ref(), Self::expr_field)?;
            }
            Expr::Enum(_, _, x, _, x1, e) => {
                self.name(x)?;
                self.punct("::")?;
                self.name(x1)?;
                self.paren(|this| this.expr(e))?;
            }
            Expr::Local(_, _, x, _) => {
                self.name(x)?;
            }
            Expr::Def(_, _, x, _) => {
                self.name(x)?;
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
            Expr::Lambda(_, _, ps, t, e) => {
                self.bars(|this| this.comma_sep(ps, Self::local))?;
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
            Expr::Match(..) => unreachable!(),
            Expr::While(_, _, e, b) => {
                self.kw("while")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.block(b)?;
            }
            Expr::Record(_, _, xts) => {
                self.fields(xts.as_ref(), Self::expr_field)?;
            }
            Expr::For(_, _, _, _, _) => unreachable!(),
            Expr::InfixBinaryOp(_, _, _, _, _) => unreachable!(),
            Expr::PrefixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::PostfixUnaryOp(_, _, _, _) => unreachable!(),
            Expr::Annotate(_, _, _) => unreachable!(),
            Expr::Paren(_, _, _) => unreachable!(),
            Expr::Dot(_, _, _, _, _, _) => unreachable!(),
            Expr::IfElse(_, _, e, b0, b1) => {
                self.paren(|this| this.expr(e))?;
                self.space()?;
                self.kw("?")?;
                self.space()?;
                self.block(b0)?;
                self.space()?;
                self.kw(":")?;
                self.space()?;
                self.block(b1)?;
            }
            Expr::Closure(_, _, _, _xts0, _xts1, _t, _e) => {
                todo!()
            }
            Expr::IntSuffix(_, _, _, _) => unreachable!(),
            Expr::FloatSuffix(_, _, _, _) => unreachable!(),
            Expr::Anonymous(_, _) => unreachable!(),
            Expr::Ref(_, _, _, _) => todo!(),
            Expr::Place(_, _, _) => todo!(),
            Expr::Deref(_, _, _) => todo!(),
            Expr::Loop(_, _, _) => todo!(),
            Expr::Unit(_, _) => todo!(),
        }
        Ok(())
    }

    fn block(&mut self, b: &Block) -> std::fmt::Result {
        self.paren(|this| {
            this.paren(|this| {
                this.lit("Supplier")?;
                this.angle(|this| this.ty(b.ty()))
            })?;
            this.space()?;
            this.paren(|_| Ok(()))?;
            this.space()?;
            this.punct("->")?;
            this.space()?;
            this.brace(|this| {
                this.indented(|this| {
                    this.newline_sep(&b.stmts, Self::stmt)?;
                    this.newline()?;
                    this.kw("return")?;
                    this.space()?;
                    this.expr(&b.expr.as_ref().unwrap())?;
                    this.punct(";")
                })?;
                this.newline()
            })
        })?;
        self.punct(".")?;
        self.lit("get")?;
        self.paren(|_| Ok(()))?;
        Ok(())
    }

    fn expr_field(&mut self, (_, e): &(Name, Expr)) -> std::fmt::Result {
        self.expr(e)
    }

    fn type_field(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.ty(t)?;
        self.space()?;
        self.name(x)
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Builtin(x, _) => {
                // TODO: Handle these in the declaration context.
                match x.data.as_str() {
                    "i32" => self.lit("Integer")?,
                    "i64" => self.lit("Long")?,
                    "f64" => self.lit("Float")?,
                    _ => self.name(x)?,
                };
            }
            Type::Alias(..) => unreachable!(),
            Type::Assoc(..) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Err => unreachable!(),
            Type::Generic(_) => unreachable!(),
            Type::Function(ts, t) => {
                self.kw("Function")?;
                self.lit(ts.len())?;
                self.angle(|this| {
                    this.comma_sep(ts, Self::ty)?;
                    if !ts.is_empty() {
                        this.punct(",")?;
                        this.space()?;
                    }
                    this.ty(t)
                })?;
            }
            Type::Tuple(ts) => {
                self.lit("Tuple")?;
                self.lit(ts.len())?;
                self.angle(|this| this.comma_sep(ts, Self::ty))?;
            }
            Type::Record(xts) => {
                self.fields(xts.as_ref(), Self::type_field)?;
            }
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
            Type::Struct(_, _) => todo!(),
            Type::Enum(_, _) => todo!(),
            Type::Ref(_, _, _) => todo!(),
            Type::Unit => todo!(),
        }
        Ok(())
    }

    fn fields<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result {
        self.brace(|this| {
            this.space()?;
            this.comma_sep(items, |this, item| f(this, item))?;
            this.space()
        })
    }

    fn fun(&mut self, f: &Function) -> std::fmt::Result {
        self.bars(|this| this.comma_sep(&f.params, Self::local))?;
        self.space()?;
        self.expr(f.body.as_udf().unwrap())
    }
}
