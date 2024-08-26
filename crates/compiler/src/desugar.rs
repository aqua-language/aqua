use std::rc::Rc;

use smol_str::format_smolstr;

use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Segment;
use crate::ast::Type;
use crate::lexer::Token;
use crate::traversal::mapper::Mapper;

use self::util::infix;
use self::util::unop;

#[derive(Debug)]
pub struct Context;

impl Context {
    pub fn new() -> Self {
        Self
    }

    pub fn desugar(&mut self, program: &Program) -> Program {
        self.map_program(program)
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
            // a.b(c) => b(a, c)
            Expr::Dot(s, _, e, x, ts, es) => {
                let e = self.map_expr(e);
                let es = self.map_exprs(es);
                let es = std::iter::once(e).chain(es).collect::<Vec<_>>();
                let path = Path::new(vec![Segment::new(*s, *x, ts.clone(), vec![].into())]);
                let e = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, t.clone(), Rc::new(e), es)
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
            // 1s => postfix_s(1)
            Expr::IntSuffix(s, _, l, r) => {
                let e0 = Expr::Int(*s, Type::Unknown, *l);
                let path = Path::new_name(Name::new(*s, format_smolstr!("postfix_{r}")));
                let e1 = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            // 1.0s => postfix_s(1.0)
            Expr::FloatSuffix(s, _, l, r) => {
                let e0 = Expr::Float(*s, Type::Unknown, *l);
                let path = Path::new_name(Name::new(*s, format_smolstr!("postfix_{r}")));
                let e1 = Expr::Path(*s, Type::Unknown, path);
                Expr::Call(*s, Type::Unknown, Rc::new(e1), vec![e0])
            }
            // (a) => a
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

mod util {
    use std::rc::Rc;

    use crate::ast::Expr;
    use crate::ast::Name;
    use crate::ast::Path;
    use crate::ast::Segment;
    use crate::ast::Type;
    use crate::span::Span;

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
