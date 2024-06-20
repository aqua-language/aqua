#![allow(clippy::type_complexity)]
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

use crate::ast::Aggr;
use crate::ast::Block;
use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Expr;
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
use crate::ast::Trait;
use crate::ast::Type;
use crate::collections::map::Map;
use crate::diag::Report;
use crate::lexer::Span;
use crate::lexer::Spanned;
use crate::lexer::Token;

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

enum Either<A, B> {
    A(A),
    B(B),
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Spanned<Token>>,
{
    const FUEL: usize = 1000;

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
        self.lexer.next();
    }

    /// Get the text of a token
    fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    /// Error recovery. Discards tokens until the first token is found,
    /// or the end of the file is reached. Takes closing/opening braces into account:
    /// 1. (+ [)]) => Recover at the second ) and report an error: Unexpected token +
    /// 2. <Open>  => Recover at the end of the file and report an error: Unmatched <Open>
    fn recover<const PEEK: bool>(
        &mut self,
        first: Token,
        follow: Token,
    ) -> Result<Spanned<Token>, Span> {
        let mut fuel = Self::FUEL;
        loop {
            let t = self.peek();
            match t.v {
                _ if fuel == 0 => {
                    self.report.err(t.s, "Too many errors", "stopping here");
                    return Err(t.s);
                }
                _ if first.contains(t.v) && self.delims.is_empty() => {
                    return if PEEK { Ok(t) } else { Ok(self.next()) }
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
            fuel -= 1;
        }
    }

    fn expect(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if first.contains(t.v) {
            Ok(self.next())
        } else {
            self.report
                .err(t.s, format!("Unexpected token `{}`", t.v), first.expected());
            self.recover::<false>(first, follow)
        }
    }

    fn start(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if first.contains(t.v) {
            Ok(t)
        } else {
            self.report
                .err(t.s, format!("Unexpected token `{}`", t.v), first.expected());
            self.recover::<true>(first, follow)
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
        f: impl FnOnce(&mut Self, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let t = f(self, Token::Eof).ok()?;
        self.expect(Token::Eof, Token::Eof).ok()?;
        Some(t.v)
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
        Ok(Spanned::new(s, Program::new(s, stmts)))
    }

    pub fn stmt_def_builtin(
        &mut self,
        follow: Token,
        body: BuiltinDef,
    ) -> Result<Spanned<StmtDef>, Span> {
        let t0 = self.expect(Token::Def, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen)?;
        let xts = self.params(follow)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::Where | Token::SemiColon)?;
        let bs = self.where_clause(follow | Token::SemiColon)?;
        self.expect(Token::SemiColon, follow)?;
        let s = t0.s + x.s;
        let body = StmtDefBody::Builtin(body);
        Ok(Spanned::new(
            s,
            StmtDef::new(s, x.v, gs, xts.v, t.v, bs, body),
        ))
    }

    pub fn stmt_type_builtin(
        &mut self,
        follow: Token,
        body: BuiltinType,
    ) -> Result<Spanned<StmtType>, Span> {
        let t0 = self.expect(Token::Type, follow)?;
        let x = self.name(follow | Token::SemiColon)?;
        let gs = self.generics(follow | Token::SemiColon)?;
        let t1 = self.expect(Token::SemiColon, follow)?;
        let s = t0.s + t1.s;
        let body = StmtTypeBody::Builtin(body);
        Ok(Spanned::new(s, StmtType::new(s, x.v, gs, body)))
    }

    pub fn stmt_impl_builtin<const N: usize>(
        &mut self,
        follow: Token,
        bodies: [BuiltinDef; N],
    ) -> Result<Spanned<StmtImpl>, Span> {
        let t0 = self.expect(Token::Impl, follow)?;
        let gs = self.generics(follow | Token::Name)?;
        let b = self.bound(follow | Token::Where | Token::LBrace)?;
        let bs = self.where_clause(follow | Token::LBrace)?;
        self.expect(Token::LBrace, follow)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        let mut bodies = bodies.into_iter();
        loop {
            let t = self.start(Token::RBrace | Token::Def | Token::Type, follow)?;
            match t.v {
                Token::Def => defs.push(Rc::new(
                    self.stmt_def_builtin(
                        follow | Token::RBrace,
                        bodies
                            .next()
                            .expect(&format!("Builtin body missing for {}", b.v)),
                    )?
                    .v,
                )),
                Token::Type => tys.push(Rc::new(self.stmt_type(follow)?.v)),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace, follow)?;
        let s = t0.s + s1.s;
        Ok(Spanned::new(s, StmtImpl::new(s, gs, b.v, bs, defs, tys)))
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
                Ok(Spanned::new(s.s, Stmt::Def(Rc::new(s.v))))
            }
            Token::Struct => {
                let s = self.stmt_struct(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Struct(Rc::new(s.v))))
            }
            Token::Enum => {
                let s = self.stmt_enum(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Enum(Rc::new(s.v))))
            }
            Token::Trait => {
                let s = self.stmt_trait(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Trait(Rc::new(s.v))))
            }
            Token::Impl => {
                let s = self.stmt_impl(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Impl(Rc::new(s.v))))
            }
            Token::Type => {
                let s = self.stmt_type(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Type(Rc::new(s.v))))
            }
            Token::Var => {
                let s = self.stmt_var(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Var(Rc::new(s.v))))
            }
            _ => {
                let s = self.stmt_expr(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Expr(Rc::new(s.v))))
            }
        }
    }

    pub fn stmt_def(&mut self, follow: Token) -> Result<Spanned<StmtDef>, Span> {
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
            StmtDef::new(s, x.v, gs, xts.v, t.v, bs, StmtDefBody::UserDefined(e.v)),
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
        Ok(Spanned::new(s, StmtStruct::new(s, x.v, gs, xts.into())))
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
        Ok(Spanned::new(
            s,
            StmtType::new(s, x.v, gs, StmtTypeBody::UserDefined(t.v)),
        ))
    }

    fn stmt_var(&mut self, follow: Token) -> Result<Spanned<StmtVar>, Span> {
        let t0 = self.expect(Token::Var, follow)?;
        let x = self.name(follow)?;
        let t = self
            .optional(Self::ty_annot, Token::Colon, follow | Token::Eq)?
            .map(|x| x.v)
            .unwrap_or(Type::Unknown);
        self.expect(Token::Eq, follow)?;
        let e = self.stmt_expr(follow)?;
        let s = t0.s + e.s;
        Ok(Spanned::new(s, StmtVar::new(s, x.v, t, e.v)))
    }

    pub fn stmt_trait(&mut self, follow: Token) -> Result<Spanned<StmtTrait>, Span> {
        let t0 = self.expect(Token::Trait, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LBrace)?;
        let bs = self.where_clause(follow | Token::LBrace)?;
        self.expect(Token::LBrace, follow)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        loop {
            let t = self.start(Token::RBrace | Token::Def | Token::Type, follow)?;
            match t.v {
                Token::Def => defs.push(Rc::new(self.stmt_def_decl(follow)?.v)),
                Token::Type => tys.push(Rc::new(self.stmt_type_decl(follow)?.v)),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace, follow)?;
        let s = t0.s + s1.s;
        Ok(Spanned::new(s, StmtTrait::new(s, x.v, gs, bs, defs, tys)))
    }

    fn stmt_def_decl(&mut self, follow: Token) -> Result<Spanned<StmtTraitDef>, Span> {
        let t0 = self.expect(Token::Def, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen)?;
        let xts = self.params(follow | Token::Colon)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::Where | Token::SemiColon)?;
        let bs = self.where_clause(follow | Token::SemiColon)?;
        let t1 = self.expect(Token::SemiColon, follow)?;
        let s = t0.s + t1.s;
        Ok(Spanned::new(
            s,
            StmtTraitDef::new(s, x.v, gs, xts.v, t.v, bs),
        ))
    }

    fn stmt_type_decl(&mut self, follow: Token) -> Result<Spanned<StmtTraitType>, Span> {
        let t0 = self.expect(Token::Type, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::Eq | Token::SemiColon)?;
        let t1 = self.expect(Token::SemiColon, follow)?;
        let s = t0.s + t1.s;
        Ok(Spanned::new(s, StmtTraitType::new(s, x.v, gs)))
    }

    pub fn stmt_impl(&mut self, follow: Token) -> Result<Spanned<StmtImpl>, Span> {
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
                Token::Def => defs.push(Rc::new(self.stmt_def(follow)?.v)),
                Token::Type => tys.push(Rc::new(self.stmt_type(follow)?.v)),
                _ => break,
            }
        }
        let s1 = self.expect(Token::RBrace, follow)?;
        let s = t0.s + s1.s;
        Ok(Spanned::new(s, StmtImpl::new(s, gs, b.v, bs, defs, tys)))
    }

    fn stmt_expr(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        let e = self.expr(follow)?;
        if e.v.is_braced() {
            Ok(Spanned::new(e.s, e.v))
        } else {
            let t = self.expect(Token::SemiColon, follow)?;
            Ok(Spanned::new(e.s + t.s, e.v))
        }
    }

    fn ty_variants(&mut self, follow: Token) -> Result<Map<Name, Type>, Span> {
        self.brace(
            |p, follow| p.seq(Self::ty_variant, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| x.v.unwrap_or_default().into())
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

    fn where_clause(&mut self, follow: Token) -> Result<Vec<Trait>, Span> {
        self.optional(
            |this, follow| {
                let t0 = this.expect(Token::Where, follow)?;
                let bs = this.seq(Self::bound, Token::Comma, Token::Name, follow)?;
                if let Some(xs) = bs {
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

    fn bound(&mut self, follow: Token) -> Result<Spanned<Trait>, Span> {
        let x = self.path(follow)?;
        Ok(Spanned::new(x.s, Trait::Path(x.s, x.v)))
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

    fn params(&mut self, follow: Token) -> Result<Spanned<Map<Name, Type>>, Span> {
        self.paren_seq(Self::param, Token::Name, follow)
            .map(|x| x.map(|x| x.into_iter().collect()))
    }

    fn param(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let name = self.name(follow)?;
        self.expect(Token::Colon, follow)?;
        let ty = self.ty(follow)?;
        let s = name.s + ty.s;
        Ok(Spanned::new(s, (name.v, ty.v)))
    }

    fn inferred_params(&mut self, follow: Token) -> Result<Spanned<Map<Name, Type>>, Span> {
        self.paren_seq(Self::inferred_param, Token::Name, follow)
            .map(|x| x.map(|x| x.into_iter().collect()))
    }

    fn inferred_param(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let x = self.name(follow)?;
        if self.start(follow | Token::Colon, follow)?.v == Token::Colon {
            self.skip();
            let t = self.ty(follow)?;
            let s = x.s + t.s;
            Ok(Spanned::new(s, (x.v, t.v)))
        } else {
            let s = x.s;
            Ok(Spanned::new(s, (x.v, Type::Unknown)))
        }
    }

    fn path(&mut self, follow: Token) -> Result<Spanned<Path>, Span> {
        let xs = self.seq_nonempty(Self::segment, Token::ColonColon, Token::Name, follow)?;
        Ok(Spanned::new(xs.s, Path::new(xs.v)))
    }

    fn segment(&mut self, follow: Token) -> Result<Spanned<Segment>, Span> {
        let name = self.name(follow)?;
        let args = self.trait_args(follow)?;
        if let Some(args) = args {
            let s = name.s + args.s;
            let seg = Segment::new(s, name.v, args.v.0, args.v.1.into());
            Ok(Spanned::new(name.s + args.s, seg))
        } else {
            let s = name.s;
            let seg = Segment::new(s, name.v, vec![], Map::new());
            Ok(Spanned::new(name.s, seg))
        }
    }

    fn trait_args(
        &mut self,
        follow: Token,
    ) -> Result<Option<Spanned<(Vec<Type>, Vec<(Name, Type)>)>>, Span> {
        let args = self.optional(
            |this, follow| {
                this.brack(
                    |this, follow| this.seq(Self::trait_arg, Token::Comma, Type::FIRST, follow),
                    follow,
                )
                .map(|x| x.flatten())
            },
            Token::LBrack,
            follow,
        )?;
        if let Some(args) = args {
            let mut tys = Vec::new();
            let mut named_tys = Vec::new();
            for arg in args.v.into_iter() {
                match arg {
                    Either::A(ty) => tys.push(ty),
                    Either::B(named_ty) => named_tys.push(named_ty),
                }
            }
            Ok(Some(Spanned::new(args.s, (tys, named_tys))))
        } else {
            Ok(None)
        }
    }

    fn trait_arg(&mut self, follow: Token) -> Result<Spanned<Either<Type, (Name, Type)>>, Span> {
        let ty0 = self.ty(follow | Token::Eq)?;
        if let Some(name) = ty0.v.as_name() {
            if self.eat(Token::Eq, follow)? {
                let ty1 = self.ty(follow)?;
                let s = ty0.s + ty1.s;
                return Ok(Spanned::new(s, Either::B((*name, ty1.v))));
            }
        }
        Ok(Spanned::new(ty0.s, Either::A(ty0.v)))
    }

    fn optional_ty_args(&mut self, follow: Token) -> Result<Vec<Type>, Span> {
        let ts = self.optional(
            |p, follow| {
                p.brack(
                    |p, follow| p.seq(Self::ty, Token::Comma, Type::FIRST, follow),
                    follow,
                )
                .map(|x| x.flatten())
            },
            Token::LBrack,
            follow,
        )?;
        let ts = ts.map(|x| x.v).unwrap_or_default();
        Ok(ts)
        //
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

    pub fn ty(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.ty_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Type::Err)))
    }

    fn ty_fallible(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        let t = self.start(Type::FIRST, follow)?;
        let lhs = match t.v {
            Token::Name => {
                let path = self.path(follow)?;
                Type::Path(path.v)
            }
            Token::LParen => {
                let t = self.ty_tuple(follow)?;
                if t.v.len() == 1 {
                    t.v.into_iter().next().unwrap()
                } else {
                    Type::Tuple(t.v)
                }
            }
            Token::Record => {
                self.skip();
                let fields = self.ty_fields(follow)?;
                Type::Record(fields.v.into())
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
            Token::Not => {
                self.skip();
                Type::Never
            }
            Token::Underscore => {
                self.skip();
                Type::Unknown
            }
            _ => unreachable!(),
        };
        Ok(Spanned::new(t.s, lhs))
    }

    pub fn pat(&mut self, follow: Token) -> Result<Spanned<Pat>, Span> {
        self.pat_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Pat::Err(s, Type::Unknown))))
    }

    fn pat_fallible(&mut self, follow: Token) -> Result<Spanned<Pat>, Span> {
        self.pat_bp(follow, 0)
    }

    fn pat_bp(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Pat>, Span> {
        let mut lhs = self
            .pat_lhs(follow | Pat::FOLLOW)
            .unwrap_or_else(|s| Spanned::new(s, Pat::Err(s, Type::Unknown)));
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
                        Pat::Annotate(s, ty.v, Rc::new(lhs.v))
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
                        lhs = Spanned::new(
                            s,
                            Pat::Or(s, Type::Unknown, Rc::new(lhs.v), Rc::new(rhs.v)),
                        );
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
                    Pat::Path(s, Type::Unknown, path.v, Some(t.v))
                } else {
                    Pat::Path(path.s, Type::Unknown, path.v, None)
                }
            }
            Token::LParen => {
                let t = self.pat_tuple(follow)?;
                if t.v.len() == 1 {
                    t.v.into_iter().next().unwrap()
                } else {
                    Pat::Tuple(t.s, Type::Unknown, t.v)
                }
            }
            Token::Record => {
                let t0 = self.next();
                let xps = self.pat_fields(follow)?;
                let s = t0.s + xps.s;
                Pat::Record(s, Type::Unknown, xps.v.into())
            }
            Token::Underscore => {
                let t = self.next();
                Pat::Wildcard(t.s, Type::Unknown)
            }
            Token::Int => {
                let t = self.next();
                let v = self.text(t).into();
                Pat::Int(t.s, Type::Unknown, v)
            }
            Token::String => {
                let t = self.next();
                let v = self.text(t).into();
                Pat::String(t.s, Type::Unknown, v)
            }
            Token::Char => {
                let t = self.next();
                let v = self.text(t).chars().next().unwrap();
                Pat::Char(t.s, Type::Unknown, v)
            }
            Token::True | Token::False => {
                let t = self.next();
                let v = t.v == Token::True;
                Pat::Bool(t.s, Type::Unknown, v)
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
        self.paren_seq(Self::expr, Expr::FIRST, follow)
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

    fn pat_args(&mut self, follow: Token) -> Result<Spanned<Vec<PathPatField>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::pat_arg, Token::Comma, Pat::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn pat_arg(&mut self, follow: Token) -> Result<Spanned<PathPatField>, Span> {
        let mut p0 = self.pat(follow | Token::Eq)?;
        if let Pat::Path(_, Type::Unknown, path, fields) = &mut p0.v {
            let t1 = self.start(Token::Eq | follow, follow)?;
            if path.as_name().is_some() && fields.is_none() && t1.v == Token::Eq {
                self.skip();
                // x = p
                let x = path.segments.pop().unwrap().name;
                let p1 = self.pat(follow)?;
                let s = p0.s + p1.s;
                return Ok(Spanned::new(s, PathPatField::Named(x, p1.v)));
            }
        }
        let s = p0.s;
        Ok(Spanned::new(s, PathPatField::Unnamed(p0.v)))
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
            let path = Path::new_name(x.v);
            let p = Pat::Path(s, Type::Unknown, path, None);
            Ok(Spanned::new(s, (x.v, p)))
        }
    }

    fn arms(&mut self, follow: Token) -> Result<Spanned<Vec<(Pat, Expr)>>, Span> {
        self.brace(
            |p, follow| p.seq(Self::arm, Token::Comma, Pat::FIRST, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn arm(&mut self, follow: Token) -> Result<Spanned<(Pat, Expr)>, Span> {
        let p = self.pat(follow | Token::FatArrow)?;
        self.expect(Token::FatArrow, follow)?;
        let e = self.expr(follow)?;
        let s = p.s + e.s;
        Ok(Spanned::new(s, (p.v, e.v)))
    }

    /// Block parsing is slightly intricate. We need to handle four cases:
    /// * { } - An empty block
    /// * { e } - A block with a single expression
    /// * { s; ... } - A block with multiple statements
    /// * { s ... } - Where s is an expression that ends
    fn block(&mut self, follow: Token) -> Result<Spanned<Block>, Span> {
        let t0 = self.expect(Token::LBrace, follow)?;
        let mut stmts = Vec::new();
        loop {
            let t1 = self.start(Stmt::FIRST | Token::SemiColon | Token::RBrace, follow)?;
            let stmt = match t1.v {
                Token::SemiColon => {
                    while self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {}
                    continue;
                }
                Token::RBrace => {
                    let t1 = self.next();
                    let s = t0.s + t1.s;
                    let expr = Expr::Tuple(s, Type::Unknown, vec![]);
                    return Ok(Spanned::new(s, Block::new(s, stmts, expr)));
                }
                t if Expr::FIRST.contains(t) => {
                    let expr = self.expr(follow | Token::RBrace | Stmt::FIRST)?;
                    if self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {
                        // { e; ... }
                        while self.eat(Token::SemiColon, follow | Stmt::FIRST | Token::RBrace)? {}
                        Stmt::Expr(Rc::new(expr.v))
                    } else if expr.v.is_braced() && self.peek().v != Token::RBrace {
                        // { { } ... }
                        Stmt::Expr(Rc::new(expr.v))
                    } else {
                        // { e }
                        let t1 = self.expect(Token::RBrace, follow)?;
                        let s = t0.s + t1.s;
                        return Ok(Spanned::new(s, Block::new(s, stmts, expr.v)));
                    }
                }
                _ => self.stmt(follow | Token::RBrace)?.v,
            };
            stmts.push(stmt);
        }
    }

    pub fn expr(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        self.expr_fallible(follow, 0)
            .or_else(|s| Ok(Spanned::new(s, Expr::Err(s, Type::Unknown))))
    }

    fn expr_fallible(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Expr>, Span> {
        let mut lhs = self
            .expr_lhs(follow | Expr::FOLLOW | Stmt::FIRST)
            .unwrap_or_else(|s| Spanned::new(s, Expr::Err(s, Type::Unknown)));
        loop {
            let follow = follow | Expr::FOLLOW | Stmt::FIRST;
            let op = self.start(follow, follow)?;
            if let Some((lbp, ())) = op.v.postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let e = match op.v {
                    // a?
                    //
                    // becomes
                    //
                    // match a { Some(x) => x, None => return None }
                    Token::Question => {
                        todo!()
                        // self.skip();
                        // let s = lhs.s + op.s;
                        // Expr::Postfix(s, Type::Hole, op.v, Rc::new(lhs.v))
                    }
                    Token::LParen => {
                        let args = self.expr_args(follow)?;
                        let s = lhs.s + args.s;
                        Expr::Call(s, Type::Unknown, Rc::new(lhs.v), args.v)
                    }
                    Token::Colon => {
                        self.skip();
                        let ty = self.ty(follow)?;
                        let s = lhs.s + ty.s;
                        Expr::Annotate(s, ty.v, Rc::new(lhs.v))
                    }
                    Token::Dot => {
                        self.skip();
                        let t = self.start(Token::Name | Token::Int, follow)?;
                        match t.v {
                            Token::Name => {
                                let x = self.name(follow)?;
                                let t =
                                    self.start(follow | Token::LBrack | Token::LParen, follow)?;
                                if t.v == Token::LBrack || t.v == Token::LParen {
                                    let ts = self.optional_ty_args(follow | Token::LParen)?;
                                    let es = self.expr_args(follow)?;
                                    let s = lhs.s + es.s;
                                    Expr::Dot(s, Type::Unknown, Rc::new(lhs.v), x.v, ts, es.v)
                                } else {
                                    let s = lhs.s + x.s;
                                    Expr::Field(s, Type::Unknown, Rc::new(lhs.v), x.v)
                                }
                            }
                            Token::Int => {
                                let t = self.index(follow)?;
                                let s = lhs.s + t.s;
                                Expr::Index(s, Type::Unknown, Rc::new(lhs.v), t.v)
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
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = lhs.s + rhs.s;
                let e = if let Token::Eq = op.v {
                    let place = if lhs.v.is_place() {
                        Expr::Err(lhs.s, Type::Unknown)
                    } else {
                        lhs.v
                    };
                    Expr::Assign(s, Type::Unknown, Rc::new(place), Rc::new(rhs.v))
                } else {
                    Expr::InfixBinaryOp(s, Type::Unknown, op.v, Rc::new(lhs.v), Rc::new(rhs.v))
                };
                lhs = Spanned::new(s, e);
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
                Ok(Spanned::new(t0.s, Expr::Bool(t0.s, Type::Unknown, v)))
            }
            Token::Int => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Int(s, Type::Unknown, v.into())))
            }
            Token::Float => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Float(s, Type::Unknown, v.into())))
            }
            Token::IntSuffix => {
                self.skip();
                let s = t0.s;
                let v = self.text(t0);
                let i = v.chars().take_while(|c| c.is_digit(10)).count();
                let l = &v[..i];
                let r = &v[i..];
                Ok(Spanned::new(
                    s,
                    Expr::IntSuffix(s, Type::Unknown, l.into(), r.into()),
                ))
            }
            Token::FloatSuffix => {
                self.skip();
                let s = t0.s;
                let v = self.text(t0);
                let i = v
                    .chars()
                    .take_while(|c| c.is_digit(10) || *c == '.')
                    .count();
                let l = &v[..i];
                let r = &v[i..];
                Ok(Spanned::new(
                    s,
                    Expr::FloatSuffix(s, Type::Unknown, l.into(), r.into()),
                ))
            }
            Token::String => {
                self.skip();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::String(s, Type::Unknown, v.into())))
            }
            Token::Char => {
                self.skip();
                let v = self.text(t0).chars().next().unwrap();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Char(s, Type::Unknown, v)))
            }
            Token::Name => {
                let path = self.path(follow)?;
                let s = path.s;
                Ok(Spanned::new(s, Expr::Path(s, Type::Unknown, path.v)))
            }
            Token::LParen => {
                let t = self.expr_args(follow)?;
                let s = t.s;
                if t.v.len() == 1 {
                    let e = t.v.into_iter().next().unwrap();
                    Ok(Spanned::new(s, Expr::Paren(t.s, Type::Unknown, Rc::new(e))))
                } else {
                    Ok(Spanned::new(s, Expr::Tuple(t.s, Type::Unknown, t.v)))
                }
            }
            Token::Minus | Token::Not => {
                let op = self.next();
                let ((), rbp) = op.v.prefix_bp().unwrap();
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = op.s + rhs.s;
                let e = Expr::PrefixUnaryOp(s, Type::Unknown, op.v, Rc::new(rhs.v));
                Ok(Spanned::new(s, e))
            }
            Token::Break => {
                self.skip();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Break(s, Type::Unknown)))
            }
            Token::Continue => {
                self.skip();
                let s = t0.s;
                Ok(Spanned::new(s, Expr::Continue(s, Type::Unknown)))
            }
            Token::Return => {
                self.skip();
                let t = self.start(follow | Expr::FIRST, follow)?;
                if Expr::FIRST.contains(t.v) {
                    let e = self.expr(follow)?;
                    let s = t0.s + e.s;
                    Ok(Spanned::new(
                        s,
                        Expr::Return(s, Type::Unknown, Rc::new(e.v)),
                    ))
                } else {
                    let s = t0.s;
                    let e = Rc::new(Expr::Tuple(s, Type::Unknown, vec![]));
                    Ok(Spanned::new(s, Expr::Return(s, Type::Unknown, e)))
                }
            }
            Token::LBrack => {
                let es = self.brack(Self::exprs, follow)?.flatten();
                let s = t0.s + es.s;
                Ok(Spanned::new(s, Expr::Array(s, Type::Unknown, es.v)))
            }
            Token::Fun => {
                let t = self.next();
                let params = self.inferred_params(follow)?;
                let t1 = self.start(follow | Token::Colon, follow | Token::Eq)?;
                let ty = if t1.v == Token::Colon {
                    self.ty_annot(follow | Token::Eq)?.v
                } else {
                    Type::Unknown
                };
                self.expect(Token::Eq, follow)?;
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    Expr::Fun(s, Type::Unknown, params.v, ty, Rc::new(e.v)),
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
                        Expr::IfElse(s, Type::Unknown, Rc::new(e.v), b0.v, b1.v),
                    ))
                } else {
                    let s = t.s + b0.s;
                    let e1 = Expr::Tuple(s, Type::Unknown, vec![]);
                    let b1 = Block::new(s, vec![], e1);
                    Ok(Spanned::new(
                        s,
                        Expr::IfElse(s, Type::Unknown, Rc::new(e.v), b0.v, b1),
                    ))
                }
            }
            Token::Let => {
                let t = self.next();
                let x = self.name(follow | Token::Eq)?;
                let t1 = self.start(follow | Token::Eq | Token::Colon, follow)?;
                let ty = if t1.v == Token::Eq {
                    Type::Unknown
                } else {
                    self.ty_annot(follow | Token::Eq)?.v
                };
                self.expect(Token::Eq, follow | Expr::FIRST)?;
                let e = self.expr(follow | Token::In)?;
                self.expect(Token::In, follow | Expr::FIRST)?;
                let e0 = self.expr(follow)?;
                let s = t.s + e0.s;
                Ok(Spanned::new(
                    s,
                    Expr::LetIn(s, Type::Unknown, x.v, ty, Rc::new(e.v), Rc::new(e0.v)),
                ))
            }
            Token::Match => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let arms = self.arms(follow)?;
                let s = t.s + arms.s;
                Ok(Spanned::new(
                    s,
                    Expr::Match(s, Type::Unknown, Rc::new(e.v), arms.v.into()),
                ))
            }
            Token::While => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Ok(Spanned::new(
                    s,
                    Expr::While(s, Type::Unknown, Rc::new(e.v), b.v),
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
                    Expr::For(s, Type::Unknown, x.v, Rc::new(e.v), b.v),
                ))
            }
            Token::Record => {
                let t = self.next();
                let es = self
                    .paren(
                        |p, follow| p.seq(Self::field_expr, Token::Comma, Token::Name, follow),
                        follow,
                    )
                    .map(|x| x.flatten())?;
                let s = t.s + es.s;
                Ok(Spanned::new(s, Expr::Record(s, Type::Unknown, es.v.into())))
            }
            Token::From => {
                let t = self.next();
                let x0 = self.name(follow | Token::In)?;
                self.expect(Token::In, follow | Expr::FIRST)?;
                let e = self.expr(follow | Query::FIRST | Token::Into)?;
                let qs = self.repeat(Self::query, Query::FIRST, follow | Query::FOLLOW)?;
                if self.start(follow | Token::Into, follow)?.v == Token::Into {
                    self.skip();
                    let qs = qs.map(|x| x.v).unwrap_or_default();
                    let x1 = self.name(follow)?;
                    let t1 = self.start(follow | Token::LBrack | Token::LParen, follow)?;
                    if t1.v == Token::LBrack || t1.v == Token::LParen {
                        let ts = self.optional_ty_args(follow)?;
                        let es = self.expr_args(follow)?;
                        let s = t.s + es.s;
                        Ok(Spanned::new(
                            s,
                            Expr::QueryInto(
                                s,
                                Type::Unknown,
                                x0.v,
                                Rc::new(e.v),
                                qs,
                                x1.v,
                                ts,
                                es.v,
                            ),
                        ))
                    } else {
                        let s = t.s + x1.s;
                        Ok(Spanned::new(
                            s,
                            Expr::QueryInto(
                                s,
                                Type::Unknown,
                                x0.v,
                                Rc::new(e.v),
                                qs,
                                x1.v,
                                vec![],
                                vec![],
                            ),
                        ))
                    }
                } else {
                    let (qs, s) = if let Some(qs) = qs {
                        (qs.v, t.s + qs.s)
                    } else {
                        (vec![], t.s + e.s)
                    };
                    Ok(Spanned::new(
                        s,
                        Expr::Query(s, Type::Unknown, x0.v, Rc::new(e.v), qs),
                    ))
                }
            }
            Token::LBrace => {
                let b = self.block(follow)?;
                Ok(Spanned::new(b.s, Expr::Block(b.s, Type::Unknown, b.v)))
            }
            t => unreachable!("{:?}", t),
        }
    }

    fn query(&mut self, follow: Token) -> Result<Spanned<Query>, Span> {
        self.query_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Query::Err(s))))
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
                Ok(Spanned::new(s, Query::From(s, x.v, Rc::new(e.v))))
            }
            Token::Where => {
                let t = self.next();
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, Query::Where(s, Rc::new(e.v))))
            }
            Token::Select => {
                let t = self.next();
                let es = self.seq_nonempty(Self::field_expr, Token::Comma, Token::Name, follow)?;
                let s = t.s + es.s;
                Ok(Spanned::new(s, Query::Select(s, es.v.into())))
            }
            Token::Group => {
                let t = self.next();
                let x = self.name(follow | Token::Eq)?;
                self.expect(Token::Eq, follow | Expr::FIRST)?;
                let e0 = self.expr(follow | Token::Over)?;
                self.expect(Token::Over, follow)?;
                let e1 = self.expr(follow | Token::Compute)?;
                self.expect(Token::Compute, follow)?;
                let aggs = self.seq_nonempty(Self::aggr, Token::Comma, Token::Name, follow)?;
                let s = t.s + aggs.s;
                Ok(Spanned::new(
                    s,
                    Query::GroupOverCompute(s, x.v, Rc::new(e0.v), Rc::new(e1.v), aggs.v),
                ))
            }
            Token::Over => {
                let t = self.next();
                let e = self.expr(follow | Token::Compute)?;
                self.expect(Token::Compute, follow | Expr::FIRST)?;
                let aggs = self.seq_nonempty(Self::aggr, Token::Comma, Token::Name, follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, Query::OverCompute(s, Rc::new(e.v), aggs.v)))
            }
            Token::Var => {
                let t = self.next();
                let x = self.name(follow | Token::Eq)?;
                self.expect(Token::Eq, follow)?;
                let e = self.expr(follow | Query::FIRST)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, Query::Var(s, x.v, Rc::new(e.v))))
            }
            Token::Join => {
                let t = self.next();
                let x = self.name(follow | Token::In)?;
                self.expect(Token::In, follow | Expr::FIRST)?;
                let e0 = self.expr(follow | Token::On | Token::Over)?;
                match self.start(Token::On | Token::Over, follow)?.v {
                    Token::On => {
                        self.skip();
                        let e1 = self.expr(follow)?;
                        let s = t.s + e1.s;
                        Ok(Spanned::new(
                            s,
                            Query::JoinOn(s, x.v, Rc::new(e0.v), Rc::new(e1.v)),
                        ))
                    }
                    Token::Over => {
                        self.skip();
                        let e1 = self.expr(follow | Token::On)?;
                        self.expect(Token::On, follow)?;
                        let e2 = self.expr(follow)?;
                        let s = t.s + e2.s;
                        Ok(Spanned::new(
                            s,
                            Query::JoinOverOn(s, x.v, Rc::new(e0.v), Rc::new(e1.v), Rc::new(e2.v)),
                        ))
                    }
                    _ => unreachable!(),
                }
            }
            t => unreachable!("{:?}", t),
        }
    }

    fn aggr(&mut self, follow: Token) -> Result<Spanned<Aggr>, Span> {
        let x = self.name(follow | Token::Eq)?;
        self.expect(Token::Eq, follow)?;
        let e0 = self.expr(follow | Token::Of)?;
        self.expect(Token::Of, follow)?;
        let e1 = self.expr(follow)?;
        let s = x.s + e1.s;
        Ok(Spanned::new(s, Aggr::new(x.v, e0.v, e1.v)))
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
                let path = Path::new_name(x.v);
                let s = x.s;
                Ok(Spanned::new(s, (x.v, Expr::Path(s, Type::Unknown, path))))
            }
        }
    }
}

impl Expr {
    const FIRST: Token = Token::Int
        .or(Token::IntSuffix)
        .or(Token::Float)
        .or(Token::FloatSuffix)
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
        .or(Token::Record)
        .or(Token::From)
        .or(Token::Let);
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

impl Stmt {
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

impl Type {
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Struct)
        .or(Token::Record)
        .or(Token::Fun)
        .or(Token::LBrack)
        .or(Token::Underscore)
        .or(Token::Not);
    const _FOLLOW: Token = Token::Eof;
}

impl Pat {
    const FIRST: Token = Token::Name
        .or(Token::LParen)
        .or(Token::Underscore)
        .or(Token::Int)
        .or(Token::String)
        .or(Token::Struct)
        .or(Token::Record)
        .or(Token::True)
        .or(Token::False)
        .or(Token::Char);
    const FOLLOW: Token = Token::Eof.or(Token::Or).or(Token::Colon);
}

impl Query {
    const FIRST: Token = Token::From
        .or(Token::Where)
        .or(Token::Over)
        .or(Token::Group)
        .or(Token::Var)
        .or(Token::Select)
        .or(Token::Join);
    const FOLLOW: Token = Expr::FOLLOW.or(Query::FIRST).or(Token::Into);
}
