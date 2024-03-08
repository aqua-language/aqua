//! https://dl.acm.org/doi/pdf/10.1145/947902.947905
//!
//! The standard method of adding error recovery to a recursive descent
//! parser has been well described by Wirth in [4] and by Amman in [I].
//! With this method, one modifies the parsing procedure P corresponding to
//! each syntactic class (nonterminal symbol) S as follows.
//!
//! a) Add a parameter, **followers**, whose value includes the set of input
//! symbols which may legally follow this instance of S.
//! b) On entry to P, test that the current input symbol, **sym**, may legally
//! start an instance of S; if it can't, report an error and read input
//! symbols until reaching a legal starter or follower of S.
//! c) On exit from P, test that **sym** may legally follow this instance of S;
//! if it can't, report an error and read input symbols until reaching a
//! legal follower of S.
//! d) Replace each call to any other parsing procedure Q corresponding to
//! a syntactic class T by the call
//! Q([...] + followers)
//! where [...] is the set of symbols which P expects to follow this
//! instance of T.
//!
//! ------------------------
//! In this implementation:
//! * We support error recovery for all nodes
//! * Expr, Stmt, Type, and Pat can become error nodes, which means that
//!   they need to know when they should stop parsing. In other words, we need
//!   to inform them of their follow set. Other nodes don't need to know this.
use std::rc::Rc;

use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Query;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::ast::UnresolvedPatField;
use crate::diag::Report;
use crate::diag::Sources;
use crate::lexer::Lexer;
use crate::lexer::Span;
use crate::lexer::Spanned;
use crate::lexer::Token;
use crate::util::trim;

pub struct Parser<'a, I>
where
    I: Iterator<Item = Spanned<Token>>,
{
    input: &'a str,
    lexer: std::iter::Peekable<I>,
    pub report: Report,
    delims: Vec<Spanned<Token>>,
}

