use std::rc::Rc;

use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::PatArg;
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
use crate::diag::Report;
use crate::lexer::Span;
use crate::lexer::Spanned;
use crate::lexer::Token;
use crate::util::expr_and;
use crate::util::expr_block;
use crate::util::expr_bool;
use crate::util::expr_break;
use crate::util::expr_call;
use crate::util::expr_match;
use crate::util::expr_or;
use crate::util::expr_var;
use crate::util::expr_while;
use crate::util::parsed_expr_assoc;
use crate::util::parsed_expr_def;
use crate::util::parsed_expr_enum;
use crate::util::parsed_expr_var;
use crate::util::parsed_pat_enum;
use crate::util::parsed_pat_var;
use crate::util::pat_wild;
use crate::util::stmt_var;
use crate::util::ty_hole;

pub struct Parser<'a, I>
where
    I: Iterator<Item = Spanned<Token>>,
{
    input: &'a str,
    lexer: std::iter::Peekable<I>,
    pub report: Report,
}

enum Either<A, B> {
    A(A),
    B(B),
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Spanned<Token>>,
{
    pub fn new(input: &'a str, lexer: I) -> Self {
        let lexer = lexer.peekable();
        let report = Report::new();
        Self {
            input,
            lexer,
            report,
        }
    }

    fn peek(&mut self) -> Token {
        self.lexer.peek().map(|x| x.value).unwrap_or(Token::Eof)
    }

    fn next(&mut self) -> Option<Spanned<Token>> {
        self.lexer.next()
    }

    fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    #[inline(always)]
    fn or_else<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
        or: impl FnOnce(Span) -> T,
    ) -> Option<T> {
        if let Some(x) = f(self) {
            Some(x)
        } else {
            self.peek_span().map(or)
        }
    }

    fn optional(&mut self, token: Token) -> Option<Span> {
        if self.peek() == token {
            Some(self.advance())
        } else {
            None
        }
    }

