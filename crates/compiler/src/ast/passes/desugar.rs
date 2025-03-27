use std::rc::Rc;

use smol_str::format_smolstr;

use crate::ast::parse::splice::Splice;
use crate::ast::parse::splice::SpliceIterator;
use crate::ast::parse::token::Token;
use crate::ast::passes::Pass;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Type;
use crate::report::Report;
use crate::report::span::Span;
use crate::traversal::mapper::Mapper;

use self::util::infix;
use self::util::unop;

#[derive(Debug)]
pub struct Context {
    anons: Stack,
    pub report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

#[derive(Debug, Default)]
pub struct Stack {
    pub scopes: Vec<Vec<Name>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            anons: Stack::default(),
            report: Report::new(),
        }
    }

    pub fn arg(&mut self, e: &Expr) -> Expr {
        if let Expr::Anonymous(..) = e {
            // Partially applied function
            self.map_expr(e)
        } else {
            self.anons.scopes.push(vec![]);
            let e = self.map_expr(e);
            let xs = self.anons.scopes.pop().unwrap();
            if xs.is_empty() {
                e
            } else {
                let xts = xs
                    .into_iter()
                    .map(|x| Local::new(x.span, x, Type::Unknown, false))
                    .collect();
                Expr::Lambda(e.span(), Type::Unknown, xts, Type::Unknown, Rc::new(e))
            }
        }
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        let t = self.map_type(e.ty());
        match e {
            Expr::InfixBinaryOp(s, _, op, e0, e1) => {
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                match *op {
                    Token::Plus => infix(*s, t, "Add", "add", e0, e1),
                    Token::Minus => infix(*s, t, "Sub", "sub", e0, e1),
                    Token::Star => infix(*s, t, "Mul", "mul", e0, e1),
                    Token::Slash => infix(*s, t, "Div", "div", e0, e1),
                    Token::Gt => infix(*s, t, "PartialOrd", "gt", e0, e1),
                    Token::Ge => infix(*s, t, "PartialOrd", "ge", e0, e1),
                    Token::Lt => infix(*s, t, "PartialOrd", "lt", e0, e1),
                    Token::Le => infix(*s, t, "PartialOrd", "le", e0, e1),
                    Token::EqEq => infix(*s, t, "PartialEq", "eq", e0, e1),
                    Token::NotEq => infix(*s, t, "PartialEq", "ne", e0, e1),
                    Token::DotDot => infix(*s, t, "Range", "range", e0, e1),
                    // a and b => match a { true => b, _ => false }
                    Token::And => Expr::Match(
                        *s,
                        Type::Unknown,
                        Rc::new(e0),
                        vec![
                            (Pat::Bool(*s, Type::Unknown, true), e1),
                            (
                                Pat::Wildcard(*s, Type::Unknown),
                                Expr::Bool(*s, Type::Unknown, false),
                            ),
                        ]
                        .into(),
                    ),
                    // a or b => match a { true => true, _ => b }
                    Token::Or => Expr::Match(
                        *s,
                        Type::Unknown,
                        Rc::new(e0),
                        vec![
                            (
                                Pat::Bool(*s, Type::Unknown, true),
                                Expr::Bool(*s, Type::Unknown, true),
                            ),
                            (Pat::Wildcard(*s, Type::Unknown), e1),
                        ]
                        .into(),
                    ),
                    _ => unreachable!(),
                }
            }
            // a.b(c) => _::b(a, c)
            Expr::Dot(s, _, e, x, ts, es) => {
                let e = self.map_expr(e);
                let es = self.map_iter(es, Self::arg);
                let es = std::iter::once(e).chain(es).collect::<Vec<_>>();
                let efun = Expr::Assoc(*s, Type::Unknown, Impl::Unknown, *x, ts.clone());
                Expr::Call(*s, t, Rc::new(efun), es)
            }
            // e(...,_+_,...) => foo(x => x.f)
            Expr::Call(s, _, e, es) => {
                let e = self.map_expr(e);
                let es = self.map_iter(es, Self::arg);
                Expr::Call(*s, t, Rc::new(e), es)
            }
            Expr::Anonymous(s, t) => {
                if let Some(xs) = self.anons.scopes.last_mut() {
                    let n = xs.len();
                    let x = Name::new(*s, format_smolstr!("_{}", n));
                    xs.push(x);
                    let path = Path::new_name(x);
                    Expr::Path(*s, t.clone(), path)
                } else {
                    Expr::Err(*s, t.clone())
                }
            }
            // -a => Neg(a)
            Expr::PrefixUnaryOp(s, _, op, e) => {
                let e = self.map_expr(e);
                match *op {
                    Token::Minus => unop(*s, t, "Neg", "neg", e),
                    Token::Not => unop(*s, t, "Not", "not", e),
                    _ => unreachable!(),
                }
            }
            // 1s => _::postfix_s(1)
            Expr::IntSuffix(s, _, l, r) => {
                let e0 = Expr::Int(*s, Type::Unknown, *l);
                let x = Name::new(*s, format_smolstr!("postfix_{r}"));
                let e1 = Expr::Assoc(*s, Type::Unknown, Impl::Unknown, x, vec![]);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            // 1.0s => _::postfix_s(1.0)
            Expr::FloatSuffix(s, _, l, r) => {
                let e0 = Expr::Float(*s, Type::Unknown, *l);
                let x = Name::new(*s, format_smolstr!("postfix_{r}"));
                let e1 = Expr::Assoc(*s, Type::Unknown, Impl::Unknown, x, vec![]);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            // (a) => a
            Expr::Paren(_, _, e) => self.map_expr(e),
            // "a ${b} c" => "a ".concat(b.toString()).concat(" c")
            Expr::String(s, _, l) => {
                let mut iter = SpliceIterator::new(l.as_str());
                let s = s.trim(1);
                if let Some(splice) = iter.next() {
                    let e = self.map_splice(splice, s);
                    iter.fold(e, |e0, splice| {
                        let e1 = self.map_splice(splice, s);
                        infix(s, Type::Unknown, "String", "concat", e0, e1)
                    })
                } else {
                    e.clone()
                }
            }
            Expr::Annotate(_, _, e) => {
                let e = self.map_expr(e);
                e.with_type(t)
            }
            _ => self._map_expr(e),
        }
    }

    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Paren(t) => self.map_type(t),
            _ => self._map_type(t),
        }
    }

    fn map_pattern(&mut self, p: &Pat) -> Pat {
        match p {
            Pat::Paren(_, _, p) => self.map_pattern(p),
            Pat::Annotate(_, t, p) => {
                let t = self.map_type(t);
                let p = self.map_pattern(p);
                p.with_type(t)
            }
            _ => self._map_pattern(p),
        }
    }
}