impl Token {
    fn infix_bp(self) -> Option<(u8, u8)> {
        let bp = match self {
            Token::Eq => (1, 2),
            Token::DotDot => (2, 3),
            Token::And | Token::Or => (3, 4),
            Token::EqEq | Token::NotEq | Token::Lt | Token::Gt | Token::Le | Token::Ge => (4, 5),
            Token::Plus | Token::Minus => (5, 6),
            Token::Star | Token::Slash => (6, 7),
            _ => return None,
        };
        Some(bp)
    }
    fn prefix_bp(self) -> Option<((), u8)> {
        let bp = match self {
            Token::Not | Token::Minus => ((), 8),
            _ => return None,
        };
        Some(bp)
    }
    fn postfix_bp(self) -> Option<(u8, ())> {
        let bp = match self {
            Token::Question | Token::LParen | Token::Dot | Token::Colon => (9, ()),
            _ => return None,
        };
        Some(bp)
    }
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
            delims: Vec::new(),
        }
    }

    // Utility functions

    /// Peek at the next token
    fn peek(&mut self) -> Spanned<Token> {
        self.lexer.peek().cloned().unwrap()
    }

    /// Get the next token
    fn next(&mut self) -> Spanned<Token> {
        self.lexer.next().unwrap()
    }

    /// Skip the next token
    fn skip(&mut self) {
        self.next();
    }

    /// Get the text of a token
    fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    #[allow(unused)]
    fn debug(&mut self, follow: Token) {
        println!("peek: {:?}", self.peek());
        println!("follow: {:?}", follow);
    }

    #[allow(unused)]
    fn debug2(&mut self, first: Token, follow: Token) {
        println!("peek: {:?}", self.peek());
        println!("first: {:?}", first);
        println!("follow: {:?}", follow);
    }

    // Error recovery takes closing/opening braces into account
    // 1. (+ [)]) => Recover at the second ) and report an error: Unexpected token +
    // 2. <Open>  => Recover at the end of the file and report an error: Unmatched <Open>
    fn recover(&mut self, first: Token, follow: Token, peek: bool) -> Result<Spanned<Token>, Span> {
        loop {
            let t = self.peek();
            match t.v {
                _ if first.contains(t.v) && self.delims.is_empty() => {
                    return if peek { Ok(t) } else { Ok(self.next()) }
                }
                // TODO: Handle what happens when the follow set contains a closing token
                _ if follow.contains(t.v) && self.delims.is_empty() => {
                    return Err(t.s);
                }
                _ if t.v == Token::Eof => {
                    if !self.delims.is_empty() {
                        self.delims.drain(..).for_each(|t| match t.v {
                            Token::LParen => {
                                self.report.err(t.s, "Unmatched `(`", "expected `)`");
                            }
                            Token::LBrace => {
                                self.report.err(t.s, "Unmatched `{`", "expected `}`");
                            }
                            Token::LBrack => {
                                self.report.err(t.s, "Unmatched `[`", "expected `]`");
                            }
                            _ => unreachable!(),
                        });
                    }
                    return Err(t.s);
                }
                Token::LBrace | Token::LParen | Token::LBrack => {
                    self.skip();
                    self.delims.push(t)
                }
                Token::RBrace if self.delims.last().is_some_and(|t1| t1.v == Token::LBrace) => {
                    self.skip();
                    self.delims.pop();
                }
                Token::RParen if self.delims.last().is_some_and(|t1| t1.v == Token::LParen) => {
                    self.skip();
                    self.delims.pop();
                }
                Token::RBrack if self.delims.last().is_some_and(|t1| t1.v == Token::LBrack) => {
                    self.skip();
                    self.delims.pop();
                }
                _ => self.skip(),
            }
        }
    }

    fn expect(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if first.contains(t.v) {
            Ok(self.next())
        } else {
            self.report
                .err(t.s, format!("Unexpected token `{}`", t.v), first.expected());
            self.recover(first, follow, false)
        }
    }

    fn start(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if first.contains(t.v) {
            Ok(t)
        } else {
            self.report
                .err(t.s, format!("Unexpected token `{}`", t.v), first.expected());
            self.recover(first, follow, true)
        }
    }

    fn optional<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        first: Token,
        follow: Token,
    ) -> Result<Option<Spanned<T>>, Span> {
        let t = self.start(first | follow, follow)?;
        if first.contains(t.v) {
            Ok(Some(f(self, follow)?))
        } else {
            Ok(None)
        }
    }

    /// Consume the next token if it is `token`
    fn eat(&mut self, token: Token, follow: Token) -> Result<bool, Span> {
        let t = self.start(token | follow, follow)?;
        if token.contains(t.v) {
            self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn seq<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        sep: Token,
        first: Token,
        follow: Token,
    ) -> Result<Option<Spanned<Vec<T>>>, Span> {
        self.optional(
            |this, follow| this.seq_nonempty(&f, sep, first, follow),
            first,
            follow,
        )
    }

    fn seq_nonempty<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        sep: Token,
        first: Token,
        follow: Token,
    ) -> Result<Spanned<Vec<T>>, Span> {
        let x = f(self, follow | sep)?;
        let s0 = x.s;
        let mut s1 = x.s;
        let mut xs = vec![x.v];
        loop {
            if self.eat(sep, follow)? {
                if let Some(t) = self.optional(&f, first, follow | sep)? {
                    s1 = t.s;
                    xs.push(t.v);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(Spanned::new(s0 + s1, xs))
    }

    fn repeat<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        first: Token,
        follow: Token,
    ) -> Result<Option<Spanned<Vec<T>>>, Span> {
        let t = self.start(first | follow, follow)?;
        if first.contains(t.v) {
            let x = f(self, follow)?;
            let s0 = t.s;
            let mut s1 = x.s;
            let mut xs = vec![x.v];
            loop {
                let t = self.start(first | follow, follow)?;
                if first.contains(t.v) {
                    let x = f(self, follow)?;
                    s1 = x.s;
                    xs.push(x.v);
                } else {
                    break Ok(Some(Spanned::new(s0 + s1, xs)));
                }
            }
        } else {
            Ok(None)
        }
    }

    // )() => Skip the first
    fn group<T>(
        &mut self,
        open: Token,
        close: Token,
        f: impl Fn(&mut Self, Token) -> Result<Option<Spanned<T>>, Span>,
        follow: Token,
    ) -> Result<Spanned<Option<T>>, Span> {
        let t0 = self.expect(open, follow)?;
        let x = f(self, follow | close)?;
        let t1 = self.expect(close, follow)?;
        Ok(Spanned::new(t0.s + t1.s, x.map(|x| x.v)))
    }

    fn brace<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Option<Spanned<T>>, Span>,
        follow: Token,
    ) -> Result<Spanned<Option<T>>, Span> {
        self.group(Token::LBrace, Token::RBrace, f, follow)
    }

    fn paren<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Option<Spanned<T>>, Span>,
        follow: Token,
    ) -> Result<Spanned<Option<T>>, Span> {
        self.group(Token::LParen, Token::RParen, f, follow)
    }

    fn brack<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Option<Spanned<T>>, Span>,
        follow: Token,
    ) -> Result<Spanned<Option<T>>, Span> {
        self.group(Token::LBrack, Token::RBrack, f, follow)
    }

    fn paren_seq<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        first: Token,
        follow: Token,
    ) -> Result<Spanned<Vec<T>>, Span> {
        self.paren(|p, follow| p.seq(&f, Token::Comma, first, follow), follow)
            .map(|x| x.flatten())
    }

    pub fn parse<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let t = f(self, Token::Eof);
        self.expect(Token::Eof, Token::Eof).ok();
        t.ok().map(|x| x.v)
    }

    // Terminals

    fn name(&mut self, follow: Token) -> Result<Spanned<Name>, Span> {
        let t = self.expect(Token::Name, follow)?;
        let v = self.text(t).to_owned();
        let x = Name::new(t.s, v);
        Ok(Spanned::new(t.s, x))
    }

    fn index(&mut self, follow: Token) -> Result<Spanned<Index>, Span> {
        let t = self.expect(Token::Int, follow)?;
        let v = self.text(t);
        match v.parse() {
            Ok(v) => Ok(Spanned::new(t.s, Index::new(t.s, v))),
            Err(e) => {
                self.report
                    .err(t.s, format!("Invalid index `{}`", v), e.to_string());
                Err(t.s)
            }
        }
    }

    // The parser
    pub fn program(&mut self, follow: Token) -> Result<Spanned<Program>, Span> {
        let mut stmts = Vec::new();
        let s0 = self.peek().s;
        let s1 = loop {
            let t = self.start(Stmt::FIRST | Token::Eof, follow)?;
            if t.v == Token::Eof {
                break t.s;
            }
            let s = self.stmt(follow)?;
            stmts.push(s.v);
            while self.eat(Token::SemiColon, follow | Stmt::FIRST)? {}
        };
        let s = s0 + s1;
        Ok(Spanned::new(s, Program::new(stmts)))
    }

    pub fn stmt(&mut self, follow: Token) -> Result<Spanned<Stmt>, Span> {
        self.stmt_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Stmt::Err(s))))
    }

    pub fn stmt_fallible(&mut self, follow: Token) -> Result<Spanned<Stmt>, Span> {
        let t = self.start(Stmt::FIRST, follow)?;
        match t.v {
            Token::Def => {
                let s = self.stmt_def(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Def(s.v)))
            }
            Token::Struct => {
                let s = self.stmt_struct(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Struct(s.v)))
            }
            Token::Enum => {
                let s = self.stmt_enum(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Enum(s.v)))
            }
            Token::Impl => {
                let s = self.stmt_impl(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Impl(s.v)))
            }
            Token::Type => {
                let s = self.stmt_type(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Type(s.v)))
            }
            Token::Var => {
                let s = self.stmt_var(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Var(s.v)))
            }
            _ => {
                let s = self.stmt_expr(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Expr(s.v)))
            }
        }
    }

    fn stmt_def(&mut self, follow: Token) -> Result<Spanned<StmtDef>, Span> {
        let t0 = self.expect(Token::Def, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen)?;
        let xts = self.params(follow)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::Where | Token::Eq)?;
        let bs = self.where_clause(follow | Token::Eq)?;
        self.expect(Token::Eq, follow)?;
        let e = self.stmt_expr(follow)?;
        let s = t0.s + e.s;
        Ok(Spanned::new(
            s,
            StmtDef::new(s, x.v, gs, bs, xts.v, t.v, Body::Expr(e.v)),
        ))
    }

    fn stmt_struct(&mut self, follow: Token) -> Result<Spanned<StmtStruct>, Span> {
        let t0 = self.expect(Token::Struct, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen | Token::SemiColon)?;
        let xts = self
            .optional(
                |p, follow| p.ty_fields(follow | Token::SemiColon),
                Token::LParen,
                follow | Token::SemiColon,
            )?
            .map(|x| x.v)
            .unwrap_or_default();
        self.expect(Token::SemiColon, follow)?;
        let s = t0.s + x.s;
        Ok(Spanned::new(s, StmtStruct::new(s, x.v, gs, xts)))
    }

    fn stmt_enum(&mut self, follow: Token) -> Result<Spanned<StmtEnum>, Span> {
        let t0 = self.expect(Token::Enum, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LBrace)?;
        let xts = self.ty_variants(follow)?;
        let s = t0.s + x.s;
        Ok(Spanned::new(s, StmtEnum::new(s, x.v, gs, xts)))
    }

    fn stmt_type(&mut self, follow: Token) -> Result<Spanned<StmtType>, Span> {
        let t0 = self.expect(Token::Type, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::Eq)?;
        self.expect(Token::Eq, follow)?;
        let t = self.ty(follow | Token::SemiColon)?;
        let s = t0.s + t.s;
        self.expect(Token::SemiColon, follow)?;
        Ok(Spanned::new(s, StmtType::new(s, x.v, gs, t.v)))
    }

    fn stmt_var(&mut self, follow: Token) -> Result<Spanned<StmtVar>, Span> {
        let t0 = self.expect(Token::Var, follow)?;
        let x = self.name(follow)?;
        let t = self
            .optional(Self::ty_annot, Token::Colon, follow | Token::Eq)?
            .map(|x| x.v)
            .unwrap_or(Type::Hole);
        self.expect(Token::Eq, follow)?;
        let e = self.stmt_expr(follow)?;
        let s = t0.s + e.s;
        Ok(Spanned::new(s, StmtVar::new(s, x.v, t, e.v)))
    }

    fn stmt_impl(&mut self, follow: Token) -> Result<Spanned<StmtImpl>, Span> {
        let t0 = self.expect(Token::Impl, follow)?;
        let gs = self.generics(follow | Token::Name)?;
        let b = self.bound(follow | Token::Where | Token::LBrace)?;
        let bs = self.where_clause(follow | Token::LBrace)?;
        self.expect(Token::LBrace, follow)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        loop {
            let t = self.start(Token::RBrace | Token::Def | Token::Type, follow)?;
            match t.v {
                Token::Def => defs.push(self.stmt_def(follow)?.v),
                Token::Type => tys.push(self.stmt_type(follow)?.v),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace, follow)?;
        let s = t0.s + s1.s;
        Ok(Spanned::new(s, StmtImpl::new(s, gs, b.v, bs, defs, tys)))
    }

    fn stmt_expr(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        let expr = self.expr(follow)?;
        if expr.v.is_braced() {
            Ok(Spanned::new(expr.s, expr.v))
        } else {
            self.expect(Token::SemiColon, follow)?;
            Ok(Spanned::new(expr.s, expr.v))
        }
    }

    fn ty_variants(&mut self, follow: Token) -> Result<Vec<(Name, Type)>, Span> {
        self.brace(
            |p, follow| p.seq(Self::ty_variant, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| x.v.unwrap_or_default())
    }

    fn ty_variant(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let x = self.name(follow)?;
        if self.start(Token::LParen | follow, follow)?.v == Token::LParen {
            self.skip();
            let t = self.ty(follow | Token::RParen)?;
            self.expect(Token::RParen, follow)?;
            let s = x.s + t.s;
            Ok(Spanned::new(s, (x.v, t.v)))
        } else {
            let s = x.s;
            Ok(Spanned::new(s, (x.v, Type::Tuple(vec![]))))
        }
    }

    fn where_clause(&mut self, follow: Token) -> Result<Vec<Bound>, Span> {
        self.optional(
            |this, follow| {
                let t0 = this.expect(Token::Where, follow)?;
                let xs = this.bounds(follow)?;
                if let Some(xs) = xs {
                    Ok(Spanned::new(t0.s + xs.s, xs.v))
                } else {
                    Ok(Spanned::new(t0.s, vec![]))
                }
            },
            Token::Where,
            follow,
        )
        .map(|x| x.map(|x| x.v).unwrap_or_default())
    }

    fn bounds(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Bound>>>, Span> {
        self.seq(Self::bound, Token::Comma, Token::Name, follow)
    }

    fn bound(&mut self, follow: Token) -> Result<Spanned<Bound>, Span> {
        let x = self.path(follow)?;
        Ok(Spanned::new(x.s, Bound::Unresolved(x.s, x.v)))
    }

    fn generics(&mut self, follow: Token) -> Result<Vec<Name>, Span> {
        self.optional(
            |this, follow| {
                this.brack(
                    |p, follow| p.seq(Self::name, Token::Comma, Token::Name, follow),
                    follow,
                )
            },
            Token::LBrack,
            follow,
        )
        .map(|x| x.and_then(|x| x.v).unwrap_or_default())
    }

    fn params(&mut self, follow: Token) -> Result<Spanned<Vec<Param>>, Span> {
        self.paren_seq(Self::param, Token::Name, follow)
    }

    fn param(&mut self, follow: Token) -> Result<Spanned<Param>, Span> {
        let name = self.name(follow)?;
        self.expect(Token::Colon, follow)?;
        let ty = self.ty(follow)?;
        let s = name.s + ty.s;
        Ok(Spanned::new(s, Param::new(s, name.v, ty.v)))
    }

    fn inferred_params(&mut self, follow: Token) -> Result<Spanned<Vec<Param>>, Span> {
        self.paren_seq(Self::inferred_param, Token::Name, follow)
    }

    fn inferred_param(&mut self, follow: Token) -> Result<Spanned<Param>, Span> {
        let x = self.name(follow)?;
        if self.start(follow | Token::Colon, follow)?.v == Token::Colon {
            self.skip();
            let t = self.ty(follow)?;
            let s = x.s + t.s;
            Ok(Spanned::new(s, Param::new(s, x.v, t.v)))
        } else {
            let s = x.s;
            Ok(Spanned::new(s, Param::new(s, x.v, Type::Hole)))
        }
    }

    fn path(&mut self, follow: Token) -> Result<Spanned<Path>, Span> {
        let xs = self.seq_nonempty(Self::path_segment, Token::ColonColon, Token::Name, follow)?;
        Ok(Spanned::new(xs.s, Path::new(xs.v)))
    }

    fn ty_args(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Type>>>, Span> {
        self.optional(
            |this, follow| this.brack(Self::tys, follow).map(|x| x.flatten()),
            Token::LBrack,
            follow,
        )
    }

    fn tys(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Type>>>, Span> {
        self.seq(Self::ty, Token::Comma, Type::FIRST, follow)
    }

    fn exprs(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Expr>>>, Span> {
        self.seq(Self::expr, Token::Comma, Expr::FIRST, follow)
    }

    fn ty_fields(&mut self, follow: Token) -> Result<Spanned<Vec<(Name, Type)>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::ty_field, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn ty_field(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let name = self.name(follow)?;
        self.expect(Token::Colon, follow)?;
        let ty = self.ty(follow)?;
        Ok(Spanned::new(name.s + ty.s, (name.v, ty.v)))
    }

    fn path_segment(&mut self, follow: Token) -> Result<Spanned<(Name, Vec<Type>)>, Span> {
        let name = self.name(follow)?;
        let args = self.ty_args(follow)?;
        if let Some(args) = args {
            Ok(Spanned::new(name.s + args.s, (name.v, args.v)))
        } else {
            Ok(Spanned::new(name.s, (name.v, vec![])))
        }
    }

    fn ty(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.ty_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Type::Err)))
    }

    fn ty_fallible(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        let t = self.start(Type::FIRST, follow)?;
        let lhs = match t.v {
            Token::Name => {
                let path = self.path(follow)?;
                Type::Unresolved(path.v)
            }
            Token::LParen => {
                let t = self.ty_tuple(follow)?;
                if t.v.len() == 1 {
                    t.v.into_iter().next().unwrap()
                } else {
                    Type::Tuple(t.v)
                }
            }
            Token::Struct => {
                self.skip();
                let fields = self.ty_fields(follow)?;
                Type::Record(fields.v)
            }
            Token::Fun => {
                self.skip();
                let tys = self.paren_seq(Self::ty, Type::FIRST, follow)?;
                self.expect(Token::Colon, follow)?;
                let ty = self.ty(follow)?;
                Type::Fun(tys.v, Rc::new(ty.v))
            }
            Token::LBrack => {
                self.skip();
                let ty = self.ty(follow | Token::SemiColon)?;
                self.expect(Token::SemiColon, follow)?;
                let n = self.index(follow | Token::RBrack)?;
                self.expect(Token::RBrack, follow)?;
                Type::Array(Rc::new(ty.v), Some(n.v.data))
            }
            _ => unreachable!(),
        };
        Ok(Spanned::new(t.s, lhs))
    }

    fn pat(&mut self, follow: Token) -> Result<Spanned<Pat>, Span> {
        self.pat_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Pat::Err(s, Type::Hole))))
    }

    fn pat_fallible(&mut self, follow: Token) -> Result<Spanned<Pat>, Span> {
        self.pat_bp(follow, 0)
    }

    fn pat_bp(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Pat>, Span> {
        let mut lhs = self
            .pat_lhs(follow | Pat::FOLLOW)
            .unwrap_or_else(|s| Spanned::new(s, Pat::Err(s, Type::Hole)));
        loop {
            let op = self.start(follow | Pat::FOLLOW, follow)?;
            if let Some((lbp, ())) = op.v.postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let e = match op.v {
                    Token::Colon => {
                        self.skip();
                        let ty = self.ty(follow)?;
                        let s = lhs.s + ty.s;
                        lhs.v.with_ty(ty.v).with_span(s)
                    }
                    _ => unreachable!(),
                };
                let s = lhs.s + op.s;
                lhs = Spanned::new(s, e);
            } else if let Some((lbp, rbp)) = op.v.infix_bp() {
                if lbp < min_bp {
                    break;
                }
                match op.v {
                    Token::Or => {
                        self.skip();
                        let rhs = self.pat_bp(follow, rbp)?;
                        let s = lhs.s + rhs.s;
                        lhs =
                            Spanned::new(s, Pat::Or(s, Type::Hole, Rc::new(lhs.v), Rc::new(rhs.v)));
                    }
                    Token::Eq => break,
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn pat_lhs(&mut self, follow: Token) -> Result<Spanned<Pat>, Span> {
        let t = self.start(Pat::FIRST, follow)?;
        let lhs = match t.v {
            Token::Name => {
                let path = self.path(follow | Token::LParen)?;
                if self.start(Token::LParen | follow, follow)?.v == Token::LParen {
                    let t = self.pat_args(follow)?;
                    let s = path.s + t.s;
                    Pat::Unresolved(s, Type::Hole, path.v, Some(t.v))
                } else {
                    Pat::Unresolved(path.s, Type::Hole, path.v, None)
                }
            }
            Token::LParen => {
                let t = self.pat_tuple(follow)?;
                if t.v.len() == 1 {
                    t.v.into_iter().next().unwrap()
                } else {
                    Pat::Tuple(t.s, Type::Hole, t.v)
                }
            }
            Token::Struct => {
                let t0 = self.next();
                let xps = self.pat_fields(follow)?;
                let s = t0.s + xps.s;
                Pat::Record(s, Type::Hole, xps.v)
            }
            Token::Underscore => {
                let t = self.next();
                Pat::Wildcard(t.s, Type::Hole)
            }
            Token::Int => {
                let t = self.next();
                let v = self.text(t).to_owned();
                Pat::Int(t.s, Type::Hole, v)
            }
            Token::String => {
                let t = self.next();
                let v = self.text(t).to_owned();
                Pat::String(t.s, Type::Hole, v)
            }
            Token::Char => {
                let t = self.next();
                let v = self.text(t).chars().next().unwrap();
                Pat::Char(t.s, Type::Hole, v)
            }
            Token::True | Token::False => {
                let t = self.next();
                let v = t.v == Token::True;
                Pat::Bool(t.s, Type::Hole, v)
            }
            _ => unreachable!(),
        };
        Ok(Spanned::new(t.s, lhs))
    }

    fn ty_annot(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.expect(Token::Colon, follow | Type::FIRST)?;
        self.ty(follow)
    }

    fn expr_args(&mut self, follow: Token) -> Result<Spanned<Vec<Expr>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::expr, Token::Comma, Expr::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn ty_tuple(&mut self, follow: Token) -> Result<Spanned<Vec<Type>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::ty, Token::Comma, Type::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn pat_tuple(&mut self, follow: Token) -> Result<Spanned<Vec<Pat>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::pat, Token::Comma, Pat::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn pat_args(&mut self, follow: Token) -> Result<Spanned<Vec<UnresolvedPatField>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::pat_arg, Token::Comma, Pat::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn pat_arg(&mut self, follow: Token) -> Result<Spanned<UnresolvedPatField>, Span> {
        let mut p0 = self.pat(follow | Token::Eq)?;
        if let Pat::Unresolved(_, Type::Hole, path, fields) = &mut p0.v {
            if path.segments.len() == 1
                && path.segments[0].1.is_empty()
                && fields.is_none()
                && self.start(Token::Eq | follow, follow)?.v == Token::Eq
            {
                self.skip();
                // x = p
                let x = path.segments.pop().unwrap().0;
                let p1 = self.pat(follow)?;
                let s = p0.s + p1.s;
                return Ok(Spanned::new(s, UnresolvedPatField::Named(x, p1.v)));
            }
        }
        let s = p0.s;
        Ok(Spanned::new(s, UnresolvedPatField::Unnamed(p0.v)))
    }

    fn pat_fields(&mut self, follow: Token) -> Result<Spanned<Vec<(Name, Pat)>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::pat_field, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn pat_field(&mut self, follow: Token) -> Result<Spanned<(Name, Pat)>, Span> {
        let x = self.name(follow)?;
        if self.start(Token::Eq | follow, follow)?.v == Token::Eq {
            self.skip();
            let p = self.pat(follow)?;
            let s = x.s + p.s;
            Ok(Spanned::new(s, (x.v, p.v)))
        } else {
            let s = x.s;
            let path = Path::new(vec![(x.v.clone(), vec![])]);
            let p = Pat::Unresolved(s, Type::Hole, path, None);
            Ok(Spanned::new(s, (x.v, p)))
        }
    }

    fn arms(&mut self, follow: Token) -> Result<Spanned<Vec<Arm>>, Span> {
        self.brace(
            |p, follow| p.seq(Self::arm, Token::Comma, Pat::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn arm(&mut self, follow: Token) -> Result<Spanned<Arm>, Span> {
        let p = self.pat(follow | Token::FatArrow)?;
        self.expect(Token::FatArrow, follow)?;
        let e = self.expr(follow)?;
        let s = p.s + e.s;
        Ok(Spanned::new(p.s + e.s, Arm::new(s, p.v, e.v)))
    }

    fn block(&mut self, follow: Token) -> Result<Spanned<Block>, Span> {
        let t0 = self.expect(Token::LBrace, follow)?;
        let mut stmts = Vec::new();
        loop {
            let t1 = self.start(Stmt::FIRST | Token::SemiColon | Token::RBrace, follow)?;
            let stmt = match t1.v {
                // Token::Var => Stmt::Var(self.stmt_var(follow)?),
                // Token::Impl => Stmt::Impl(self.stmt_impl(follow)?),
                // Token::Trait => Stmt::Trait(self.stmt_trait(follow)?),
                // Token::Struct => Stmt::Struct(self.stmt_struct(follow)?),
                // Token::Enum => Stmt::Enum(self.stmt_enum(follow)?),
                Token::SemiColon => {
                    while self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {}
                    continue;
                }
                Token::RBrace => {
                    let t1 = self.next();
                    let s = t0.s + t1.s;
                    let expr = Expr::Tuple(s, Type::Hole, vec![]);
                    return Ok(Spanned::new(s, Block::new(s, stmts, expr)));
                }
                t if Expr::FIRST.contains(t) => {
                    let expr = self.expr(follow | Token::RBrace | Stmt::FIRST)?;
                    if self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {
                        // {e;...}
                        while self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {}
                        Stmt::Expr(expr.v)
                    } else {
                        if expr.v.is_braced() && self.peek().v != Token::RBrace {
                            // { { } ... }
                            Stmt::Expr(expr.v)
                        } else {
                            // {e}
                            let t1 = self.expect(Token::RBrace, follow)?;
                            let s = t0.s + t1.s;
                            return Ok(Spanned::new(s, Block::new(s, stmts, expr.v)));
                        }
                    }
                }
                _ => self.stmt(follow)?.v,
            };
            stmts.push(stmt);
        }
    }

    fn expr(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        self.expr_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Expr::Err(s, Type::Hole))))
    }

    fn expr_fallible(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        self.expr_bp(follow, 0)
    }

    fn expr_bp(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Expr>, Span> {
        let mut lhs = self
            .expr_lhs(follow | Expr::FOLLOW)
            .unwrap_or_else(|s| Spanned::new(s, Expr::Err(s, Type::Hole)));
        loop {
            let op = self.start(follow | Expr::FOLLOW | Stmt::FIRST, follow)?;
            if let Some((lbp, ())) = op.v.postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let e = match op.v {
                    Token::Question => {
                        self.skip();
                        let s = lhs.s + op.s;
                        Expr::Postfix(s, Type::Hole, op.v, Rc::new(lhs.v))
                    }
                    Token::LParen => {
                        let args = self.expr_args(follow)?;
                        let s = lhs.s + args.s;
                        Expr::Call(s, Type::Hole, Rc::new(lhs.v), args.v)
                    }
                    Token::Colon => {
                        self.skip();
                        let ty = self.ty(follow)?;
                        let s = lhs.s + ty.s;
                        lhs.v.with_ty(ty.v).with_span(s)
                    }
                    Token::Dot => {
                        self.skip();
                        let t = self.start(Token::Name | Token::Int, follow)?;
                        match t.v {
                            Token::Name => {
                                let name = self.name(follow)?;
                                let t = self.start(
                                    follow | Token::LBrack | Token::LParen | Expr::FOLLOW,
                                    follow,
                                )?;
                                if t.v == Token::LBrack || t.v == Token::LParen {
                                    let tys = self.ty_args(follow | Token::LParen)?;
                                    let args = self.expr_args(follow)?;
                                    let s = lhs.s + args.s;
                                    let args = std::iter::once(lhs.v)
                                        .chain(args.v.into_iter())
                                        .collect::<Vec<_>>();
                                    let tys = tys.map(|x| x.v).unwrap_or_default();
                                    let path = Path::new(vec![(name.v, tys)]);
                                    let fun = Expr::Unresolved(lhs.s, Type::Hole, path);
                                    Expr::Call(s, Type::Hole, Rc::new(fun), args)
                                } else {
                                    let s = lhs.s + name.s;
                                    Expr::Field(s, Type::Hole, Rc::new(lhs.v), name.v)
                                }
                            }
                            Token::Int => {
                                let t = self.index(follow)?;
                                let s = lhs.s + t.s;
                                Expr::Index(s, Type::Hole, Rc::new(lhs.v), t.v)
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                };
                let s = lhs.s + op.s;
                lhs = Spanned::new(s, e);
            } else if let Some((lbp, rbp)) = op.v.infix_bp() {
                if lbp < min_bp {
                    break;
                }
                self.skip();
                let rhs = self.expr_bp(follow, rbp)?;
                let s = lhs.s + rhs.s;
                lhs = Spanned::new(
                    s,
                    Expr::Infix(s, Type::Hole, op.v, Rc::new(lhs.v), Rc::new(rhs.v)),
                );
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn expr_lhs(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        let t0 = self.start(Expr::FIRST, follow)?;
        match t0.v {
            Token::True | Token::False => {
                self.skip();
                let v = t0.v == Token::True;
                Ok(Spanned::new(t0.s, Expr::Bool(t0.s, Type::Hole, v)))
            }
            Token::Int => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Int(s, Type::Hole, v)))
            }
            Token::Float => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Float(s, Type::Hole, v)))
            }
            Token::String => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::String(s, Type::Hole, v)))
            }
            Token::Char => {
                self.skip();
                let v = self.text(t0).chars().next().unwrap();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Char(s, Type::Hole, v)))
            }
            Token::Name => {
                let path = self.path(follow)?;
                let s = path.s;
                Ok(Spanned::new(s, Expr::Unresolved(s, Type::Hole, path.v)))
            }
            Token::LParen => {
                self.debug(follow);
                let t = self.expr_args(follow)?;
                let s = t.s;
                if t.v.len() == 1 {
                    Ok(Spanned::new(s, t.v.into_iter().next().unwrap()))
                } else {
                    Ok(Spanned::new(s, Expr::Tuple(t.s, Type::Hole, t.v)))
                }
            }
            Token::Minus => {
                let op = self.next();
                let ((), rbp) = op.v.prefix_bp().unwrap();
                let rhs = self.expr_bp(follow, rbp)?;
                let s = op.s + rhs.s;
                Ok(Spanned::new(
                    s,
                    Expr::Prefix(op.s, Type::Hole, op.v, Rc::new(rhs.v)),
                ))
            }
            Token::Not => {
                let op = self.next();
                let ((), rbp) = op.v.prefix_bp().unwrap();
                let rhs = self.expr_bp(follow, rbp)?;
                let s = op.s + rhs.s;
                Ok(Spanned::new(
                    s,
                    Expr::Prefix(op.s, Type::Hole, op.v, Rc::new(rhs.v)),
                ))
            }
            Token::Break => {
                self.skip();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Break(s, Type::Hole)))
            }
            Token::Continue => {
                self.skip();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Continue(s, Type::Hole)))
            }
            Token::Return => {
                self.skip();
                let t = self.start(follow | Expr::FIRST, follow)?;
                if Expr::FIRST.contains(t.v) {
                    let e = self.expr(follow)?;
                    let s = t0.s + e.s;
                    Ok(Spanned::new(s, Expr::Return(s, Type::Hole, Rc::new(e.v))))
                } else {
                    let s = t0.s;
                    let e = Rc::new(Expr::Tuple(s, Type::Hole, vec![]));
                    Ok(Spanned::new(s, Expr::Return(s, Type::Hole, e)))
                }
            }
            Token::LBrack => {
                let es = self.brack(Self::exprs, follow)?.flatten();
                let s = t0.s + es.s;
                Ok(Spanned::new(s, Expr::Array(s, Type::Hole, es.v)))
            }
            Token::Fun => {
                let t = self.next();
                let params = self.inferred_params(follow)?;
                let t1 = self.start(follow | Token::Colon, follow | Token::Eq)?;
                let ty = if t1.v == Token::Colon {
                    self.ty_annot(follow | Token::Eq)?.v
                } else {
                    Type::Hole
                };
                self.expect(Token::Eq, follow)?;
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    Expr::Fun(s, Type::Hole, params.v, ty, Rc::new(e.v)),
                ))
            }
            Token::If => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let b0 = self.block(follow | Token::Else | Stmt::FIRST)?;
                if self.start(follow | Token::Else | Stmt::FIRST, follow)?.v == Token::Else {
                    self.skip();
                    let b1 = self.block(follow)?;
                    let s = t.s + b1.s;
                    Ok(Spanned::new(
                        s,
                        Expr::If(s, Type::Hole, Rc::new(e.v), b0.v, b1.v),
                    ))
                } else {
                    let s = t.s + b0.s;
                    let b1 = Block::new(s, vec![], Expr::Tuple(s, Type::Hole, vec![]));
                    Ok(Spanned::new(
                        s,
                        Expr::If(s, Type::Hole, Rc::new(e.v), b0.v, b1),
                    ))
                }
            }
            Token::Match => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let arms = self.arms(follow)?;
                let s = t.s + arms.s;
                Ok(Spanned::new(
                    s,
                    Expr::Match(s, Type::Hole, Rc::new(e.v), arms.v),
                ))
            }
            Token::While => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Ok(Spanned::new(
                    s,
                    Expr::While(s, Type::Hole, Rc::new(e.v), b.v),
                ))
            }
            Token::For => {
                let t = self.next();
                let x = self.name(follow)?;
                self.expect(Token::In, follow)?;
                let e = self.expr(follow | Token::LBrace)?;
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Ok(Spanned::new(
                    s,
                    Expr::For(s, Type::Hole, x.v, Rc::new(e.v), b.v),
                ))
            }
            Token::From => {
                let qs = self
                    .repeat(Self::query, Query::FIRST, follow | Query::FOLLOW)?
                    .unwrap();
                let s = t0.s + qs.s;
                Ok(Spanned::new(s, Expr::Query(s, Type::Hole, qs.v)))
            }
            Token::LBrace => {
                let b = self.block(follow)?;
                Ok(Spanned::new(b.s, Expr::Block(b.s, Type::Hole, b.v)))
            }
            _ => unreachable!(),
        }
    }

    fn query(&mut self, follow: Token) -> Result<Spanned<Query>, Span> {
        self.query_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Query::Err(s, Type::Hole))))
    }

    fn query_fallible(&mut self, follow: Token) -> Result<Spanned<Query>, Span> {
        let t = self.start(Query::FIRST, follow)?;
        match t.v {
            Token::From => {
                let t = self.next();
                let x = self.name(follow | Token::In)?;
                self.expect(Token::In, follow)?;
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    Query::From(s, Type::Hole, x.v, Rc::new(e.v)),
                ))
            }
            Token::Where => {
                let t = self.next();
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, Query::Where(s, Type::Hole, Rc::new(e.v))))
            }
            Token::Select => {
                let t = self.next();
                let es = self.seq_nonempty(Self::field_expr, Token::Comma, Token::Name, follow)?;
                let s = t.s + es.s;
                Ok(Spanned::new(s, Query::Select(s, Type::Hole, es.v)))
            }
            Token::Compute => {
                let t = self.next();
                let x = self.name(follow | Token::Eq)?;
                self.expect(Token::Eq, follow)?;
                let e0 = self.expr(follow | Token::Of)?;
                self.expect(Token::Of, follow)?;
                let e1 = self.expr(follow)?;
                let s = t.s + e1.s;
                Ok(Spanned::new(
                    s,
                    Query::Compute(s, Type::Hole, x.v, Rc::new(e0.v), Rc::new(e1.v)),
                ))
            }
            Token::Group => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let qs = self.brace(
                    |p, follow| p.repeat(Self::query, Query::FIRST, follow),
                    follow,
                )?;
                let s = t.s + qs.s;
                Ok(Spanned::new(
                    s,
                    Query::Group(s, Type::Hole, Rc::new(e.v), qs.v.unwrap_or_default()),
                ))
            }
            Token::Var => {
                let t = self.next();
                let x = self.name(follow | Token::Eq)?;
                self.expect(Token::Eq, follow)?;
                let e = self.expr(follow | Query::FIRST)?;
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    Query::Var(s, Type::Hole, x.v, Rc::new(e.v)),
                ))
            }
            Token::Into => {
                let t = self.next();
                let x = self.name(follow | Token::LParen)?;
                let ts = self
                    .ty_args(follow | Token::LParen)?
                    .map(|x| x.v)
                    .unwrap_or_default();
                let es = self.expr_args(follow | Query::FIRST)?;
                let s = t.s + es.s;
                Ok(Spanned::new(s, Query::Into(s, Type::Hole, x.v, ts, es.v)))
            }
            Token::Over => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let qs = self
                    .brace(
                        |p, follow| p.repeat(Self::query, Query::FIRST, follow),
                        follow,
                    )?
                    .v
                    .unwrap_or_default();
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    Query::Over(s, Type::Hole, Rc::new(e.v), qs),
                ))
            }
            _ => unreachable!(),
        }
    }

    fn field_expr(&mut self, follow: Token) -> Result<Spanned<(Name, Expr)>, Span> {
        let x = self.name(follow | Token::Eq)?;
        match self.start(Token::Eq | follow, follow)?.v {
            Token::Eq => {
                self.next();
                let e = self.expr(follow)?;
                let s = x.s + e.s;
                Ok(Spanned::new(s, (x.v, e.v)))
            }
            _ => {
                let path = Path::new(vec![(x.v.clone(), vec![])]);
                let s = x.s;
                Ok(Spanned::new(
                    s,
                    (x.v, Expr::Unresolved(s, Type::Hole, path)),
                ))
            }
        }
    }
}

