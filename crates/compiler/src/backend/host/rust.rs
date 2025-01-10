use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtLocal;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::Type;
use crate::backend::codegen::Codegen;
use crate::builtins::value::Function;
use crate::print::Print;

// This wrapper causes the underlying structure to be printed as Rust code.
struct Wrapper<T>(T, usize);

impl Ast {
    pub fn to_rust(&self) -> impl std::fmt::Display + '_ {
        Wrapper(self, 0)
    }
}

impl crate::analysis::declare::Context {
    pub fn to_rust(&self) -> impl std::fmt::Display + '_ {
        Wrapper(self, 0)
    }
}

impl Function {
    pub fn to_rust(&self, indent: usize) -> impl std::fmt::Display + '_ {
        Wrapper(self, indent)
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a Ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.program(self.0)
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a crate::analysis::declare::Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
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
        Ok(())
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.indent = self.1;
        p.fun(self.0)
    }
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.f
    }

    fn indent_mut(&mut self) -> &mut usize {
        &mut self.indent
    }
}

struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    indent: usize,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Printer<'a, 'b> {
        Printer { f, indent: 0 }
    }
}

impl<'a, 'b> Codegen<'b> for Printer<'a, 'b> {
    fn program(&mut self, p: &Ast) -> std::fmt::Result {
        self.newline_sep(&p.stmts, Self::stmt)
    }

    fn local(&mut self, l: &Local) -> std::fmt::Result {
        if l.mutable {
            self.kw("mut")?;
            self.space()?;
        }
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
        self.kw("let")?;
        self.space()?;
        self.local(&s.local)?;
        self.space()?;
        self.punct("=")?;
        self.space()?;
        self.expr(&s.expr)?;
        self.punct(";")
    }

    fn stmt_def(&mut self, s: &StmtDef) -> std::fmt::Result {
        match &s.body {
            ExprBody::UserDefined(e) => {
                self.kw("fn")?;
                self.space()?;
                self.name(&s.name)?;
                self.paren(|this| this.comma_sep(&s.params, Self::local))?;
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
                self.space()?;
                self.punct(":")?;
                self.space()?;
                self.kw("fn")?;
                self.paren(|this| this.comma_sep(s.params.iter().map(|v| &v.ty), Self::ty))?;
                self.space()?;
                self.punct("->")?;
                self.space()?;
                self.ty(&s.ty)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit(b.codegen.as_ref().unwrap().rust)?;
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
        self.lit("#[data]")?;
        self.newline()?;
        self.kw("struct")?;
        self.space()?;
        self.name(&s.name)?;
        self.space()?;
        self.fields(s.fields.as_ref(), Self::type_field)
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
        self.lit(&s.body.as_bit().unwrap().codegen.as_ref().unwrap().rust)?;
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
                self.paren(|this| this.comma_sep_trailing(es, Self::expr))?;
            }
            Expr::Struct(_, _, name, _, xes) => {
                self.name(name)?;
                self.space()?;
                self.fields(xes.as_ref(), Self::expr_field)?;
            }
            Expr::Enum(_, _, name, _, x1, e) => {
                self.name(name)?;
                self.punct("::")?;
                self.name(x1)?;
                self.paren(|this| this.expr(e))?;
            }
            Expr::Local(_, _, x, _) => {
                self.name(x)?;
            }
            Expr::Def(_, _, name, _) => {
                self.name(name)?;
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
            Expr::For(_, _, _, _, _) => todo!(),
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
            Expr::Unit(_, _) => {
                self.kw("()")?;
            }
        }
        Ok(())
    }

    fn block(&mut self, b: &Block) -> std::fmt::Result {
        self.brace(|this| {
            this.indented(|this| {
                this.newline_sep(&b.stmts, Self::stmt)?;
                this.newline()?;
                if let Some(e) = b.expr.as_ref() {
                    this.expr(e)?;
                }
                Ok(())
            })?;
            this.newline()
        })
    }

    fn expr_field(&mut self, (x, e): &(Name, Expr)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(": ")?;
        self.expr(e)
    }

    fn type_field(&mut self, (x, t): &(Name, Type)) -> std::fmt::Result {
        self.name(x)?;
        self.punct(":")?;
        self.space()?;
        self.ty(t)
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Builtin(name, _) => {
                self.name(name)?;
            }
            Type::Alias(..) => unreachable!(),
            Type::Assoc(..) => unreachable!("{t}"),
            Type::Var(_) => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Err => unreachable!(),
            Type::Generic(_) => unreachable!(),
            Type::Function(ts, t) => {
                self.kw("fn")?;
                self.paren(|this| this.comma_sep(ts, Self::ty))?;
                self.space()?;
                self.punct("->")?;
                self.space()?;
                self.ty(t)?;
            }
            Type::Tuple(ts) => {
                self.paren(|this| this.comma_sep_trailing(ts, Self::ty))?;
            }
            Type::Unit => {
                self.kw("()")?;
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
            Type::Struct(x, ts) => {
                self.name(x)?;
                self.space()?;
                self.fields(ts, Self::ty)?;
            }
            Type::Enum(x, ts) => {
                self.name(x)?;
                self.space()?;
                self.fields(ts, Self::ty)?;
            }
            Type::Ref(_, _, _) => todo!(),
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
