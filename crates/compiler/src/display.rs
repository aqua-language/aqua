use crate::ast::Aggr;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Index;
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
use crate::ast::TypeVar;
use crate::print::Print;

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).expr(self)
    }
}

impl std::fmt::Display for ExprBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).expr_body(self)
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).block(self)
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).program(self)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).path(self)
    }
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt(self)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).ty(self)
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).name(self)
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).index(self)
    }
}

impl std::fmt::Display for Pat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).pat(self)
    }
}

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).bound(self)
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).query_clause(self)
    }
}

impl std::fmt::Display for StmtImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_impl(self)
    }
}

impl std::fmt::Display for StmtDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_def(self)
    }
}

impl std::fmt::Display for StmtVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_var(self)
    }
}

impl std::fmt::Display for StmtStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_struct(self)
    }
}

impl std::fmt::Display for StmtTraitDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_def_decl(self)
    }
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for StmtTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).stmt_trait(self)
    }
}

impl<'a, 'b> Print<'b> for Pretty<'a, 'b> {
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

pub struct Pretty<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    noindent: bool,
    indent_level: usize,
    verbose: bool,
}

impl<'a, 'b> Pretty<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Pretty<'a, 'b> {
        Pretty {
            f,
            noindent: false,
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

    fn program(&mut self, p: &Program) -> std::fmt::Result {
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

    fn stmt_var(&mut self, s: &StmtVar) -> std::fmt::Result {
        self.kw("var")?;
        self.space()?;
        self.name(&s.name)?;
        if s.ty != Type::Unknown {
            self.punct(":")?;
            self.space()?;
            self.ty(&s.ty)?;
        }
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
            ExprBody::Builtin(_) => self.kw("<builtin>"),
        }
    }

    fn stmt_impl(&mut self, s: &StmtImpl) -> std::fmt::Result {
        self.kw("impl")?;
        self.generics(&s.generics)?;
        self.space()?;
        self.bound(&s.head)?;
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

    fn where_clause(&mut self, ts: &[Trait]) -> std::fmt::Result {
        self.if_nonempty(ts, |this, ts| {
            this.space()?;
            this.kw("where")?;
            this.space()?;
            this.comma_sep(ts, Self::bound)
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
            TypeBody::Builtin(_) => self.kw("<builtin>"),
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
                self.fields(xes.as_ref(), Self::assign)?;
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
            Expr::TraitMethod(_, _, b, x1, ts1) => {
                self.bound(b)?;
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
                self.comma_scope(arms, Self::arm)?;
            }
            Expr::While(_, _, e0, b) => {
                self.kw("while")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.block(b)?;
            }
            Expr::Record(_, _, xts) => {
                self.kw("record")?;
                self.fields(xts.as_ref(), Self::assign)?;
            }
            Expr::Value(_, _) => {
                self.kw("<value>")?;
            }
            Expr::For(_, _, x, e, b) => {
                self.kw("for")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e)?;
                self.space()?;
                self.block(b)?;
            }
            Expr::Char(_, _, c) => {
                self.char(*c)?;
            }
            Expr::Unresolved(_, _, x, ts) => {
                self.name(x)?;
                self.type_args(ts)?;
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
                if this.verbose || !b.expr.is_unit() {
                    this.newline()?;
                    this.expr(&b.expr)?;
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
                this.ty(expr.type_of())
            })?;
        } else {
            self._expr(expr)?;
        }
        Ok(())
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

    fn query_clause(&mut self, q: &Query) -> std::fmt::Result {
        match q {
            Query::From(_, x, e) => {
                self.kw("from")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e)?;
            }
            Query::Where(_, e) => {
                self.kw("where")?;
                self.space()?;
                self.expr(e)?;
            }
            Query::Select(_, xes) => {
                self.kw("select")?;
                self.space()?;
                self.comma_scope(xes.as_ref(), Self::assign)?;
            }
            Query::GroupOverCompute(_, x, e0, e1, aggrs) => {
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
            Query::OverCompute(_, e, aggrs) => {
                self.kw("over")?;
                self.space()?;
                self.expr(e)?;
                self.indented(|this| {
                    this.kw("compute")?;
                    this.newline_comma_sep(aggrs, Self::aggr)
                })?;
            }
            Query::Var(_, x, e) => {
                self.kw("var")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.expr(e)?;
            }
            Query::JoinOn(_, x, e0, e1) => {
                self.kw("join")?;
                self.space()?;
                self.name(x)?;
                self.space()?;
                self.kw("in")?;
                self.space()?;
                self.expr(e0)?;
                self.space()?;
                self.kw("on")?;
                self.space()?;
                self.expr(e1)?;
            }
            Query::JoinOverOn(_, x, e0, e1, e2) => {
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
            Query::Err(_) => {
                self.kw("<err>")?;
            }
        }
        Ok(())
    }

    fn aggr(&mut self, a: &Aggr) -> std::fmt::Result {
        self.name(&a.x)?;
        self.punct("=")?;
        self.expr(&a.e0)?;
        self.space()?;
        self.kw("of")?;
        self.space()?;
        self.expr(&a.e1)
    }

    fn bound(&mut self, b: &Trait) -> std::fmt::Result {
        match b {
            Trait::Path(_, path) => {
                self.path(path)?;
            }
            Trait::Cons(x, ts, xts) => {
                self.name(x)?;
                if !ts.is_empty() || !xts.is_empty() {
                    self.brack(|this| {
                        this.if_nonempty(ts, |this, ts| this.comma_sep(ts, Self::ty))?;
                        this.if_nonempty(xts, |this, xts| {
                            if !ts.is_empty() {
                                this.punct(",")?;
                                this.space()?;
                            }
                            this.comma_sep(xts, |this, (x, t)| {
                                this.name(x)?;
                                this.punct("=")?;
                                this.ty(t)
                            })
                        })?;
                        Ok(())
                    })?;
                }
            }
            Trait::Type(t) => {
                self.ty(t)?;
            }
            Trait::Err => {
                self.kw("<err>")?;
            }
            Trait::Var(_) => todo!(),
        }
        Ok(())
    }

    fn ty(&mut self, t: &Type) -> std::fmt::Result {
        match t {
            Type::Cons(name, ts) => {
                self.name(name)?;
                self.type_args(ts)?;
            }
            Type::Assoc(b, x1, ts1) => {
                self.bound(b)?;
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
                self.kw("record")?;
                self.fields(xts.as_ref(), Self::annotate)?;
            }
            Type::Alias(name, xts) => {
                self.name(name)?;
                self.type_args(xts)?;
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
        }
        Ok(())
    }

    fn pat(&mut self, p: &Pat) -> std::fmt::Result {
        if self.verbose {
            self.paren(|this| {
                this._pat(p)?;
                this.punct(":")?;
                this.ty(p.type_of())
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
                self.fields(xps, Self::bind)?;
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
                self.fields(xps, Self::bind)?;
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
        }
        Ok(())
    }

    fn path(&mut self, p: &Path) -> std::fmt::Result {
        self.sep("::", false, &p.segments, Self::segment)
    }

    fn segment(&mut self, seg: &Segment) -> std::fmt::Result {
        self.name(&seg.name)?;
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
}

pub trait IntoVerbose {
    fn verbose(&self) -> Verbose<&Self>;
}

impl IntoVerbose for Vec<Stmt> {
    fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl IntoVerbose for Vec<Expr> {
    fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl IntoVerbose for Vec<Type> {
    fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Stmt>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).verbose().stmts(self.0)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Expr>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).verbose().expr_args(self.0)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Vec<Type>> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Pretty::new(f).verbose().type_args(self.0)
    }
}

pub struct Verbose<T>(T);

impl Program {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Expr {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Pat {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Stmt {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Type {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Path {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl StmtTraitDef {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl StmtTrait {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl StmtImpl {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl Trait {
    pub fn verbose(&self) -> Verbose<&Self> {
        Verbose(self)
    }
}

impl<'a> std::fmt::Display for Verbose<&'a Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().program(self.0)
    }
}

impl std::fmt::Display for Verbose<&StmtImpl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().stmt_impl(self.0)
    }
}

impl std::fmt::Display for Verbose<&Expr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().expr(self.0)
    }
}

impl std::fmt::Display for Verbose<&Pat> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().pat(self.0)
    }
}

impl std::fmt::Display for Verbose<&Stmt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().stmt(self.0)
    }
}

impl std::fmt::Display for Verbose<&Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Pretty::new(f);
        p.verbose = true;
        p.ty(self.0)
    }
}

impl std::fmt::Display for Verbose<&Path> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().path(self.0)
    }
}

impl std::fmt::Display for Verbose<&StmtTraitDef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().stmt_def_decl(self.0)
    }
}

impl std::fmt::Display for Verbose<&Trait> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pretty::new(f).verbose().bound(self.0)
    }
}