impl Stmt {
    pub fn parse_ok(input: &str) -> Stmt {
        ok(parse(input, |a, b| Parser::stmt(a, b)))
    }

    pub fn parse_err(input: &str) -> (Stmt, String) {
        parse(input, |a, b| Parser::stmt(a, b)).unwrap_err()
    }
}

impl Expr {
    pub fn parse_ok(input: &str) -> Expr {
        ok(parse(input, |a, b| Parser::expr(a, b)))
    }
    pub fn parse_err(input: &str) -> (Expr, String) {
        parse(input, |a, b| Parser::expr(a, b)).unwrap_err()
    }
}

impl Type {
    pub fn parse_ok(input: &str) -> Type {
        ok(parse(input, |a, b| Parser::ty(a, b)))
    }
    pub fn parse_err(input: &str) -> (Type, String) {
        parse(input, |a, b| Parser::ty(a, b)).unwrap_err()
    }
}

impl Pat {
    pub fn parse_ok(input: &str) -> Pat {
        ok(parse(input, |a, b| Parser::pat(a, b)))
    }
    pub fn parse_err(input: &str) -> (Pat, String) {
        parse(input, |a, b| Parser::pat(a, b)).unwrap_err()
    }
}

impl Program {
    pub fn parse_ok(input: &str) -> Program {
        ok(parse(input, |a, b| Parser::program(a, b)))
    }
    pub fn parse_err(input: &str) -> (Program, String) {
        parse(input, |a, b| Parser::program(a, b)).unwrap_err()
    }
}