impl Context {
    fn map_splice(&mut self, splice: Splice, span: Span) -> Expr {
        let range = splice.range();
        let file = span.file().unwrap();
        let start = span.start().unwrap() + range.start as u32;
        let end = span.start().unwrap() + range.end as u32;
        let span = Span::new(file, start..end);

        match splice {
            Splice::Text(s, _) => Expr::String(span, Type::Unknown, s.into()),
            Splice::Delim(s, _) => {
                // Only lex the part of the file that is the splice.
                // let source = &self.sources.fetch(&file).unwrap().text()[..end as usize];
                // let lexer = crate::lexer::Lexer::new_from(file, source, start as usize);
                let lexer = crate::ast::parse::lexer::Lexer::new_from(file, s, start as usize);
                let mut parser = crate::ast::parse::parser::Parser::new(s, lexer);
                if let Ok(e) = parser.parse(|p, follow| p.expr(follow)) {
                    let e = self.map_expr(&e.v);
                    unop(span, Type::Unknown, "Display", "toString", e)
                } else {
                    // TODO: Report error
                    Expr::Err(span, Type::Unknown)
                }
            }
            Splice::Err(_, _) => {
                // TODO: Report error
                Expr::Err(span, Type::Unknown)
            }
        }
    }
}

mod util {
    use std::rc::Rc;

    use crate::ast::Expr;
    use crate::ast::Name;
    use crate::ast::Path;
    use crate::ast::Segment;
    use crate::ast::Type;
    use crate::report::span::Span;

    pub(super) fn unop(s: Span, t: Type, x0: &'static str, x1: &'static str, e: Expr) -> Expr {
        let s0 = Segment::new_name(Name::new(s, x0));
        let s1 = Segment::new_name(Name::new(s, x1));
        let path = Path::new(vec![s0, s1]);
        let fun = Expr::Path(s, Type::Unknown, path);
        Expr::Call(s, t, Rc::new(fun), vec![e])
    }

    pub(super) fn infix(
        s: Span,
        t: Type,
        x0: &'static str,
        x1: &'static str,
        e0: Expr,
        e1: Expr,
    ) -> Expr {
        let s0 = Segment::new_name(Name::new(s, x0));
        let s1 = Segment::new_name(Name::new(s, x1));
        let path = Path::new(vec![s0, s1]);
        let fun = Expr::Path(s, Type::Unknown, path);
        Expr::Call(s, t, Rc::new(fun), vec![e0, e1])
    }
}
