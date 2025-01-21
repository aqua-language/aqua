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
use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::BuiltinDef;
use crate::ast::BuiltinType;
use crate::ast::Effect;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::Index;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::QueryOp;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtLocal;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::collections::map::Map;
use crate::mir;
use crate::report::Diagnostic;
use crate::report::Report;
use crate::syntax::lookahead::Lookahead;
use crate::syntax::span::Span;
use crate::syntax::spanned::Spanned;
use crate::syntax::token::Token;

pub struct Parser<'a, I>
where
    I: Iterator<Item = Spanned<Token>>,
{
    input: &'a str,
    iter: std::iter::Peekable<I>,
    openers: Vec<Spanned<Token>>,
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
    pub fn new(input: &'a str, iter: I) -> Self {
        Self {
            input,
            iter: iter.peekable(),
            report: Report::new(),
            openers: Vec::new(),
        }
    }

    // Utility functions

    /// Peek at the next token
    fn peek(&mut self) -> Spanned<Token> {
        self.iter.peek().cloned().unwrap()
    }

    /// Get the next token
    fn next(&mut self) -> Spanned<Token> {
        self.iter.next().unwrap()
    }

    /// Skip the next token
    fn advance(&mut self) {
        self.iter.next();
    }

    /// Get the text of a token
    fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    fn report_unmatched(&mut self) {
        for t in self.openers.drain(..) {
            let diag = match t.v {
                Token::LParen => Diagnostic::err(t.s, "Unmatched `(`", "expected `)`"),
                Token::LBrace => Diagnostic::err(t.s, "Unmatched `{`", "expected `}`"),
                Token::LBrack => Diagnostic::err(t.s, "Unmatched `[`", "expected `]`"),
                _ => unreachable!(),
            };
            self.report.add(diag);
        }
    }

    /// Error recovery. Discards tokens until `first` or `follow` is found.
    /// Takes closing/opening braces into account:
    /// 1. (+ [)]) <eof> => Recover at the second ) and report an error: Unexpected token +
    /// 2. [ <eof>  => Recover at the end of the file and report an error: Unmatched [
    fn recover(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let mut fuel = 1000; // Stop recovering after having skipped 1000 consecutive tokens.
        loop {
            let t = self.peek();
            match t.v {
                _ if first.contains(t.v) && self.openers.is_empty() => {
                    return Ok(t);
                }
                _ if fuel == 0 || t.v == Token::Eof => {
                    if !self.openers.is_empty() {
                        self.report_unmatched();
                    }
                    return Err(t.s);
                }
                // TODO: Handle what happens when the follow set contains a closing token
                _ if follow.contains(t.v) && self.openers.is_empty() => {
                    return Err(t.s);
                }
                Token::LBrace | Token::LParen | Token::LBrack => {
                    self.advance();
                    self.openers.push(t)
                }
                Token::RBrace | Token::RParen | Token::RBrack
                    if self.openers.last().is_some_and(|t1| t1.v.opens(t.v)) =>
                {
                    self.advance();
                    self.openers.pop();
                }
                _ => self.advance(),
            }
            fuel -= 1;
        }
    }

    fn expect(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        self.start(first, follow)?;
        Ok(self.next())
    }

    fn start(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if first.contains(t.v) {
            Ok(t)
        } else {
            self.report.add(Diagnostic::err(
                t.s,
                format!("Unexpected token `{}`", t.v),
                first.expected(),
            ));
            self.recover(first, follow)
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

    fn or_default<T: Default>(
        &mut self,
        f: impl Fn(&mut Self) -> Result<Spanned<Option<T>>, Span>,
    ) -> Result<Spanned<T>, Span> {
        f(self).map(|x| {
            let v = if let Some(x) = x.v { x } else { T::default() };
            Spanned::new(x.s, v)
        })
    }

    fn paren_seq<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        first: Token,
        follow: Token,
    ) -> Result<Spanned<Vec<T>>, Span> {
        self.or_default(|p| p.paren(|p, follow| p.seq(&f, Token::Comma, first, follow), follow))
    }

    pub fn parse<T>(
        &mut self,
        f: impl FnOnce(&mut Self, Token) -> Result<Spanned<T>, Span>,
    ) -> Result<Spanned<T>, Span> {
        let t = f(self, Token::Eof)?;
        self.expect(Token::Eof, Token::Eof)?;
        Ok(t)
    }

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
                self.report.add(Diagnostic::err(
                    t.s,
                    format!("Invalid index `{}`", v),
                    e.to_string(),
                ));
                Err(t.s)
            }
        }
    }

    fn label(&mut self, follow: Token) -> Result<Spanned<Name>, Span> {
        let t = self.expect(Token::Label, follow | Token::Colon)?;
        self.expect(Token::Colon, follow)?;
        let v = self.text(t).to_owned();
        let x = Name::new(t.s, v);
        Ok(Spanned::new(t.s, x))
    }

    pub fn program(&mut self, follow: Token) -> Result<Spanned<Ast>, Span> {
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
        Ok(Spanned::new(s, Ast::new(s, stmts)))
    }

    pub fn stmt_def_builtin(
        &mut self,
        follow: Token,
        body: BuiltinDef,
    ) -> Result<Spanned<StmtDef>, Span> {
        let t0 = self.expect(Token::Def, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen)?;
        let xts = self.params(follow, true)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::LBrace | Token::Where | Token::SemiColon)?;
        let effect = self.effect(follow | Token::Where | Token::SemiColon)?;
        let where_clause = self.where_clause(follow | Token::SemiColon)?;
        self.expect(Token::SemiColon, follow)?;
        let s = t0.s + x.s;
        let body = ExprBody::Builtin(body);
        Ok(Spanned::new(
            s,
            StmtDef::new(s, x.v, gs, xts.v, t.v, effect, where_clause, body),
        ))
    }

    pub fn effect(&mut self, follow: Token) -> Result<Effect, Span> {
        if self.eat(Token::Tilde, follow | Token::LBrace)? {
            let list = self.brace(
                |p, follow| p.seq(Self::name, Token::Comma, Token::Name, follow),
                follow,
            )?;
            if let Some(list) = list.v {
                let effect = list
                    .into_iter()
                    .fold(Effect::Unknown, |acc, x| Effect::Cons(x, Rc::new(acc)));
                Ok(effect)
            } else {
                Ok(Effect::Nil)
            }
        } else {
            Ok(Effect::Unknown)
        }
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
        let body = TypeBody::Builtin(body);
        Ok(Spanned::new(s, StmtType::new(s, x.v, gs, body)))
    }

    pub fn stmt_impl_builtin(
        &mut self,
        follow: Token,
        bodies: &[BuiltinDef],
    ) -> Result<Spanned<StmtImpl>, Span> {
        let t0 = self.expect(Token::Impl, follow)?;
        let gs = self.generics(follow | Token::Name)?;
        let b = self.bound(follow | Token::Where | Token::LBrace)?;
        let bs = self.where_clause(follow | Token::LBrace)?;
        self.expect(Token::LBrace, follow)?;
        let mut defs = Vec::new();
        let mut tys = Vec::new();
        let mut bodies = bodies.iter().cloned();
        loop {
            let t = self.start(Token::RBrace | Token::Def | Token::Type, follow)?;
            match t.v {
                Token::Def => defs.push(Rc::new(
                    self.stmt_def_builtin(follow | Token::RBrace, bodies.next().unwrap())?
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
            Token::Val | Token::Var => {
                let s = self.stmt_local(follow | Stmt::FOLLOW)?;
                Ok(Spanned::new(s.s, Stmt::Local(Rc::new(s.v))))
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
        let xts = self.params(follow, true)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::LBrace | Token::Where | Token::Eq)?;
        let effect = self.effect(follow | Token::Where | Token::Eq)?;
        let where_clause = self.where_clause(follow | Token::Eq)?;
        self.expect(Token::Eq, follow)?;
        let e = self.stmt_expr(follow)?;
        let s = t0.s + e.s;
        Ok(Spanned::new(
            s,
            StmtDef::new(
                s,
                x.v,
                gs,
                xts.v,
                t.v,
                effect,
                where_clause,
                ExprBody::UserDefined(Rc::new(e.v)),
            ),
        ))
    }

    fn stmt_struct(&mut self, follow: Token) -> Result<Spanned<StmtStruct>, Span> {
        let t0 = self.expect(Token::Struct, follow)?;
        let x = self.name(follow)?;
        let gs = self.generics(follow | Token::LParen | Token::SemiColon)?;
        let xts = self
            .optional(
                |p, follow| p.fields(follow | Token::SemiColon),
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
        let xts = self.variants(follow)?;
        let s = t0.s + xts.s;
        Ok(Spanned::new(s, StmtEnum::new(s, x.v, gs, xts.v)))
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
            StmtType::new(s, x.v, gs, TypeBody::UserDefined(t.v)),
        ))
    }

    fn stmt_local(&mut self, follow: Token) -> Result<Spanned<StmtLocal>, Span> {
        let t0 = self.expect(Token::Val | Token::Var, follow | Token::Name)?;
        let m = t0.v == Token::Var;
        let x = self.name(follow)?;
        let t = self.optional_type_annot(follow | Token::Eq)?;
        let l = Local::new(x.s, x.v, t, m);
        self.expect(Token::Eq, follow)?;
        let e = self.optional(|p, follow| p.expr(follow), Expr::FIRST, follow)?;
        let t1 = self.expect(Token::SemiColon, follow)?;
        if let Some(e) = e {
            let s = t0.s + t1.s;
            Ok(Spanned::new(s, StmtLocal::new(s, l, Some(e.v))))
        } else {
            let s = t0.s;
            Ok(Spanned::new(s, StmtLocal::new(s, l, None)))
        }
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
        let ls = self.params(follow | Token::Colon, true)?;
        self.expect(Token::Colon, follow)?;
        let t = self.ty(follow | Token::Where | Token::SemiColon)?;
        let e = self.effect(follow | Token::Where | Token::SemiColon)?;
        let bs = self.where_clause(follow | Token::SemiColon)?;
        let t1 = self.expect(Token::SemiColon, follow)?;
        let s = t0.s + t1.s;
        Ok(Spanned::new(
            s,
            StmtTraitDef::new(s, x.v, gs, ls.v, t.v, e, bs),
        ))
    }

    fn local(
        &mut self,
        follow: Token,
        type_annotation_required: bool,
    ) -> Result<Spanned<Local>, Span> {
        let mutable = self.eat(Token::Mut, follow | Token::Name)?;
        let x = self.name(follow)?;
        if type_annotation_required {
            self.expect(Token::Colon, follow)?;
            let t = self.ty(follow)?;
            let s = x.s + t.s;
            Ok(Spanned::new(s, Local::new(s, x.v, t.v, mutable)))
        } else {
            let t = self.optional_type_annot(follow | Token::Eq)?;
            let s = x.s;
            Ok(Spanned::new(s, Local::new(s, x.v, t, mutable)))
        }
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

    fn variants(&mut self, follow: Token) -> Result<Spanned<Map<Name, Type>>, Span> {
        self.brace(
            |p, follow| p.seq(Self::variant, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| Spanned::new(x.s, x.v.unwrap_or_default().into()))
    }

    fn variant(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let x = self.name(follow)?;
        if self.start(Token::LParen | follow, follow)?.v == Token::LParen {
            self.advance();
            let t = self.ty(follow | Token::RParen)?;
            self.expect(Token::RParen, follow)?;
            let s = x.s + t.s;
            Ok(Spanned::new(s, (x.v, t.v)))
        } else {
            let s = x.s;
            Ok(Spanned::new(s, (x.v, Type::Unit)))
        }
    }

    fn where_clause(&mut self, follow: Token) -> Result<Vec<Impl>, Span> {
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

    fn bound(&mut self, follow: Token) -> Result<Spanned<Impl>, Span> {
        let x = self.path(follow)?;
        Ok(Spanned::new(x.s, Impl::Path(x.s, x.v)))
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

    fn params(
        &mut self,
        follow: Token,
        type_annotation_required: bool,
    ) -> Result<Spanned<Vec<Local>>, Span> {
        self.paren_seq(
            |ctx, follow| ctx.local(follow, type_annotation_required),
            Token::Name | Token::Mut,
            follow,
        )
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
            |p, follow| {
                p.or_default(|p| {
                    p.brack(
                        |this, follow| this.seq(Self::trait_arg, Token::Comma, Type::FIRST, follow),
                        follow,
                    )
                })
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
                p.or_default(|p| {
                    p.brack(
                        |p, follow| p.seq(Self::ty, Token::Comma, Type::FIRST, follow),
                        follow,
                    )
                })
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

    fn fields(&mut self, follow: Token) -> Result<Spanned<Map<Name, Type>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::field, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| Spanned::new(x.s, x.v.unwrap_or_default().into()))
    }

    pub fn ty(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.ty_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, Type::Err)))
    }

    fn ty_fallible(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.ty_bp(follow, 0)
    }

    fn ty_bp(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Type>, Span> {
        let mut lhs = self
            .ty_lhs(follow | Type::FOLLOW)
            .unwrap_or_else(|s| Spanned::new(s, Type::Err));
        loop {
            let op = self.start(follow | Type::FOLLOW, follow | Type::FOLLOW)?;
            if let Some((lbp, ())) = op.v.ty_postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let t = match op.v {
                    Token::FatArrow => {
                        self.advance();
                        let ts = lhs.v.as_params();
                        let ty = self.ty(follow | Token::LBrace)?;
                        let effect = self.effect(follow)?;
                        Type::Function(ts, Rc::new(ty.v), effect)
                    }
                    _ => unreachable!(),
                };
                let s = lhs.s + op.s;
                lhs = Spanned::new(s, t);
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn ty_lhs(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        let lhs = match self.start(Type::FIRST, follow)?.v {
            Token::Name => {
                let path = self.path(follow)?;
                Spanned::new(path.s, Type::Path(path.v))
            }
            Token::LParen => {
                let t = self.ty_tuple(follow)?;
                let ty = match t.v.len() {
                    0 => Type::Unit,
                    1 => t.v.into_iter().next().unwrap(),
                    _ => Type::Tuple(t.v),
                };
                Spanned::new(t.s, ty)
            }
            Token::Ampersand => {
                let t = self.next();
                let m = self.eat(Token::Mut, follow | Type::FIRST)?;
                let t1 = self.ty(follow)?;
                Spanned::new(t.s + t1.s, Type::Ref(vec![], Rc::new(t1.v), m))
            }
            Token::Record => {
                let t = self.next();
                let fields = self.fields(follow)?;
                Spanned::new(t.s + fields.s, Type::Record(fields.v))
            }
            Token::LBrack => {
                let t = self.next();
                let ty = self.ty(follow | Token::SemiColon)?;
                self.expect(Token::SemiColon, follow)?;
                let n = self.index(follow | Token::RBrack)?;
                let t1 = self.expect(Token::RBrack, follow)?;
                Spanned::new(t.s + t1.s, Type::Array(Rc::new(ty.v), Some(n.v.data)))
            }
            Token::Not => {
                let t = self.next();
                Spanned::new(t.s, Type::Never)
            }
            Token::Underscore => {
                let t = self.next();
                Spanned::new(t.s, Type::Unknown)
            }
            _ => unreachable!(),
        };
        Ok(lhs)
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
            let op = self.start(follow | Pat::FOLLOW, follow | Pat::FOLLOW)?;
            if let Some((lbp, ())) = op.v.pat_postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let e = match op.v {
                    Token::Colon => {
                        self.advance();
                        let ty = self.ty(follow)?;
                        let s = lhs.s + ty.s;
                        Pat::Annotate(s, ty.v, Rc::new(lhs.v))
                    }
                    _ => unreachable!(),
                };
                let s = lhs.s + op.s;
                lhs = Spanned::new(s, e);
            } else if let Some((lbp, rbp)) = op.v.pat_infix_bp() {
                if lbp < min_bp {
                    break;
                }
                match op.v {
                    Token::Or => {
                        self.advance();
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
                    Spanned::new(s, Pat::Path(s, Type::Unknown, path.v, Some(t.v)))
                } else {
                    let s = path.s;
                    Spanned::new(s, Pat::Path(s, Type::Unknown, path.v, None))
                }
            }
            Token::LParen => {
                let t = self.pat_tuple(follow)?;
                let p = match t.v.len() {
                    0 => Pat::Unit(t.s, Type::Unknown),
                    1 => t.v.into_iter().next().unwrap(),
                    _ => Pat::Tuple(t.s, Type::Unknown, t.v),
                };
                Spanned::new(t.s, p)
            }
            Token::Record => {
                let t0 = self.next();
                let xps = self.pat_fields(follow)?;
                let s = t0.s + xps.s;
                Spanned::new(s, Pat::Record(s, Type::Unknown, xps.v))
            }
            Token::Underscore => {
                let t = self.next();
                Spanned::new(t.s, Pat::Wildcard(t.s, Type::Unknown))
            }
            Token::Int => {
                let t = self.next();
                let v = self.text(t).into();
                Spanned::new(t.s, Pat::Int(t.s, Type::Unknown, v))
            }
            Token::String => {
                let t = self.next();
                let v = self.text(t).into();
                Spanned::new(t.s, Pat::String(t.s, Type::Unknown, v))
            }
            Token::Char => {
                let t = self.next();
                let v = self.text(t).chars().next().unwrap();
                Spanned::new(t.s, Pat::Char(t.s, Type::Unknown, v))
            }
            Token::True | Token::False => {
                let t = self.next();
                let v = t.v == Token::True;
                Spanned::new(t.s, Pat::Bool(t.s, Type::Unknown, v))
            }
            _ => unreachable!(),
        };
        Ok(lhs)
    }

    fn ty_annot(&mut self, follow: Token) -> Result<Spanned<Type>, Span> {
        self.expect(Token::Colon, follow | Type::FIRST)?;
        self.ty(follow)
    }

    fn expr_args(&mut self, follow: Token) -> Result<Spanned<Vec<Expr>>, Span> {
        self.paren_seq(Self::expr, Expr::FIRST, follow)
    }

    fn ty_tuple(&mut self, follow: Token) -> Result<Spanned<Vec<Type>>, Span> {
        self.or_default(|p| {
            p.paren(
                |p, follow| p.seq(Self::ty, Token::Comma, Type::FIRST, follow),
                follow,
            )
        })
    }

    fn pat_tuple(&mut self, follow: Token) -> Result<Spanned<Vec<Pat>>, Span> {
        self.or_default(|p| {
            p.paren(
                |p, follow| p.seq(Self::pat, Token::Comma, Pat::FIRST, follow),
                follow,
            )
        })
    }

    fn pat_args(&mut self, follow: Token) -> Result<Spanned<Vec<PathPatField>>, Span> {
        self.or_default(|p| {
            p.paren(
                |p, follow| p.seq(Self::pat_arg, Token::Comma, Pat::FIRST, follow),
                follow,
            )
        })
    }

    fn pat_arg(&mut self, follow: Token) -> Result<Spanned<PathPatField>, Span> {
        let mut p0 = self.pat(follow | Token::Eq)?;
        if let Pat::Path(_, Type::Unknown, path, fields) = &mut p0.v {
            let t1 = self.start(Token::Eq | follow, follow)?;
            if path.as_name().is_some() && fields.is_none() && t1.v == Token::Eq {
                self.advance();
                // x = p
                let x = path.segments.pop().unwrap().x;
                let p1 = self.pat(follow)?;
                let s = p0.s + p1.s;
                return Ok(Spanned::new(s, PathPatField::Named(x, p1.v)));
            }
        }
        let s = p0.s;
        Ok(Spanned::new(s, PathPatField::Unnamed(p0.v)))
    }

    fn pat_fields(&mut self, follow: Token) -> Result<Spanned<Map<Name, Pat>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::field_pat, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| Spanned::new(x.s, x.v.unwrap_or_default().into()))
    }

    fn field(&mut self, follow: Token) -> Result<Spanned<(Name, Type)>, Span> {
        let x = self.name(follow)?;
        let t = self.optional(|p, follow| p.ty_annot(follow), Token::Colon, follow)?;
        if let Some(t) = t {
            let s = x.s + t.s;
            Ok(Spanned::new(s, (x.v, t.v)))
        } else {
            let s = x.s;
            Ok(Spanned::new(s, (x.v, Type::Unknown)))
        }
    }

    fn field_pat(&mut self, follow: Token) -> Result<Spanned<(Name, Pat)>, Span> {
        let x = self.name(follow)?;
        if self.start(Token::Eq | follow, follow)?.v == Token::Eq {
            self.advance();
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
        self.or_default(|p| {
            p.brace(
                |p, follow| p.seq(Self::arm, Token::Comma, Pat::FIRST, follow),
                follow,
            )
        })
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
    /// * { s ... } - Where s is a block expression
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
                    return Ok(Spanned::new(s, Block::new(s, stmts, None)));
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
                        // { ... e }
                        let t1 = self.expect(Token::RBrace, follow)?;
                        let s = t0.s + t1.s;
                        return Ok(Spanned::new(s, Block::new(s, stmts, Some(expr.v))));
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
            if let Some((lbp, ())) = op.v.expr_postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                let e = match op.v {
                    Token::LParen => {
                        let args = self.expr_args(follow)?;
                        let s = lhs.s + args.s;
                        Expr::Call(s, Type::Unknown, Rc::new(lhs.v), args.v)
                    }
                    Token::Colon => {
                        self.advance();
                        let ty = self.ty(follow)?;
                        let s = lhs.s + ty.s;
                        Expr::Annotate(s, ty.v, Rc::new(lhs.v))
                    }
                    Token::FatArrow => {
                        self.advance();
                        if let Some(ls) = lhs.v.as_locals() {
                            let rhs = self.expr(follow)?;
                            let e = Rc::new(rhs.v);
                            let s = lhs.s + rhs.s;
                            Expr::Lambda(s, Type::Unknown, ls, Type::Unknown, e)
                        } else {
                            self.report.add(Diagnostic::err(
                                lhs.s,
                                "Expected function parameters",
                                "Found expression",
                            ));
                            Expr::Err(lhs.s, Type::Unknown)
                        }
                    }
                    Token::Dot => {
                        self.advance();
                        let t = self.start(
                            Token::Name | Token::Int | Token::Star | Token::Ampersand,
                            follow,
                        )?;
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
                            Token::Star => {
                                let t = self.next();
                                let s = lhs.s + t.s;
                                Expr::Deref(s, Type::Unknown, Rc::new(lhs.v))
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                };
                let s = lhs.s + op.s;
                lhs = Spanned::new(s, e);
            } else if let Some((lbp, rbp)) = op.v.expr_infix_bp() {
                if lbp < min_bp {
                    break;
                }
                self.advance();
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = lhs.s + rhs.s;
                let e = if Token::Eq == op.v {
                    Expr::Assign(s, Type::Unknown, Rc::new(lhs.v), Rc::new(rhs.v))
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
        let lhs = match t0.v {
            Token::True | Token::False => {
                self.advance();
                let v = t0.v == Token::True;
                Spanned::new(t0.s, Expr::Bool(t0.s, Type::Unknown, v))
            }
            Token::Int => {
                self.advance();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Spanned::new(s, Expr::Int(s, Type::Unknown, v.into()))
            }
            Token::Float => {
                self.advance();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Spanned::new(s, Expr::Float(s, Type::Unknown, v.into()))
            }
            Token::IntSuffix => {
                self.advance();
                let s = t0.s;
                let v = self.text(t0);
                let i = v.chars().take_while(|c| c.is_digit(10)).count();
                let l = &v[..i];
                let r = &v[i..];
                Spanned::new(s, Expr::IntSuffix(s, Type::Unknown, l.into(), r.into()))
            }
            Token::FloatSuffix => {
                self.advance();
                let s = t0.s;
                let v = self.text(t0);
                let i = v
                    .chars()
                    .take_while(|c| c.is_digit(10) || *c == '.')
                    .count();
                let l = &v[..i];
                let r = &v[i..];
                Spanned::new(s, Expr::FloatSuffix(s, Type::Unknown, l.into(), r.into()))
            }
            Token::String => {
                self.advance();
                let v = self.text(t0).to_owned();
                let s = t0.s;
                Spanned::new(s, Expr::String(s, Type::Unknown, v.into()))
            }
            Token::Char => {
                self.advance();
                let v = self.text(t0).chars().next().unwrap();
                let s = t0.s;
                Spanned::new(s, Expr::Char(s, Type::Unknown, v))
            }
            Token::Name => {
                let path = self.path(follow)?;
                let s = path.s;
                Spanned::new(s, Expr::Path(s, Type::Unknown, path.v))
            }
            Token::LParen => {
                let t = self.expr_args(follow)?;
                let s = t.s;
                match t.v.len() {
                    0 => Spanned::new(s, Expr::Unit(s, Type::Unknown)),
                    1 => {
                        let e = t.v.into_iter().next().unwrap();
                        Spanned::new(s, Expr::Paren(t.s, Type::Unknown, Rc::new(e)))
                    }
                    _ => Spanned::new(s, Expr::Tuple(t.s, Type::Unknown, t.v)),
                }
            }
            Token::Minus | Token::Not => {
                let op = self.next();
                let ((), rbp) = op.v.expr_prefix_bp().unwrap();
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = op.s + rhs.s;
                let e = Expr::PrefixUnaryOp(s, Type::Unknown, op.v, Rc::new(rhs.v));
                Spanned::new(s, e)
            }
            Token::Star => {
                let op = self.next();
                let ((), rbp) = op.v.expr_prefix_bp().unwrap();
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = op.s + rhs.s;
                Spanned::new(s, Expr::Deref(s, Type::Unknown, Rc::new(rhs.v)))
            }
            Token::Ampersand => {
                let op = self.next();
                let m = self.eat(Token::Mut, follow | Expr::FOLLOW)?;
                let ((), rbp) = op.v.expr_prefix_bp().unwrap();
                let rhs = self.expr_fallible(follow, rbp)?;
                let s = op.s + rhs.s;
                Spanned::new(s, Expr::Ref(s, Type::Unknown, Rc::new(rhs.v), m))
            }
            Token::Break => {
                self.advance();
                let label = self.optional(Self::label, Token::Label, follow)?;
                let s = t0.s + label.map(|x| x.s);
                Spanned::new(s, Expr::Break(s, Type::Unknown, label.map(|x| x.v)))
            }
            Token::Continue => {
                self.advance();
                let label = self.optional(Self::label, Token::Label, follow)?;
                let s = t0.s + label.map(|x| x.s);
                Spanned::new(s, Expr::Continue(s, Type::Unknown, label.map(|x| x.v)))
            }
            Token::Return => {
                self.advance();
                let t = self.start(follow | Expr::FIRST, follow)?;
                if Expr::FIRST.contains(t.v) {
                    let e = self.expr(follow)?;
                    let s = t0.s + e.s;
                    Spanned::new(s, Expr::Return(s, Type::Unknown, Rc::new(e.v)))
                } else {
                    let s = t0.s;
                    let e = Rc::new(Expr::Unit(s, Type::Unknown));
                    Spanned::new(s, Expr::Return(s, Type::Unknown, e))
                }
            }
            Token::LBrack => {
                let es = self.or_default(|p| p.brack(Self::exprs, follow))?;
                let s = t0.s + es.s;
                Spanned::new(s, Expr::Array(s, Type::Unknown, es.v))
            }
            Token::If => {
                let t = self.next();
                let e0 = self.expr(follow | Token::LBrace)?;
                let b0 = self.block(follow | Token::Else | Stmt::FIRST)?;
                if self.start(follow | Token::Else | Stmt::FIRST, follow)?.v == Token::Else {
                    self.advance();
                    let b1 = self.block(follow)?;
                    let s = t.s + b1.s;
                    Spanned::new(
                        s,
                        Expr::IfElse(
                            s,
                            Type::Unknown,
                            Rc::new(e0.v),
                            Rc::new(b0.v),
                            Rc::new(b1.v),
                        ),
                    )
                } else {
                    let s = t.s + b0.s;
                    let b1 = Block::new(s, vec![], None);
                    Spanned::new(
                        s,
                        Expr::IfElse(s, Type::Unknown, Rc::new(e0.v), Rc::new(b0.v), Rc::new(b1)),
                    )
                }
            }
            Token::Match => {
                let t = self.next();
                let e = self.expr(follow | Token::LBrace)?;
                let arms = self.arms(follow)?;
                let s = t.s + arms.s;
                Spanned::new(
                    s,
                    Expr::Match(s, Type::Unknown, Rc::new(e.v), arms.v.into()),
                )
            }
            Token::While => {
                let t = self.next();
                let l = self
                    .optional(Self::label, Token::Label, follow)?
                    .map(|x| x.v);
                let e = self.expr(follow | Token::LBrace)?;
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Spanned::new(
                    s,
                    Expr::While(s, Type::Unknown, l, Rc::new(e.v), Rc::new(b.v)),
                )
            }
            Token::Loop => {
                let t = self.next();
                let l = self
                    .optional(Self::label, Token::Label, follow | Token::LBrace)?
                    .map(|x| x.v);
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Spanned::new(s, Expr::Loop(s, Type::Unknown, l, Rc::new(b.v)))
            }
            Token::For => {
                let t = self.next();
                let lab = self
                    .optional(Self::label, Token::Label, follow)?
                    .map(|x| x.v);
                let l = self.local(follow, false)?;
                self.expect(Token::In, follow)?;
                let e = self.expr(follow | Token::LBrace)?;
                let b = self.block(follow)?;
                let s = t.s + b.s;
                Spanned::new(
                    s,
                    Expr::For(s, Type::Unknown, lab, l.v, Rc::new(e.v), Rc::new(b.v)),
                )
            }
            Token::Record => {
                let t = self.next();
                let es = self.expr_fields(follow | Token::RBrace)?;
                let s = t.s + es.s;
                Spanned::new(s, Expr::Record(s, Type::Unknown, es.v))
            }
            Token::From => {
                let t = self.next();
                let l = self.local(follow | Token::In, false)?;
                self.expect(Token::In, follow | Expr::FIRST)?;
                let e = self.expr(follow | QueryOp::FIRST | Token::Into)?;
                let qs = self.repeat(Self::query_op, QueryOp::FIRST, follow | QueryOp::FOLLOW)?;
                if self.start(follow | Token::Into, follow)?.v == Token::Into {
                    self.advance();
                    let qs = qs.map(|x| x.v).unwrap_or_default();
                    let x1 = self.name(follow)?;
                    let t1 = self.start(follow | Token::LBrack | Token::LParen, follow)?;
                    if t1.v == Token::LBrack || t1.v == Token::LParen {
                        let ts = self.optional_ty_args(follow)?;
                        let es = self.expr_args(follow)?;
                        let s = t.s + es.s;
                        Spanned::new(
                            s,
                            Expr::QueryInto(
                                s,
                                Type::Unknown,
                                l.v,
                                Rc::new(e.v),
                                qs,
                                x1.v,
                                ts,
                                es.v,
                            ),
                        )
                    } else {
                        let s = t.s + x1.s;
                        Spanned::new(
                            s,
                            Expr::QueryInto(
                                s,
                                Type::Unknown,
                                l.v,
                                Rc::new(e.v),
                                qs,
                                x1.v,
                                vec![],
                                vec![],
                            ),
                        )
                    }
                } else {
                    let (qs, s) = if let Some(qs) = qs {
                        (qs.v, t.s + qs.s)
                    } else {
                        (vec![], t.s + e.s)
                    };
                    Spanned::new(s, Expr::Query(s, Type::Unknown, l.v, Rc::new(e.v), qs))
                }
            }
            Token::LBrace => {
                let b = self.block(follow)?;
                Spanned::new(b.s, Expr::Block(b.s, Type::Unknown, Rc::new(b.v)))
            }
            Token::Underscore => {
                let t = self.next();
                if self.start(follow | Token::ColonColon, follow)?.v == Token::ColonColon {
                    self.next();
                    let x = self.name(follow)?;
                    let ts = self.optional_ty_args(follow)?;
                    let s = t.s + x.s;
                    Spanned::new(s, Expr::Assoc(s, Type::Unknown, Impl::Unknown, x.v, ts))
                } else {
                    Spanned::new(t.s, Expr::Anonymous(t.s, Type::Unknown))
                }
            }
            t => unreachable!("{:?}", t),
        };
        Ok(lhs)
    }

    fn query_op(&mut self, follow: Token) -> Result<Spanned<QueryOp>, Span> {
        self.query_op_fallible(follow)
            .or_else(|s| Ok(Spanned::new(s, QueryOp::Err(s))))
    }

    fn optional_type_annot(&mut self, follow: Token) -> Result<Type, Span> {
        Ok(self
            .optional(Self::ty_annot, Token::Colon, follow | Token::In)?
            .map(|x| x.v)
            .unwrap_or(Type::Unknown))
    }

    fn query_op_fallible(&mut self, follow: Token) -> Result<Spanned<QueryOp>, Span> {
        let t = self.start(QueryOp::FIRST, follow)?;
        match t.v {
            Token::From => {
                let t = self.next();
                let l = self.local(follow | Token::In, false)?;
                self.expect(Token::In, follow)?;
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, QueryOp::From(s, l.v, Rc::new(e.v))))
            }
            Token::Where => {
                let t = self.next();
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, QueryOp::Where(s, Rc::new(e.v))))
            }
            Token::Limit => {
                let t = self.next();
                let e = self.expr(follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, QueryOp::Limit(s, Rc::new(e.v))))
            }
            Token::Select => {
                let t = self.next();
                let es = self.seq_nonempty(Self::local_expr, Token::Comma, Token::Name, follow)?;
                let s = t.s + es.s;
                Ok(Spanned::new(s, QueryOp::Select(s, es.v)))
            }
            Token::Group => {
                let t = self.next();
                let l = self.local(follow | Token::Eq, false)?;
                self.expect(Token::Eq, follow | Expr::FIRST)?;
                let e0 = self.expr(follow | Token::Over)?;
                self.expect(Token::Over, follow)?;
                let e1 = self.expr(follow | Token::Compute)?;
                self.expect(Token::Compute, follow)?;
                let aggs = self.seq_nonempty(Self::aggr, Token::Comma, Token::Name, follow)?;
                let s = t.s + aggs.s;
                Ok(Spanned::new(
                    s,
                    QueryOp::GroupOverCompute(s, l.v, Rc::new(e0.v), Rc::new(e1.v), aggs.v),
                ))
            }
            Token::Over => {
                let t = self.next();
                let e = self.expr(follow | Token::Compute)?;
                self.expect(Token::Compute, follow | Expr::FIRST)?;
                let aggs = self.seq_nonempty(Self::aggr, Token::Comma, Token::Name, follow)?;
                let s = t.s + e.s;
                Ok(Spanned::new(
                    s,
                    QueryOp::OverCompute(s, Rc::new(e.v), aggs.v),
                ))
            }
            Token::Var => {
                let t = self.next();
                let l = self.local(follow | Token::Eq, false)?;
                self.expect(Token::Eq, follow)?;
                let e = self.expr(follow | QueryOp::FIRST)?;
                let s = t.s + e.s;
                Ok(Spanned::new(s, QueryOp::Local(s, l.v, Rc::new(e.v))))
            }
            Token::Drop => {
                let t = self.next();
                let x = self.name(follow)?;
                let s = t.s + x.s;
                Ok(Spanned::new(s, QueryOp::Drop(s, x.v)))
            }
            Token::Join => {
                let t = self.next();
                let l = self.local(follow | Token::In, false)?;
                self.expect(Token::In, follow | Expr::FIRST)?;
                let e0 = self.expr(follow | Token::On | Token::Over)?;
                match self.start(Token::On | Token::Over, follow)?.v {
                    Token::On => {
                        self.advance();
                        let e1 = self.expr(follow)?;
                        let s = t.s + e1.s;
                        Ok(Spanned::new(
                            s,
                            QueryOp::JoinOn(s, l.v, Rc::new(e0.v), Rc::new(e1.v)),
                        ))
                    }
                    Token::Over => {
                        self.advance();
                        let e1 = self.expr(follow | Token::On)?;
                        self.expect(Token::On, follow)?;
                        let e2 = self.expr(follow)?;
                        let s = t.s + e2.s;
                        Ok(Spanned::new(
                            s,
                            QueryOp::JoinOverOn(
                                s,
                                l.v,
                                Rc::new(e0.v),
                                Rc::new(e1.v),
                                Rc::new(e2.v),
                            ),
                        ))
                    }
                    _ => unreachable!(),
                }
            }
            t => unreachable!("{:?}", t),
        }
    }

    fn aggr(&mut self, follow: Token) -> Result<Spanned<Aggr>, Span> {
        let l = self.local(follow | Token::Eq, false)?;
        self.expect(Token::Eq, follow)?;
        let e0 = self.name(follow | Token::Of)?;
        self.expect(Token::Of, follow)?;
        let e1 = self.expr(follow | Token::If)?;
        let (s, e2) = if self.start(Token::If | follow, follow)?.v == Token::If {
            self.advance();
            let e2 = self.expr(follow)?;
            (l.s + e2.s, Some(e2.v))
        } else {
            (l.s + e1.s, None)
        };
        Ok(Spanned::new(s, Aggr::new(l.v, e0.v, e1.v, e2)))
    }

    fn field_expr(&mut self, follow: Token) -> Result<Spanned<(Name, Expr)>, Span> {
        let e0 = self.expr(follow)?;
        if let Some((x, e)) = e0.v.as_field_expr() {
            Ok(Spanned::new(e0.s, (*x, e.clone())))
        } else {
            self.report.add(Diagnostic::err(
                e0.s,
                "expected field expression",
                "found expression",
            ));
            Err(e0.s)
        }
    }

    fn local_expr(&mut self, follow: Token) -> Result<Spanned<(Local, Expr)>, Span> {
        let m = self.eat(Token::Mut, follow | Token::Name)?;
        let e0 = self.expr(follow)?;
        if let Some((x, e)) = e0.v.as_field_expr() {
            Ok(Spanned::new(
                e0.s,
                (Local::new(x.span, *x, Type::Unknown, m), e.clone()),
            ))
        } else {
            self.report.add(Diagnostic::err(
                e0.s,
                "expected field expression",
                "found expression",
            ));
            Err(e0.s)
        }
    }

    fn expr_fields(&mut self, follow: Token) -> Result<Spanned<Map<Name, Expr>>, Span> {
        self.paren(
            |p, follow| p.seq(Self::field_expr, Token::Comma, Token::Name, follow),
            follow,
        )
        .map(|x| Spanned::new(x.s, x.v.unwrap_or_default().into()))
    }

    // MIR parsing
    fn mir_function(&mut self, follow: Token) -> Result<Spanned<mir::Function>, Span> {
        
    }
}