    fn optional_any<const N: usize>(&mut self, tokens: [Token; N]) -> Option<Span> {
        let t = self.peek();
        if tokens.into_iter().any(|t1| t == t1) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn advance(&mut self) -> Span {
        self.next().unwrap().span
    }

    fn peek_span(&mut self) -> Option<Span> {
        self.lexer.peek().map(|t| t.span)
    }

    fn recover<const N: usize>(&mut self, tokens: [Token; N]) -> Option<Spanned<Token>> {
        loop {
            match self.peek() {
                t if t == Token::Eof => return None,
                t if tokens.contains(&t) => return self.next(),
                Token::SemiColon | Token::RBrace | Token::RBrack | Token::RParen => {
                    return None;
                }
                _ => self.next(),
            };
        }
    }

    #[inline(always)]
    fn recursive<T>(&mut self, f: impl Fn(&mut Self) -> Option<T>) -> Option<T> {
        let red_zone = 32 * 1024;
        let stack_size = 1024 * 1024;
        stacker::maybe_grow(
            red_zone,
            stack_size,
            #[inline(always)]
            || f(self),
        )
    }

    fn expect_any<const N: usize>(&mut self, tokens: [Token; N]) -> Option<Spanned<Token>> {
        match self.peek() {
            t if tokens.contains(&t) => Some(self.next().unwrap()),
            Token::Eof => {
                let s = self.peek_span().unwrap();
                self.report.err(
                    s,
                    "Unexpected end of file",
                    format!(
                        "Expected {}",
                        tokens
                            .into_iter()
                            .map(|t| format!("`{}`", t))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                None
            }
            Token::RBrace | Token::RBrack | Token::RParen => None,
            t => {
                let label = format!("Unexpected token `{t}`");
                let msg = format!(
                    "Expected {}",
                    tokens
                        .into_iter()
                        .map(|t| format!("`{}`", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let s = self.advance();
                self.report.err(s, label, msg);
                self.recover(tokens)
            }
        }
    }

    fn expect(&mut self, token: Token) -> Option<Span> {
        Some(self.expect_any([token])?.span)
    }

    fn sep_until<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
        sep: Token,
        r: Token,
    ) -> Option<Vec<T>> {
        let mut xs = vec![];
        if self.peek() != r {
            xs.push(f(self)?);
            while self.optional(sep).is_some() {
                if self.peek() == r {
                    println!("{r}");
                    break;
                } else {
                    xs.push(f(self)?);
                }
            }
        }
        Some(xs)
    }

    // Can end with a trailing comma
    fn sep_until_cond<T>(
        &mut self,
        sep: Token,
        f: impl Fn(&mut Self) -> Option<T>,
        c: impl Fn(&mut Self) -> bool,
    ) -> Option<Vec<T>> {
        let mut xs = vec![];
        if !c(self) {
            xs.push(f(self)?);
            while self.optional(sep).is_some() {
                if !c(self) {
                    xs.push(f(self)?);
                } else {
                    break;
                }
            }
        }
        Some(xs)
    }

    fn comma_sep_until_cond<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
        c: impl Fn(&mut Self) -> bool,
    ) -> Option<Vec<T>> {
        let mut xs = vec![];
        if !c(self) {
            xs.push(f(self)?);
            while self.optional(Token::Comma).is_some() {
                if !c(self) {
                    xs.push(f(self)?);
                } else {
                    break;
                }
            }
        }
        Some(xs)
    }

    fn sep<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
        sep: Token,
        l: Token,
        r: Token,
    ) -> Option<Vec<T>> {
        self.expect(l)?;
        let xs = self.sep_until(f, sep, r)?;
        self.expect(r)?;
        Some(xs)
    }

    fn comma_sep<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
        l: Token,
        r: Token,
    ) -> Option<Vec<T>> {
        self.sep(f, Token::Comma, l, r)
    }

    pub fn parse(&mut self) -> Program {
        let mut stmts = Vec::new();
        loop {
            if let Some(stmt) = self.stmt() {
                stmts.push(stmt);
            }
            while self.optional(Token::SemiColon).is_some() {}
            if self.peek() == Token::Eof {
                break;
            }
        }
        Program::new(stmts)
    }

    fn name(&mut self) -> Option<Name> {
        match self.peek() {
            Token::Name => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                let name = Name::new(t.span, v);
                Some(name)
            }
            _ => None,
        }
    }

    fn index(&mut self) -> Option<Index> {
        match self.peek() {
            Token::Int => {
                let t = self.next().unwrap();
                let v = self.text(t);
                match v.parse() {
                    Ok(v) => Some(Index::new(t.span, v)),
                    Err(e) => {
                        self.report
                            .err(t.span, format!("Invalid index `{}`", v), format!("{e}"));
                        None
                    }
                }
            }
            _ => None,
        }
    }

    pub fn stmt(&mut self) -> Option<Stmt> {
        let stmt = match self.peek() {
            Token::Def => Stmt::Def(self.stmt_def()?),
            Token::Impl => Stmt::Impl(self.stmt_impl()?),
            Token::Var => Stmt::Var(self.stmt_var()?),
            Token::Struct => Stmt::Struct(self.stmt_struct()?),
            Token::Enum => Stmt::Enum(self.stmt_enum()?),
            Token::Type => Stmt::Type(self.stmt_type()?),
            Token::Trait => Stmt::Trait(self.stmt_trait()?),
            _ => Stmt::Expr(self.stmt_expr()?),
        };
        Some(stmt)
    }

    pub fn stmt_type(&mut self) -> Option<StmtType> {
        self.expect(Token::Type)?;
        let name = self.name()?;
        let generics = self.generics()?;
        self.expect(Token::Eq)?;
        let ty = self.ty()?;
        self.expect(Token::SemiColon)?;
        Some(StmtType::new(name.span, name, generics, ty))
    }

    pub fn stmt_expr(&mut self) -> Option<Expr> {
        let expr = self.expr()?;
        if !matches!(expr, Expr::Block(..) | Expr::Match(..)) {
            self.expect(Token::SemiColon)?;
        }
        Some(expr)
    }

    pub fn unresolved_path(&mut self) -> Option<UnresolvedPath> {
        let mut segments = vec![];
        segments.push(self.unresolved_path_segment()?);
        while self.optional(Token::ColonColon).is_some() {
            segments.push(self.unresolved_path_segment()?);
        }
        Some(UnresolvedPath::new(segments))
    }

    pub fn unresolved_path_segment(&mut self) -> Option<(Name, Vec<Type>)> {
        let name = self.name()?;
        let ts = self.ty_args()?;
        Some((name, ts))
    }

    pub fn stmt_def(&mut self) -> Option<StmtDef> {
        let s0 = self.expect(Token::Def)?;
        let name = self.name()?;
        let generics = self.generics()?;
        let params = self.params()?;
        self.expect(Token::Colon)?;
        let ty = self.ty()?;
        let trs = self.where_clause(|this| matches!(this.peek(), Token::Eq | Token::LBrace))?;
        let expr = self.expr_or_block(Self::stmt_expr)?;
        Some(StmtDef::new(
            s0 + expr.span(),
            name,
            generics,
            trs,
            params,
            ty,
            Body::Expr(expr),
        ))
    }

    fn where_clause(&mut self, cond: impl Fn(&mut Self) -> bool) -> Option<Vec<Bound>> {
        if self.peek() == Token::Where {
            self.advance();
            self.comma_sep_until_cond(Self::bound, cond)
        } else {
            Some(vec![])
        }
    }

    pub fn stmt_trait(&mut self) -> Option<StmtTrait> {
        let s0 = self.expect(Token::Trait)?;
        let name = self.name()?;
        let generics = self.generics()?;
        let body = self.where_clause(|this| this.peek() == Token::LBrace)?;
        self.expect(Token::LBrace)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        loop {
            match self.peek() {
                Token::Def => defs.push(self.stmt_def_decl()?),
                Token::Type => tys.push(self.stmt_type_decl()?),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace)?;
        Some(StmtTrait::new(s0 + s1, name, generics, body, defs, tys))
    }

    fn stmt_def_decl(&mut self) -> Option<TraitDef> {
        let s0 = self.expect(Token::Def)?;
        let name = self.name()?;
        let generics = self.generics()?;
        let params = self.params()?;
        self.expect(Token::Colon)?;
        let ty = self.ty()?;
        let trs = self.where_clause(|this| this.peek() == Token::SemiColon)?;
        let s1 = self.expect(Token::SemiColon)?;
        Some(TraitDef::new(s0 + s1, name, generics, trs, params, ty))
    }

    fn stmt_type_decl(&mut self) -> Option<TraitType> {
        self.expect(Token::Type)?;
        let name = self.name()?;
        let generics = self.generics()?;
        self.expect(Token::SemiColon)?;
        Some(TraitType::new(name.span, name, generics))
    }

    pub fn stmt_impl(&mut self) -> Option<StmtImpl> {
        let s0 = self.expect(Token::Impl)?;
        let generics = self.generics()?;
        let head = self.bound()?;
        let body = self.where_clause(|this| this.peek() == Token::LBrace)?;
        self.expect(Token::LBrace)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        loop {
            match self.peek() {
                Token::Def => defs.push(self.stmt_def()?),
                Token::Type => tys.push(self.stmt_type()?),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace)?;
        Some(StmtImpl::new(s0 + s1, generics, head, body, defs, tys))
    }

    pub fn stmt_struct(&mut self) -> Option<StmtStruct> {
        let s0 = self.expect(Token::Struct)?;
        let name = self.name()?;
        let generics = self.generics()?;
        let fields = if self.peek() == Token::LParen {
            self.comma_sep(Self::ty_field, Token::LParen, Token::RParen)?
        } else {
            vec![]
        };
        self.expect(Token::SemiColon)?;
        Some(StmtStruct::new(s0, name, generics, fields))
    }

    fn ty_field(&mut self) -> Option<(Name, Type)> {
        let name = self.name()?;
        self.expect(Token::Colon)?;
        let ty = self.ty()?;
        Some((name, ty))
    }

    pub fn stmt_enum(&mut self) -> Option<StmtEnum> {
        let s0 = self.expect(Token::Enum)?;
        let name = self.name()?;
        let generics = self.generics()?;
        let variants = self.comma_sep(Self::ty_variant, Token::LBrace, Token::RBrace)?;
        Some(StmtEnum::new(s0, name, generics, variants))
    }

    fn ty_variant(&mut self) -> Option<(Name, Type)> {
        let name = self.name()?;
        let ty = if let Token::LParen = self.peek() {
            self.ty_paren()?
        } else {
            Type::Unresolved(UnresolvedPath::new(vec![(Name::from("Unit"), vec![])]))
        };
        Some((name, ty))
    }

    // Returns Either::A(T) for (T)
    // Returns Either::B((Span, Vec<T>)) for (), (T,), (T, T, ...)
    fn paren<T>(
        &mut self,
        f: impl Fn(&mut Self) -> Option<T>,
    ) -> Option<Either<T, (Span, Vec<T>)>> {
        let s0 = self.expect(Token::LParen)?;
        if let Some(s1) = self.optional(Token::RParen) {
            Some(Either::B((s0 + s1, vec![])))
        } else {
            let v = f(self)?;
            if self.optional(Token::RParen).is_some() {
                Some(Either::A(v))
            } else {
                let mut vs = vec![v];
                while self.optional(Token::Comma).is_some() {
                    if self.peek() == Token::RParen {
                        break;
                    } else {
                        vs.push(f(self)?);
                    }
                }
                let s1 = self.expect(Token::RParen)?;
                Some(Either::B((s0 + s1, vs)))
            }
        }
    }

    fn ty_paren(&mut self) -> Option<Type> {
        match self.paren(Self::ty)? {
            Either::A(t) => Some(t),
            Either::B((_, ts)) => Some(Type::Tuple(ts)),
        }
    }

    fn pat_paren(&mut self) -> Option<Pat> {
        match self.paren(Self::pat)? {
            Either::A(p) => Some(p),
            Either::B((s, ps)) => Some(Pat::Tuple(s, ty_hole(), ps)),
        }
    }

    fn expr_paren(&mut self) -> Option<Expr> {
        match self.paren(Self::expr)? {
            Either::A(e) => Some(e),
            Either::B((s, es)) => Some(Expr::Tuple(s, ty_hole(), es)),
        }
    }

    fn stmt_assoc_type(&mut self) -> Option<(Name, Type)> {
        self.expect(Token::Type)?;
        let name = self.name()?;
        self.expect(Token::Eq)?;
        let ty = self.ty()?;
        self.expect(Token::SemiColon)?;
        Some((name, ty))
    }

    fn stmt_var(&mut self) -> Option<StmtVar> {
        let s0 = self.expect(Token::Var)?;
        let name = self.name()?;
        let ty = if self.optional(Token::Colon).is_some() {
            self.ty()?
        } else {
            ty_hole()
        };
        self.expect(Token::Eq)?;
        let expr = self.expr()?;
        let s1 = self.expect(Token::SemiColon)?;
        Some(StmtVar::new(s0 + s1, name, ty, expr))
    }

    pub fn bound(&mut self) -> Option<Bound> {
        let path = self.unresolved_path()?;
        let s0 = path.segments.first().unwrap().0.span;
        let s1 = path.segments.last().unwrap().0.span;
        Some(Bound::Unresolved(s0 + s1, path))
    }

    fn params(&mut self) -> Option<Vec<Param>> {
        self.comma_sep(Self::param, Token::LParen, Token::RParen)
    }

    fn param(&mut self) -> Option<Param> {
        let name = self.name()?;
        let ty = self.ty_annot()?;
        Some(Param::new(name.span, name, ty))
    }

    fn generics(&mut self) -> Option<Vec<Name>> {
        if self.peek() == Token::LBrack {
            self.comma_sep(Self::name, Token::LBrack, Token::RBrack)
        } else {
            Some(vec![])
        }
    }

    fn ty_fun(&mut self) -> Option<Type> {
        self.expect(Token::Fun)?;
        let tys = self.comma_sep(Self::ty, Token::LParen, Token::RParen)?;
        self.expect(Token::Colon)?;
        let ty = self.ty()?;
        Some(Type::Fun(tys, Rc::new(ty)))
    }

    pub fn ty(&mut self) -> Option<Type> {
        self.recursive(|this| this.try_ty(Self::ty0))
    }

    pub fn try_ty(&mut self, f: impl Fn(&mut Self) -> Option<Type>) -> Option<Type> {
        self.or_else(f, |_| Type::Err)
    }

    pub fn ty0(&mut self) -> Option<Type> {
        match self.peek() {
            Token::Name => self.ty_name(),
            Token::Underscore => self.ty_underscore(),
            Token::LParen => self.ty_paren(),
            Token::Fun => self.ty_fun(),
            _ => None,
        }
    }

    fn ty_name(&mut self) -> Option<Type> {
        let path = self.unresolved_path()?;
        Some(Type::Unresolved(path))
    }

    fn ty_underscore(&mut self) -> Option<Type> {
        self.expect(Token::Underscore)?;
        Some(ty_hole())
    }

    fn ty_args(&mut self) -> Option<Vec<Type>> {
        if self.peek() == Token::LBrack {
            self.comma_sep(Self::ty, Token::LBrack, Token::RBrack)
        } else {
            Some(vec![])
        }
    }

    fn expr_field(&mut self) -> Option<(Name, Expr)> {
        let expr = self.expr1()?;
        match expr {
            Expr::Field(s, t, e, x) => Some((x.clone(), Expr::Field(s, t, e, x))),
            Expr::Unresolved(s, t, path) => {
                if path.segments.len() == 1 && path.segments[0].1.is_empty() && t == Type::Hole {
                    let x = path.segments.into_iter().next().unwrap().0;
                    if self.optional(Token::Eq).is_some() {
                        let expr = self.expr()?;
                        Some((x, expr))
                    } else {
                        Some((x.clone(), parsed_expr_var(s, x.to_string().as_str())))
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline(always)]
    fn expr_block(&mut self) -> Option<Expr> {
        let s0 = self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        let expr = loop {
            let stmt = match self.peek() {
                Token::Def => Stmt::Def(self.stmt_def()?),
                Token::Var => Stmt::Var(self.stmt_var()?),
                Token::Impl => Stmt::Impl(self.stmt_impl()?),
                Token::Trait => Stmt::Trait(self.stmt_trait()?),
                Token::Struct => Stmt::Struct(self.stmt_struct()?),
                Token::Enum => Stmt::Enum(self.stmt_enum()?),
                Token::SemiColon => {
                    self.next();
                    continue;
                }
                Token::RBrace => {
                    let s1 = self.advance();
                    let expr = Expr::Tuple(s0 + s1, ty_hole(), vec![]);
                    break Expr::Block(s0 + s1, ty_hole(), stmts, Rc::new(expr));
                }
                _ => {
                    let expr = self.expr()?;
                    if let Expr::Block(..) = expr {
                        // {{}}
                        if let Token::RBrace = self.peek() {
                            let s1 = self.advance();
                            break Expr::Block(s0 + s1, ty_hole(), stmts, Rc::new(expr));
                        } else {
                            Stmt::Expr(expr)
                        }
                    } else if self.expect(Token::SemiColon).is_some() {
                        while self.optional(Token::SemiColon).is_some() {}
                        Stmt::Expr(expr)
                    } else {
                        let s1 = self.expect(Token::RBrace)?;
                        break Expr::Block(s0 + s1, ty_hole(), stmts, Rc::new(expr));
                    }
                }
            };
            stmts.push(stmt);
        };
        Some(expr)
    }

    fn expr_array(&mut self) -> Option<Expr> {
        let es = self.comma_sep(Self::expr, Token::LBrack, Token::RBrack)?;
        Some(Expr::Array(Span::default(), ty_hole(), es))
    }

    fn expr_args(&mut self) -> Option<Vec<Expr>> {
        self.comma_sep(Self::expr, Token::LParen, Token::RParen)
    }

    fn expr_call(&mut self, e0: Expr) -> Option<Expr> {
        let es = self.expr_args()?;
        Some(Expr::Call(e0.span(), ty_hole(), Rc::new(e0), es))
    }

    fn tbinop(
        &mut self,
        lhs: Expr,
        t: Token,
        f: impl Fn(&mut Self) -> Option<Expr>,
        trait_name: impl Into<Name>,
        def_name: impl Into<Name>,
    ) -> Option<Expr> {
        let s = self.expect(t)?;
        let rhs = self.try_expr(f)?;
        let op = parsed_expr_assoc(s, trait_name, def_name);
        let s1 = lhs.span() + rhs.span();
        Some(Expr::Call(s1, ty_hole(), Rc::new(op), vec![lhs, rhs]))
    }

    fn def_binop(
        &mut self,
        lhs: Expr,
        t: Token,
        f: impl Fn(&mut Self) -> Option<Expr>,
        def_name: &'static str,
    ) -> Option<Expr> {
        let s = self.expect(t)?;
        let rhs = self.try_expr(f)?;
        let op = parsed_expr_def(s, def_name, vec![]);
        let s1 = lhs.span() + rhs.span();
        Some(Expr::Call(s1, ty_hole(), Rc::new(op), vec![lhs, rhs]))
    }

    fn unop(
        &mut self,
        t: Token,
        f: impl Fn(&mut Self) -> Option<Expr>,
        trait_name: impl Into<Name>,
        name: &'static str,
    ) -> Option<Expr> {
        let s = self.expect(t)?;
        let e = self.try_expr(f)?;
        let s1 = s + e.span();
        let op = parsed_expr_assoc(s, trait_name, name);
        Some(Expr::Call(s1, ty_hole(), Rc::new(op), vec![e]))
    }

    fn expr_dot(&mut self, expr: Expr) -> Option<Expr> {
        let s = self.expect(Token::Dot)?;
        match self.peek() {
            Token::Name => {
                let name = self.name()?;
                if let Token::LBrack | Token::LParen = self.peek() {
                    let ts = self.ty_args()?;
                    let es = self.expr_args()?;
                    let span = expr.span() + name.span;
                    let es = std::iter::once(expr).chain(es).collect::<Vec<_>>();
                    let e1 = parsed_expr_def(s, name, ts);
                    Some(Expr::Call(span, ty_hole(), Rc::new(e1), es))
                } else {
                    Some(Expr::Field(s, ty_hole(), Rc::new(expr), name))
                }
            }
            Token::Int => {
                let i = self.index()?;
                Some(Expr::Index(i.span, ty_hole(), Rc::new(expr), i))
            }
            _ => None,
        }
    }

    fn expr_assign(&mut self, lhs: Expr) -> Option<Expr> {
        self.expect(Token::Eq)?;
        let rhs = self.try_expr(Self::expr9)?;
        let s1 = lhs.span() + rhs.span();
        Some(Expr::Assign(s1, ty_hole(), Rc::new(lhs), Rc::new(rhs)))
    }

    // a?
    //
    // becomes
    //
    // match a {
    //    Some(x) => x,
    //    _ => return None,
    // }
    fn expr_question(&mut self, lhs: Expr) -> Option<Expr> {
        let s = self.expect(Token::Question)?;
        let p0 = parsed_pat_enum(s, "Option", "Some", parsed_pat_var(s, "x"));
        let e0 = parsed_expr_var(s, "x");
        let p1 = Pat::Wildcard(s, ty_hole());
        let e1 = Expr::Return(s, ty_hole(), Rc::new(parsed_expr_enum(s, "Option", "None")));
        let expr = Expr::Match(s, ty_hole(), Rc::new(lhs), vec![(p0, e0), (p1, e1)]);
        Some(expr)
    }

    fn expr_end(&mut self) -> bool {
        matches!(
            self.peek(),
            Token::SemiColon
                | Token::Comma
                | Token::RBrace
                | Token::RBrack
                | Token::RParen
                | Token::Eof
        )
    }

    fn expr_return(&mut self) -> Option<Expr> {
        let s = self.expect(Token::Return)?;
        let e = if self.expr_end() {
            Expr::Tuple(s, ty_hole(), vec![])
        } else {
            self.try_expr(Self::expr9)?
        };
        Some(Expr::Return(s, ty_hole(), Rc::new(e)))
    }

    fn expr_continue(&mut self) -> Option<Expr> {
        let s = self.expect(Token::Continue)?;
        Some(Expr::Continue(s, ty_hole()))
    }

    fn expr_break(&mut self) -> Option<Expr> {
        let s = self.expect(Token::Break)?;
        Some(Expr::Break(s, ty_hole()))
    }

    fn ty_annot(&mut self) -> Option<Type> {
        if self.optional(Token::Colon).is_some() {
            self.ty()
        } else {
            Some(ty_hole())
        }
    }

    fn expr_fun(&mut self) -> Option<Expr> {
        let s = self.expect(Token::Fun)?;
        let ps = self.params()?;
        let t = self.ty_annot()?;
        let e = self.expr_or_block(Self::expr)?;
        Some(Expr::Fun(s + e.span(), ty_hole(), ps, t, Rc::new(e)))
    }

    fn expr_query(&mut self) -> Option<Expr> {
        let qs = self.query_stmts()?;
        let s = qs.first().unwrap().span() + qs.last().unwrap().span();
        Some(Expr::Query(s, ty_hole(), qs))
    }

    fn end_of_query_stmt(&mut self) -> bool {
        matches!(
            self.peek(),
            Token::From
                | Token::Where
                | Token::Select
                | Token::Group
                | Token::Order
                | Token::Join
                | Token::With
                | Token::Over
                | Token::Into
                | Token::Compute
                | Token::SemiColon
                | Token::RBrace
                | Token::RBrack
                | Token::RParen
                | Token::Eof
        )
    }

    fn query_stmts(&mut self) -> Option<Vec<Query>> {
        let mut qs = Vec::new();
        loop {
            let q = match self.peek() {
                Token::From => self.query_from()?,
                Token::Where => self.query_where()?,
                Token::Select => self.query_select()?,
                Token::Group => self.query_group()?,
                Token::Order => self.query_order()?,
                Token::Join => self.query_join()?,
                Token::With => self.query_with()?,
                Token::Over => self.query_over()?,
                Token::Into => self.query_into()?,
                Token::Compute => self.query_compute()?,
                _ => break,
            };
            qs.push(q);
        }
        Some(qs)
    }

    fn name_in_expr(&mut self) -> Option<(Name, Expr)> {
        let name = self.name()?;
        self.expect(Token::In)?;
        let expr = self.expr()?;
        Some((name, expr))
    }

    fn query_from(&mut self) -> Option<Query> {
        let s = self.expect(Token::From)?;
        let xes = if self.peek() == Token::LBrace {
            self.comma_sep(Self::name_in_expr, Token::LBrace, Token::RBrace)?
        } else {
            self.comma_sep_until_cond(Self::name_in_expr, Self::end_of_query_stmt)?
        };
        Some(Query::From(s, ty_hole(), xes))
    }

    fn query_select(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::Select)?;
        let xes = if self.peek() == Token::LBrace {
            self.comma_sep(Self::expr_field, Token::LBrace, Token::RBrace)?
        } else {
            self.comma_sep_until_cond(Self::expr_field, Self::end_of_query_stmt)?
        };
        Some(Query::Select(s0, ty_hole(), xes))
    }

    fn name_eq_expr(&mut self) -> Option<(Name, Expr)> {
        let name = self.name()?;
        self.expect(Token::Eq)?;
        let expr = self.expr()?;
        Some((name, expr))
    }

    fn query_with(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::With)?;
        let xes = if self.peek() == Token::LBrace {
            self.comma_sep(Self::name_eq_expr, Token::LBrace, Token::RBrace)?
        } else {
            self.comma_sep_until_cond(Self::name_eq_expr, Self::end_of_query_stmt)?
        };
        Some(Query::With(s0, ty_hole(), xes))
    }

    fn query_compute(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::Compute)?;
        let aggs = if self.peek() == Token::LBrace {
            self.comma_sep(Self::agg, Token::LBrace, Token::RBrace)?
        } else {
            self.comma_sep_until_cond(Self::agg, Self::end_of_query_stmt)?
        };
        Some(Query::Compute(s0, ty_hole(), aggs))
    }

    fn agg(&mut self) -> Option<(Name, Expr, Expr)> {
        let x = self.name()?;
        self.expect(Token::Eq)?;
        let e0 = self.expr()?;
        self.expect(Token::Of)?;
        let e1 = self.expr()?;
        Some((x, e0, e1))
    }

    fn query_where(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::Where)?;
        let e = self.expr()?;
        let s = s0 + e.span();
        Some(Query::Where(s, ty_hole(), Rc::new(e)))
    }

    fn query_group(&mut self) -> Option<Query> {
        let s = self.expect(Token::Group)?;
        let xs = self.query_stmt_args(Self::name)?;
        self.expect(Token::LBrace)?;
        let qs = self.query_stmts()?;
        self.expect(Token::RBrace)?;
        Some(Query::Group(s, ty_hole(), xs, qs))
    }

    fn query_over(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::Over)?;
        let e = self.expr()?;
        self.expect(Token::LBrace)?;
        let qs = self.query_stmts()?;
        self.expect(Token::RBrace)?;
        let s = s0 + e.span();
        Some(Query::Over(s, ty_hole(), Rc::new(e), qs))
    }

    fn query_order(&mut self) -> Option<Query> {
        let s0 = self.expect(Token::Order)?;
        let os = self.query_stmt_args(Self::ordering)?;
        Some(Query::Order(s0, ty_hole(), os))
    }

    fn ordering(&mut self) -> Option<(Name, bool)> {
        let x = self.name()?;
        let k = self.optional(Token::Desc).is_some();
        Some((x, k))
    }

    fn query_join(&mut self) -> Option<Query> {
        let s = self.expect(Token::Join)?;
        let xes = self.query_stmt_args(Self::source)?;
        self.expect(Token::On)?;
        let e = self.expr()?;
        let t = ty_hole();
        Some(Query::Join(s, t, xes, Rc::new(e)))
    }

    fn source(&mut self) -> Option<(Name, Expr)> {
        let x = self.name()?;
        self.expect(Token::In)?;
        let e = self.expr()?;
        Some((x, e))
    }

    fn query_stmt_args<T>(&mut self, f: impl Fn(&mut Self) -> Option<T>) -> Option<Vec<T>> {
        if self.peek() == Token::LBrace {
            self.comma_sep(f, Token::LBrace, Token::RBrace)
        } else {
            self.comma_sep_until_cond(f, Self::end_of_query_stmt)
        }
    }

    fn query_into(&mut self) -> Option<Query> {
        let s = self.expect(Token::Into)?;
        let es = self.query_stmt_args(Self::expr)?;
        Some(Query::Into(s, ty_hole(), es))
    }

    fn expr_for(&mut self) -> Option<Expr> {
        self.expect(Token::For)?;
        let x = self.name()?;
        self.expect(Token::In)?;
        let e0 = self.expr()?;
        let e1 = self.expr_block()?;
        // {
        //   let it = @IntoIterator::into_iter(e0);
        //   while true {
        //     match it.next() {
        //       Option::Some(x) => e1,
        //       Option::None => break
        //     }
        //   }
        // }
        Some(expr_block(
            [stmt_var(
                "it",
                ty_hole(),
                expr_call(
                    parsed_expr_assoc(Span::default(), "IntoIterator", "into_iter"),
                    [e0],
                ),
            )],
            expr_while(
                expr_bool(true),
                expr_match(
                    expr_call(expr_var("next"), [expr_var("it")]),
                    [
                        (
                            parsed_pat_enum(
                                Span::default(),
                                "Option",
                                "Some",
                                parsed_pat_var(Span::default(), x),
                            ),
                            e1,
                        ),
                        (
                            parsed_pat_enum(Span::default(), "Option", "None", pat_wild()),
                            expr_break(),
                        ),
                    ],
                ),
            ),
        ))
    }

    fn expr_while(&mut self) -> Option<Expr> {
        self.expect(Token::While)?;
        let e = self.expr()?;
        let e1 = self.expr_block_or_expr()?;
        let s = e.span() + e1.span();
        Some(Expr::While(s, ty_hole(), Rc::new(e), Rc::new(e1)))
    }

    fn expr_match(&mut self) -> Option<Expr> {
        self.expect(Token::Match)?;
        let e = self.expr()?;
        let arms = self.comma_sep(Self::arm, Token::LBrace, Token::RBrace)?;
        Some(Expr::Match(e.span(), ty_hole(), Rc::new(e), arms))
    }

    fn expr_block_or_expr(&mut self) -> Option<Expr> {
        match self.expr_block()? {
            Expr::Block(_, _, ss, e) if ss.is_empty() => Some(e.as_ref().clone()),
            e => Some(e),
        }
    }

    fn expr_if(&mut self) -> Option<Expr> {
        self.expect(Token::If)?;
        let e = self.expr()?;
        let e1 = self.expr_block_or_expr()?;
        let e2 = if self.optional(Token::Else).is_some() {
            self.expr_block_or_expr()?
        } else {
            Expr::Tuple(Span::default(), ty_hole(), vec![])
        };
        let p1 = Pat::Bool(Span::default(), ty_hole(), true);
        let p2 = Pat::Wildcard(Span::default(), ty_hole());
        Some(Expr::Match(
            e.span() + e1.span() + e2.span(),
            ty_hole(),
            Rc::new(e),
            vec![(p1, e1), (p2, e2)],
        ))
    }

    fn expr_or_block(&mut self, f: impl Fn(&mut Self) -> Option<Expr>) -> Option<Expr> {
        if self.peek() == Token::LBrace {
            self.expr_block()
        } else {
            self.expect(Token::Eq)?;
            f(self)
        }
    }

    fn arm(&mut self) -> Option<(Pat, Expr)> {
        let p = self.pat()?;
        self.expect(Token::FatArrow)?;
        let e = self.expr()?;
        Some((p, e))
    }

    fn expr_annot(&mut self, e: Expr) -> Option<Expr> {
        self.expect(Token::Colon)?;
        let t = self.ty()?;
        Some(e.of(t))
    }

    #[inline(always)]
    fn expr0(&mut self) -> Option<Expr> {
        match self.peek() {
            Token::Int => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                Some(Expr::Int(t.span, ty_hole(), v))
            }
            Token::IntSuffix => {
                // let t = self.next().unwrap();
                // let v = self.text(t).to_owned();
                todo!()
                // let v0 = v0.to_owned();
                // let v1 = format!("postfix_{v1}");
                // let s = self.advance();
                // let e = parsed_expr_def(s, v1, vec![]);
                // Some(Expr::Call(
                //     s,
                //     ty_hole(),
                //     Rc::new(e),
                //     vec![Expr::Int(s, ty_hole(), v0)],
                // ))
            }
            Token::Float => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                Some(Expr::Float(t.span, ty_hole(), v))
            }
            Token::FloatSuffix => {
                todo!()
                // let v0 = v0.to_owned();
                // let v1 = format!("postfix_{v1}");
                // let s = self.advance();
                // let e = parsed_expr_def(s, v1, vec![]);
                // Some(Expr::Call(
                //     s,
                //     ty_hole(),
                //     Rc::new(e),
                //     vec![Expr::Float(s, ty_hole(), v0)],
                // ))
            }
            Token::True => {
                let s = self.advance();
                Some(Expr::Bool(s, ty_hole(), true))
            }
            Token::String => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                Some(Expr::String(t.span, ty_hole(), v))
            }
            Token::False => {
                let s = self.advance();
                Some(Expr::Bool(s, ty_hole(), false))
            }
            Token::Name => self.expr_path(),
            Token::LParen => self.expr_paren(),
            Token::LBrack => self.expr_array(),
            Token::LBrace => self.expr_block(),
            Token::Match => self.expr_match(),
            Token::If => self.expr_if(),
            Token::From => self.expr_query(),
            Token::For => self.expr_for(),
            Token::While => self.expr_while(),
            _ => None,
        }
    }

    #[inline(always)]
    fn expr1(&mut self) -> Option<Expr> {
        let mut expr = self.try_expr(Self::expr0)?;
        loop {
            expr = match self.peek() {
                Token::LParen => self.expr_call(expr)?,
                Token::Dot => self.expr_dot(expr)?,
                Token::Question => self.expr_question(expr)?,
                _ => break Some(expr),
            }
        }
    }

    #[inline(always)]
    fn expr2(&mut self) -> Option<Expr> {
        match self.peek() {
            Token::Minus => self.unop(Token::Minus, Self::expr2, "Neg", "neg"),
            Token::Not => self.unop(Token::Not, Self::expr2, "Not", "not"),
            _ => self.try_expr(Self::expr1),
        }
    }

    #[inline(always)]
    fn expr3(&mut self) -> Option<Expr> {
        let mut expr = self.try_expr(Self::expr2)?;
        loop {
            expr = match self.peek() {
                Token::Colon => self.expr_annot(expr)?,
                _ => break Some(expr),
            }
        }
    }

    #[inline(always)]
    fn expr4(&mut self) -> Option<Expr> {
        let mut expr = self.try_expr(Self::expr3)?;
        loop {
            expr = match self.peek() {
                Token::Star => self.tbinop(expr, Token::Star, Self::expr3, "Mul", "mul")?,
                Token::Slash => self.tbinop(expr, Token::Slash, Self::expr3, "Div", "div")?,
                _ => break Some(expr),
            }
        }
    }

    #[inline(always)]
    fn expr5(&mut self) -> Option<Expr> {
        let mut expr = self.try_expr(Self::expr4)?;
        loop {
            expr = match self.peek() {
                Token::Plus => self.tbinop(expr, Token::Plus, Self::expr4, "Add", "add")?,
                Token::Minus => self.tbinop(expr, Token::Minus, Self::expr4, "Sub", "sub")?,
                _ => break Some(expr),
            }
        }
    }

    fn temp(span: Span, id: usize) -> (Expr, impl FnOnce(Expr) -> Stmt) {
        let name = Name::from(format!("_{id}"));
        let var = parsed_expr_var(span, name.clone());
        let stmt = move |e| Stmt::Var(StmtVar::new(span, name, ty_hole(), e));
        (var, stmt)
    }

    // f() == g() => f() == g()
    //
    // f() == g() == h() => { val _0 = f();
    //                        val _1 = g();
    //                        _0 == _1 && _1 == h() }
    //
    // f() == g() == h() == i() => { val _0 = f();
    //                               val _1 = g();
    //                               _0 == _1 &&
    //                               { val _2 = h();
    //                                 _1 == _2 } }
    #[inline(always)]
    fn expr6(&mut self) -> Option<Expr> {
        let lhs = self.try_expr(Self::expr5)?;
        let op = match self.peek() {
            Token::EqEq => parsed_expr_assoc(self.advance(), "PartialEq", "eq"),
            Token::NotEq => parsed_expr_assoc(self.advance(), "PartialEq", "ne"),
            Token::Lt => parsed_expr_assoc(self.advance(), "PartialOrd", "lt"),
            Token::Gt => parsed_expr_assoc(self.advance(), "PartialOrd", "gt"),
            Token::Le => parsed_expr_assoc(self.advance(), "PartialOrd", "le"),
            Token::Ge => parsed_expr_assoc(self.advance(), "PartialOrd", "ge"),
            _ => return Some(lhs),
        };
        let rhs = self.try_expr(Self::expr5)?;
        let (lhs_var, lhs_stmt) = Self::temp(lhs.span(), 0);
        let (rhs_var, rhs_stmt) = Self::temp(rhs.span(), 1);
        let (expr, n) = self.expr6_rec(rhs_var.clone(), 2)?;
        let t = ty_hole();
        if n > 2 {
            let span = lhs.span() + expr.span();
            let expr_op = Expr::Call(span, t.clone(), Rc::new(op), vec![lhs_var, rhs_var]);
            let expr_and = expr_and(expr_op, expr).with_span(span);
            let stmts = vec![lhs_stmt(lhs), rhs_stmt(rhs)];
            Some(Expr::Block(span, t, stmts, Rc::new(expr_and)))
        } else {
            Some(Expr::Call(expr.span(), t, Rc::new(op), vec![lhs, rhs]))
        }
    }

    fn expr6_rec(&mut self, lhs: Expr, id: usize) -> Option<(Expr, usize)> {
        let op = match self.peek() {
            Token::EqEq => parsed_expr_assoc(self.advance(), "PartialEq", "eq"),
            Token::NotEq => parsed_expr_assoc(self.advance(), "PartialEq", "ne"),
            Token::Lt => parsed_expr_assoc(self.advance(), "PartialOrd", "lt"),
            Token::Gt => parsed_expr_assoc(self.advance(), "PartialOrd", "gt"),
            Token::Le => parsed_expr_assoc(self.advance(), "PartialOrd", "le"),
            Token::Ge => parsed_expr_assoc(self.advance(), "PartialOrd", "ge"),
            _ => return Some((lhs, id)),
        };
        let rhs = self.try_expr(Self::expr5)?;
        let span = lhs.span() + rhs.span();
        let (rhs_var, rhs_stmt) = Self::temp(span, id);
        let (expr, n) = self.expr6_rec(rhs_var.clone(), id + 1)?;
        let t = ty_hole();
        if n > id + 1 {
            let span = lhs.span() + expr.span();
            let expr_op = Expr::Call(span, ty_hole(), Rc::new(op), vec![lhs, rhs_var]);
            let expr_and = expr_and(expr_op, expr).with_span(span);
            let stmts = vec![rhs_stmt(rhs)];
            Some((Expr::Block(span, t, stmts, Rc::new(expr_and)), n))
        } else {
            Some((Expr::Call(op.span(), t, Rc::new(op), vec![lhs, rhs]), n))
        }
    }

    #[inline(always)]
    fn expr7(&mut self) -> Option<Expr> {
        let mut expr = self.try_expr(Self::expr6)?;
        loop {
            expr = match self.peek() {
                Token::And => {
                    let s = self.advance();
                    let rhs = self.try_expr(Self::expr6)?;
                    expr_and(expr, rhs).with_span(s)
                }
                Token::Or => {
                    let s = self.advance();
                    let rhs = self.try_expr(Self::expr6)?;
                    expr_or(expr, rhs).with_span(s)
                }
                _ => break Some(expr),
            }
        }
    }

    #[inline(always)]
    fn expr8(&mut self) -> Option<Expr> {
        let expr = self.try_expr(Self::expr7)?;
        match self.peek() {
            Token::DotDot => self.def_binop(expr, Token::DotDot, Self::expr7, "range"),
            _ => Some(expr),
        }
    }

    #[inline(always)]
    fn expr9(&mut self) -> Option<Expr> {
        let expr = self.try_expr(Self::expr8)?;
        match self.peek() {
            Token::Eq => self.expr_assign(expr),
            _ => Some(expr),
        }
    }

    #[inline(always)]
    fn expr10(&mut self) -> Option<Expr> {
        match self.peek() {
            Token::Return => self.expr_return(),
            Token::Continue => self.expr_continue(),
            Token::Break => self.expr_break(),
            Token::Fun => self.expr_fun(),
            _ => self.try_expr(Self::expr9),
        }
    }

    #[inline(always)]
    fn try_expr(&mut self, f: impl Fn(&mut Self) -> Option<Expr>) -> Option<Expr> {
        self.or_else(
            #[inline(always)]
            |this| f(this),
            |s| Expr::Err(s, ty_hole()),
        )
    }

    pub fn expr(&mut self) -> Option<Expr> {
        self.recursive(
            #[inline(always)]
            |this| this.try_expr(Self::expr10),
        )
    }

    pub fn pat0(&mut self) -> Option<Pat> {
        match self.peek() {
            Token::Name => self.pat_path(),
            Token::LParen => self.pat_paren(),
            Token::Underscore => self.pat_underscore(),
            Token::Int => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                Some(Pat::Int(t.span, ty_hole(), v))
            }
            Token::String => {
                let t = self.next().unwrap();
                let v = self.text(t).to_owned();
                Some(Pat::String(t.span, ty_hole(), v))
            }
            _ => None,
        }
    }

    pub fn pat1(&mut self) -> Option<Pat> {
        let p = self.pat0()?;
        match self.peek() {
            Token::Colon => {
                self.advance();
                let t = self.ty()?;
                Some(p.with_ty(t))
            }
            _ => Some(p),
        }
    }

    pub fn try_pat(&mut self, f: impl Fn(&mut Self) -> Option<Pat>) -> Option<Pat> {
        self.or_else(f, |s| Pat::Err(s, ty_hole()))
    }

    pub fn pat(&mut self) -> Option<Pat> {
        self.recursive(|this| this.try_pat(Self::pat1))
    }

    fn pat_underscore(&mut self) -> Option<Pat> {
        let s = self.expect(Token::Underscore)?;
        Some(Pat::Wildcard(s, ty_hole()))
    }

    fn pat_args(&mut self) -> Option<Option<Vec<PatArg>>> {
        if self.peek() == Token::LParen {
            Some(Some(self.comma_sep(
                Self::pat_arg,
                Token::LParen,
                Token::RParen,
            )?))
        } else {
            Some(None)
        }
    }

    fn pat_arg(&mut self) -> Option<PatArg> {
        let mut p = self.pat()?;
        if let Pat::Unresolved(_, Type::Hole, path, args) = &mut p {
            if path.segments.len() == 1
                && path.segments[0].1.is_empty()
                && args.is_none()
                && self.optional(Token::Eq).is_some()
            {
                let x = path.segments.pop().unwrap().0;
                let p = self.pat()?;
                return Some(PatArg::Named(x, p));
            }
        }
        Some(PatArg::Unnamed(p))
    }

    fn pat_path(&mut self) -> Option<Pat> {
        let path = self.unresolved_path()?;
        let args = self.pat_args()?;
        Some(Pat::Unresolved(Span::default(), ty_hole(), path, args))
    }

    fn expr_path(&mut self) -> Option<Expr> {
        let path = self.unresolved_path()?;
        Some(Expr::Unresolved(Span::default(), ty_hole(), path))
    }
}
