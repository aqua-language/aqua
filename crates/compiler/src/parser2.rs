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

use std::rc::Rc;

use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::Type;
use crate::ast::UnresolvedPath;
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
    parens: Vec<Span>,
    braces: Vec<Span>,
    bracks: Vec<Span>,
}

enum Either<A, B> {
    A(A),
    B(B),
}

impl Token {
    fn infix_bp(self) -> Option<(u8, u8)> {
        let bp = match self {
            Token::DotDot => (1, 2),
            Token::And | Token::Or => (1, 2),
            Token::EqEq | Token::NotEq | Token::Lt | Token::Gt | Token::Le | Token::Ge => (2, 3),
            Token::Plus | Token::Minus => (3, 4),
            Token::Star | Token::Slash => (4, 5),
            _ => return None,
        };
        Some(bp)
    }
    fn prefix_bp(self) -> Option<((), u8)> {
        let bp = match self {
            Token::Plus | Token::Minus => ((), 6),
            _ => return None,
        };
        Some(bp)
    }
    fn postfix_bp(self) -> Option<(u8, ())> {
        let bp = match self {
            Token::Question | Token::LParen => (7, ()),
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
            parens: Vec::new(),
            braces: Vec::new(),
            bracks: Vec::new(),
        }
    }

    // Utility functions

    fn peek(&mut self) -> Spanned<Token> {
        self.lexer.peek().cloned().unwrap()
    }

    fn next(&mut self) -> Spanned<Token> {
        self.lexer.next().unwrap()
    }

    fn skip(&mut self) {
        self.next();
    }