fn ok<T>(t: Result<T, (T, String)>) -> T {
    match t {
        Ok(t) => t,
        Err((_, e)) => panic!("{}", e),
    }
}

fn parse<'a, T>(
    input: &'a str,
    f: impl for<'b> FnOnce(&mut Parser<'a, &'b mut Lexer<'a>>, Token) -> Result<Spanned<T>, Span>,
) -> Result<T, (T, String)> {
    let mut sources = Sources::new();
    let id = sources.add("test", input);
    let mut lexer = Lexer::new(id, input);
    let mut parser = Parser::new(input, &mut lexer);
    let output = f(&mut parser, Token::Eof).unwrap();
    let mut report = parser.report;
    report.merge(&mut lexer.report);
    if report.is_empty() {
        Ok(output.v)
    } else {
        Err((output.v, trim(report.string(&mut sources).unwrap())))
    }
}

impl Expr {
    fn is_braced(&self) -> bool {
        matches!(
            self,
            Expr::Block(..) | Expr::If(..) | Expr::Match(..) | Expr::While(..) | Expr::For(..)
        ) || matches!(self, Expr::Query(_, _, qs) if qs.last().is_some_and(|x| x.is_braced()))
    }
}

impl Query {
    fn is_braced(&self) -> bool {
        matches!(
            self,
            Query::Group(..) | Query::Over(..) | Query::Into(..) | Query::Compute(..)
        )
    }
}

