use std::rc::Rc;

use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Segment;
use crate::ast::Type;
use crate::lexer::Span;
use crate::lexer::Token;
use crate::traversal::mapper::Mapper;

#[derive(Debug)]
pub struct Context;

impl Context {
    pub fn new() -> Self {
        Self
    }

    pub fn desugar(&mut self, program: &Program) -> Program {
        self.map_program(program)
    }

    fn infix(
        &mut self,
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

    fn unop(&mut self, s: Span, t: Type, x0: &'static str, x1: &'static str, e: Expr) -> Expr {
        let s0 = Segment::new_name(Name::new(s, x0));
        let s1 = Segment::new_name(Name::new(s, x1));
        let path = Path::new(vec![s0, s1]);
        let fun = Expr::Path(s, Type::Unknown, path);
        Expr::Call(s, t, Rc::new(fun), vec![e])
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        let t = self.map_type(e.type_of());
        match e {
            Expr::InfixBinaryOp(s, _, op, e0, e1) => {
                let e0 = self.map_expr(e0);
                let e1 = self.map_expr(e1);
                match *op {
                    Token::Plus => self.infix(*s, t, "Add", "add", e0, e1),
                    Token::Minus => self.infix(*s, t, "Sub", "sub", e0, e1),
                    Token::Star => self.infix(*s, t, "Mul", "mul", e0, e1),
                    Token::Slash => self.infix(*s, t, "Div", "div", e0, e1),
                    Token::Gt => self.infix(*s, t, "PartialOrd", "gt", e0, e1),
                    Token::Ge => self.infix(*s, t, "PartialOrd", "ge", e0, e1),
                    Token::Lt => self.infix(*s, t, "PartialOrd", "lt", e0, e1),
                    Token::Le => self.infix(*s, t, "PartialOrd", "le", e0, e1),
                    Token::EqEq => self.infix(*s, t, "PartialEq", "eq", e0, e1),
                    Token::NotEq => self.infix(*s, t, "PartialEq", "ne", e0, e1),
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
            Expr::Dot(s, _, e, x, ts, es) => {
                let es = std::iter::once(e.as_ref().clone())
                    .chain(es.clone())
                    .collect::<Vec<_>>();
                let path = Path::new(vec![Segment::new(*s, *x, ts.clone(), vec![].into())]);
                let e = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, t.clone(), Rc::new(e), es)
            }
            Expr::PrefixUnaryOp(s, _, op, e) => {
                let e = self.map_expr(e);
                match *op {
                    Token::Minus => self.unop(*s, t, "Neg", "neg", e),
                    Token::Not => self.unop(*s, t, "Not", "not", e),
                    _ => unreachable!(),
                }
            }
            Expr::IntSuffix(s, _, x, v) => {
                let e0 = Expr::Int(*s, Type::Unknown, *v);
                let path = Path::new_name(Name::new(*s, *x));
                let e1 = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            Expr::FloatSuffix(s, _, x, v) => {
                let e0 = Expr::Float(*s, Type::Unknown, *v);
                let path = Path::new_name(Name::new(*s, *x));
                let e1 = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            Expr::Paren(_, _, e) => self.map_expr(e),
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