    fn text(&self, t: Spanned<Token>) -> &'a str {
        t.text(self.input)
    }

    fn optional(&mut self, token: Token, follow: Token) -> Option<Spanned<Token>> {
        if follow.contains(self.peek().value) {
            return None;
        }
        self.start(token | follow, follow).ok().and_then(|t| {
            if t.value == token {
                Some(t)
            } else {
                None
            }
        })
    }

    fn eat(&mut self, token: Token, follow: Token) -> bool {
        self.optional(token, follow).is_some()
    }

    fn eat_all(&mut self, token: Token, follow: Token) {
        while self.eat(token, follow) {}
    }

    fn at(&mut self, token: Token) -> bool {
        self.peek().value == token
    }

    fn peek_span(&mut self) -> Span {
        self.peek().span
    }

    fn next_span(&mut self) -> Span {
        self.next().span
    }

    fn balanced(&mut self) -> bool {
        self.parens.is_empty() && self.braces.is_empty() && self.bracks.is_empty()
    }

    // Error recovery takes closing/opening braces into account
    // 1. (+ [)]) => Recover at the second ) and report an error: Unexpected token +
    // 2. <Open>  => Recover at the end of the file and report an error: Unmatched <Open>
    fn recover(&mut self, first: Token, follow: Token, peek: bool) -> Result<Spanned<Token>, Span> {
        loop {
            let t = self.peek();
            match t.value {
                _ if first.contains(t.value) && self.balanced() => {
                    if peek {
                        return Ok(t);
                    } else {
                        return Ok(self.next());
                    }
                }
                // Should this be here or after?
                // Do we need to check if parens/braces/bracks are balanced?
                // * A closing token can be in follow, e.g., (a,b,c)
                // * An opening token can be in follow, e.g., f()
                _ if follow.contains(t.value) => {
                    if !self.balanced() {
                        self.parens.drain(..).for_each(|span| {
                            self.report.err(span, "Unmatched `(`", "expected `)`");
                        });
                        self.braces.drain(..).for_each(|span| {
                            self.report.err(span, "Unmatched `{`", "expected `}`");
                        });
                        self.bracks.drain(..).for_each(|span| {
                            self.report.err(span, "Unmatched `[`", "expected `]`");
                        });
                    }
                    return Err(t.span);
                }
                Token::LBrace => self.braces.push(t.span),
                Token::LParen => self.parens.push(t.span),
                Token::LBrack => self.bracks.push(t.span),
                Token::RBrace => {
                    self.braces.pop();
                }
                Token::RParen => {
                    self.parens.pop();
                }
                Token::RBrack => {
                    self.bracks.pop();
                }
                _ => {
                    self.skip();
                }
            };
        }
    }

    fn expect(&mut self, expected: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if t.value == expected {
            Ok(self.next())
        } else {
            self.report.err(
                t.span,
                format!("Unexpected token {}", t.value),
                format!("expected {expected}"),
            );
            self.recover(expected, follow, false)
        }
    }

    fn start(&mut self, first: Token, follow: Token) -> Result<Spanned<Token>, Span> {
        let t = self.peek();
        if !first.contains(t.value) {
            self.report.err(
                t.span,
                format!("Unexpected token {}", t.value),
                format!("expected one of {:?}", first),
            );
            self.recover(first, follow, true)
        } else {
            Ok(t)
        }
    }

    fn expect_token(&mut self, expected: Token, follow: Token) -> Result<Token, Span> {
        self.expect(expected, follow).map(|t| t.value)
    }

    fn seq<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        sep: Token,
        follow: Token,
    ) -> Result<Option<Spanned<Vec<T>>>, Span> {
        let mut xs = Vec::new();
        if !follow.contains(self.peek().value) {
            let x = f(self, follow | sep)?;
            let s0 = x.span;
            let mut s1 = x.span;
            xs.push(x.value);
            while self.eat(sep, follow) {
                let x = f(self, follow | sep)?;
                s1 = x.span;
                xs.push(x.value);
            }
            return Ok(Some(Spanned::new(s0 + s1, xs)));
        }
        Ok(None)
    }

    fn seq_nonempty<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
        sep: Token,
        follow: Token,
    ) -> Result<Spanned<Vec<T>>, Span> {
        let x = f(self, follow | sep)?;
        let s0 = x.span;
        let mut s1 = x.span;
        let mut xs = vec![x.value];
        while self.eat(sep, follow) {
            let x = f(self, follow | sep)?;
            s1 = x.span;
            xs.push(x.value);
        }
        Ok(Spanned::new(s0 + s1, xs))
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
        Ok(Spanned::new(t0.span + t1.span, x.map(|x| x.value)))
    }

    // Terminals

    fn name(&mut self, follow: Token) -> Result<Spanned<Name>, Span> {
        let t = self.expect(Token::Name, follow)?;
        let v = self.text(t).to_owned();
        let name = Name::new(t.span, v);
        Ok(Spanned::new(t.span, name))
    }

    fn index(&mut self, follow: Token) -> Result<Spanned<Index>, Span> {
        let t = self.expect(Token::Int, follow)?;
        let v = self.text(t);
        match v.parse() {
            Ok(v) => Ok(Spanned::new(t.span, Index::new(t.span, v))),
            Err(e) => {
                self.report
                    .err(t.span, format!("Invalid index `{}`", v), format!("{e}"));
                Err(t.span)
            }
        }
    }

    pub fn parse<T>(
        &mut self,
        f: impl Fn(&mut Self, Token) -> Result<Spanned<T>, Span>,
    ) -> Option<T> {
        let t = f(self, Token::Eof);
        self.expect(Token::Eof, Token::Eof).ok();
        t.ok().map(|x| x.value)
    }

    // The parser
    pub fn program(&mut self, follow: Token) -> Result<Spanned<Program>, Span> {
        let mut stmts = Vec::new();
        let s0 = self.peek_span();
        let mut s1 = s0;
        
        while !follow.contains(self.peek().value) {
            println!("{:?}", self.peek().value);
            match self.stmt(follow | Token::SemiColon) {
                Ok(stmt) => {
                    s1 = stmt.span;
                    stmts.push(stmt.value);
                }
                Err(span) => {
                    s1 = span;
                    stmts.push(Stmt::Err(span));
                }
            }
            self.eat_all(Token::SemiColon, follow);
        }
        Ok(Spanned::new(s0 + s1, Program::new(stmts)))
    }

    pub fn stmt(&mut self, follow: Token) -> Result<Spanned<Stmt>, Span> {
        let t = self.start(
            Token::Def
                | Token::Type
                | Token::Trait
                | Token::Struct
                | Token::Enum
                | Token::Impl
                | Token::Var,
            follow,
        )?;
        match t.value {
            Token::Def => self.stmt_def(follow),
            t => todo!("{t}"),
        }
    }

    fn where_clause(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Bound>>>, Span> {
        if follow.contains(self.peek().value) {
            return Ok(None);
        }
        if let Some(t0) = self.optional(Token::Where, follow) {
            let xs = self.bounds(follow)?;
            if let Some(xs) = xs {
                Ok(Some(Spanned::new(t0.span + xs.span, xs.value)))
            } else {
                Ok(Some(Spanned::new(t0.span, vec![])))
            }
        } else {
            Ok(None)
        }
    }

    fn bounds(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Bound>>>, Span> {
        self.seq(Self::bound, Token::Comma, follow)
    }

    fn bound(&mut self, follow: Token) -> Result<Spanned<Bound>, Span> {
        let x = self.path(follow)?;
        Ok(Spanned::new(x.span, Bound::Unresolved(x.span, x.value)))
    }

    fn path(&mut self, follow: Token) -> Result<Spanned<UnresolvedPath>, Span> {
        let xs = self.seq_nonempty(Self::path_segment, Token::ColonColon, follow)?;
        Ok(Spanned::new(xs.span, UnresolvedPath::new(xs.value)))
    }

    fn ty_args(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Type>>>, Span> {
        if self.peek().value == Token::LBrack {
            self.group(
                Token::LBrack,
                Token::RBrack,
                |p, follow| p.seq(Self::ty, Token::Comma, follow),
                follow,
            )
            .map(|x| x.transpose())
        } else {
            Ok(None)
        }
    }

    fn path_segment(&mut self, follow: Token) -> Result<Spanned<(Name, Vec<Type>)>, Span> {
        let name = self.name(follow)?;
        let args = self.ty_args(follow)?;
        if let Some(args) = args {
            Ok(Spanned::new(
                name.span + args.span,
                (name.value, args.value),
            ))
        } else {
            Ok(Spanned::new(name.span, (name.value, vec![])))
        }
    }

    fn ty(&mut self, _follows: Token) -> Result<Spanned<Type>, Span> {
        Ok(Spanned::new(Span::default(), Type::Err))
    }

    fn generics(&mut self, follow: Token) -> Result<Option<Spanned<Vec<Name>>>, Span> {
        if self.peek().value == Token::LBrack {
            self.group(
                Token::LBrack,
                Token::RBrack,
                |p, follow| p.seq(Self::name, Token::Comma, follow),
                follow,
            )
            .map(|x| x.transpose())
        } else {
            Ok(None)
        }
    }

    fn params(&mut self, follow: Token) -> Result<Spanned<Vec<Param>>, Span> {
        self.group(
            Token::LParen,
            Token::RParen,
            |p, follow| p.seq(Self::param, Token::Comma, follow),
            follow,
        )
        .map(|x| x.flatten())
    }

    fn param(&mut self, follow: Token) -> Result<Spanned<Param>, Span> {
        let name = self.name(follow)?;
        self.expect(Token::Colon, follow)?;
        let ty = self.ty(follow)?;
        let span = name.span + ty.span;
        Ok(Spanned::new(span, Param::new(span, name.value, ty.value)))
    }

    fn stmt_def(&mut self, follow: Token) -> Result<Spanned<Stmt>, Span> {
        let t0 = self.expect(Token::Def, follow)?;
        let name = self.name(follow)?;
        let generics = self.generics(follow)?;
        let params = self.params(follow)?;
        self.expect(Token::Colon, follow)?;
        let ty = self.ty(follow.or(Token::Where).or(Token::Eq))?;
        let where_clause = self.where_clause(follow.or(Token::Eq))?;
        self.expect(Token::Eq, follow)?;
        let e = self.expr(follow)?;
        Ok(Spanned::new(
            t0.span + e.span,
            Stmt::Def(StmtDef::new(
                t0.span + e.span,
                name.value,
                generics.map(|x| x.value).unwrap_or_default(),
                where_clause.map(|x| x.value).unwrap_or_default(),
                params.value,
                ty.value,
                Body::Expr(e.value),
            )),
        ))
    }

    fn expr_tuple(&mut self, follow: Token) -> Result<Spanned<Option<Vec<Expr>>>, Span> {
        self.group(
            Token::LParen,
            Token::RParen,
            |p, follow| p.seq(Self::expr, Token::Comma, follow),
            follow,
        )
    }

    fn expr_lhs(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        let t = self.start(
            Token::Int
                | Token::Float
                | Token::String
                | Token::Name
                | Token::LParen
                | Token::Plus
                | Token::Minus,
            follow,
        )?;
        let lhs = match t.value {
            Token::Int => {
                let t = self.next();
                let v = self.text(t).to_owned();
                Expr::Int(t.span, Type::Hole, v)
            }
            Token::Float => {
                let t = self.next();
                let v = self.text(t).to_owned();
                Expr::Float(t.span, Type::Hole, v)
            }
            Token::String => {
                let t = self.next();
                let v = self.text(t).to_owned();
                Expr::String(t.span, Type::Hole, v)
            }
            Token::Name => {
                let path = self.path(follow)?;
                Expr::Unresolved(t.span, Type::Hole, path.value)
            }
            Token::LParen => {
                let t = self.expr_tuple(follow)?;
                if let Some(es) = t.value {
                    if es.len() == 1 {
                        es.into_iter().next().unwrap()
                    } else {
                        Expr::Tuple(t.span, Type::Hole, es)
                    }
                } else {
                    Expr::Tuple(t.span, Type::Hole, vec![])
                }
            }
            Token::Plus => {
                let op = self.next();
                let ((), rbp) = op.value.prefix_bp().unwrap();
                let rhs = self.expr_bp(follow, rbp)?;
                Expr::Postfix(op.span, Type::Hole, op.value, Rc::new(rhs.value))
            }
            Token::Minus => {
                let op = self.next();
                let ((), rbp) = op.value.prefix_bp().unwrap();
                let rhs = self.expr_bp(follow, rbp)?;
                Expr::Postfix(op.span, Type::Hole, op.value, Rc::new(rhs.value))
            }
            _ => unreachable!(),
        };
        Ok(Spanned::new(t.span, lhs))
    }

    fn expr(&mut self, follow: Token) -> Result<Spanned<Expr>, Span> {
        self.expr_bp(follow, 0)
    }

    fn expr_bp(&mut self, follow: Token, min_bp: u8) -> Result<Spanned<Expr>, Span> {
        let mut lhs = self
            .expr_lhs(
                follow
                    | Token::And
                    | Token::Or
                    | Token::DotDot
                    | Token::EqEq
                    | Token::NotEq
                    | Token::Lt
                    | Token::Gt
                    | Token::Le
                    | Token::Ge
                    | Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash,
            )
            .unwrap_or_else(|s| Spanned::new(s, Expr::Err(s, Type::Hole)));
        loop {
            let op = self.peek();
            if follow.contains(op.value) {
                break;
            }
            if let Some((lbp, ())) = op.value.postfix_bp() {
                if lbp < min_bp {
                    break;
                }
                self.next();
                let span = lhs.span + op.span;
                lhs = Spanned::new(
                    span,
                    Expr::Postfix(span, Type::Hole, op.value, Rc::new(lhs.value)),
                );
                continue;
            }
            if let Some((lbp, rbp)) = op.value.infix_bp() {
                if lbp < min_bp {
                    break;
                }
                self.next();
                let rhs = self.expr_bp(follow, rbp)?;
                let span = lhs.span + rhs.span;
                lhs = Spanned::new(
                    span,
                    Expr::Infix(
                        span,
                        Type::Hole,
                        op.value,
                        Rc::new(lhs.value),
                        Rc::new(rhs.value),
                    ),
                );
            } else {
                break;
            }
        }
        Ok(lhs)
    }
}