trait TokenSets {
    const FIRST: Token;
    const FOLLOW: Token;
}

impl TokenSets for Expr {
    const FIRST: Token = Token::Int
        .or(Token::Float)
        .or(Token::String)
        .or(Token::Name)
        .or(Token::LParen)
        .or(Token::Minus)
        .or(Token::Break)
        .or(Token::Continue)
        .or(Token::Return)
        .or(Token::LBrack)
        .or(Token::Fun)
        .or(Token::If)
        .or(Token::Match)
        .or(Token::While)
        .or(Token::True)
        .or(Token::False)
        .or(Token::For)
        .or(Token::Not)
        .or(Token::Char)
        .or(Token::LBrace)
        .or(Query::FIRST);
    const FOLLOW: Token = Token::Eof
        .or(Token::And)
        .or(Token::DotDot)
        .or(Token::Dot)
        .or(Token::Eq)
        .or(Token::EqEq)
        .or(Token::Ge)
        .or(Token::Gt)
        .or(Token::Le)
        .or(Token::Lt)
        .or(Token::Minus)
        .or(Token::NotEq)
        .or(Token::Or)
        .or(Token::Plus)
        .or(Token::Slash)
        .or(Token::Star)
        .or(Token::LParen)
        .or(Token::Colon)
        .or(Token::SemiColon);
}

impl TokenSets for Stmt {
    const FIRST: Token = Token::Def
        .or(Token::Type)
        .or(Token::Trait)
        .or(Token::Struct)
        .or(Token::Enum)
        .or(Token::Impl)
        .or(Token::Var)
        .or(Expr::FIRST);
    const FOLLOW: Token = Token::Eof;
}

impl TokenSets for Type {
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Struct)
        .or(Token::Fun)
        .or(Token::LBrack);
    const FOLLOW: Token = Token::Eof;
}

impl TokenSets for Pat {
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Underscore)
        .or(Token::Int)
        .or(Token::String)
        .or(Token::Struct)
        .or(Token::True)
        .or(Token::False)
        .or(Token::Char);
    const FOLLOW: Token = Token::Eof.or(Token::Or).or(Token::Colon);
}

impl TokenSets for Query {
    const FIRST: Token = Token::From
        .or(Token::Where)
        .or(Token::Over)
        .or(Token::Group)
        .or(Token::Var)
        .or(Token::Select)
        .or(Token::Compute)
        .or(Token::Into);
    const FOLLOW: Token = Expr::FOLLOW.or(Query::FIRST);
}
