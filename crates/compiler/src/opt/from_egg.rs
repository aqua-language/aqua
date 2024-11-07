#![allow(unused)]

use std::rc::Rc;

use egglog::ast::Literal;
use egglog::TermDag;

use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Type;
use crate::span::Span;
use crate::symbol::Symbol;

pub type EggExpr = egglog::Term;

struct Context {
    termdag: TermDag,
}

impl Context {
    fn into_expr(&mut self, id: usize) -> Expr {
        match self.termdag.get(id) {
            EggExpr::App(x, es) => {
                let t = self.into_type(es[0]);
                match x.as_str() {
                    "Expr::Int" => {
                        let x = self.into_int(es[1]);
                        Expr::Int(Span::Generated, t, x.to_string().into())
                    }
                    "Expr::Float" => {
                        let x = self.into_float(es[1]);
                        Expr::Float(Span::Generated, t, Symbol::from(x.to_string()))
                    }
                    "Expr::Bool" => {
                        let x = self.into_bool(es[1]);
                        Expr::Bool(Span::Generated, t, x)
                    }
                    "Expr::String" => {
                        let s = self.into_symbol(es[1]);
                        Expr::String(Span::Generated, t, s)
                    }
                    "Expr::IfElse" => {
                        let e0 = self.into_expr(es[1]);
                        let e1 = self.into_expr(es[2]);
                        let e2 = self.into_expr(es[3]);
                        Expr::IfElse(Span::Generated, t, Rc::new(e0), Rc::new(e1), Rc::new(e2))
                    }
                    "Expr::Char" => {
                        let s = self.into_char(es[1]);
                        Expr::Char(Span::Generated, t, s)
                    }
                    "Expr::Tuple" => {
                        let es = self.into_expr_list(es[1]);
                        Expr::Tuple(Span::Generated, t, es)
                    }
                    "Expr::Record" => {
                        let es = self.into_expr_map(es[1]);
                        Expr::Record(Span::Generated, t, es)
                    }
                    "Expr::Index" => {
                        let e = self.into_expr(es[1]);
                        let i = self.into_int(es[2]) as usize;
                        Expr::Index(Span::Generated, t, Rc::new(e), Index::from(i))
                    }
                    "Expr::Field" => {
                        let e = self.into_expr(es[1]);
                        let x = self.into_name(es[2]);
                        Expr::Field(Span::Generated, t, Rc::new(e), x)
                    }
                    "Expr::Update" => todo!(),
                    "Expr::Var" => {
                        let x = self.into_name(es[1]);
                        Expr::Var(Span::Generated, t, x)
                    }
                    "Expr::Def" => {
                        let x = self.into_symbol(es[1]);
                        Expr::Def(Span::Generated, t, Name::from(x), vec![])
                    }
                    "Expr::Call" => {
                        let e0 = self.into_expr(es[1]);
                        let es = self.into_expr_list(es[2]);
                        Expr::Call(Span::Generated, t, Rc::new(e0), es)
                    }
                    "Expr::IfElse" => {
                        let e0 = self.into_expr(es[1]);
                        let e1 = self.into_expr(es[2]);
                        let e2 = self.into_expr(es[3]);
                        Expr::IfElse(Span::Generated, t, Rc::new(e0), Rc::new(e1), Rc::new(e2))
                    }
                    "Expr::Array" => todo!(),
                    "Expr::Lambda" => todo!(),
                    "Expr::LetIn" => {
                        let x = self.into_name(es[1]);
                        let e0 = self.into_expr(es[2]);
                        let e1 = self.into_expr(es[3]);
                        Expr::LetIn(
                            Span::Generated,
                            t,
                            x,
                            e0.type_of().clone(),
                            Rc::new(e0),
                            Rc::new(e1),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn into_symbol(&mut self, id: usize) -> Symbol {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::String(x)) => Symbol::from(x.as_str()),
            _ => unreachable!(),
        }
    }

    fn into_type_list(&mut self, mut id: usize) -> Vec<Type> {
        let mut ts = vec![];
        loop {
            let EggExpr::App(x, es) = self.termdag.get(id) else {
                unreachable!()
            };
            match x.as_str() {
                "ListCons" => {
                    let t = self.into_type(es[0]);
                    id = es[1];
                    ts.push(t);
                }
                "ListNil" => break,
                _ => unreachable!(),
            }
        }
        ts
    }

    fn into_type_map(&mut self, mut id: usize) -> Map<Name, Type> {
        let mut ts = Map::new();
        loop {
            let EggExpr::App(x, es) = self.termdag.get(id) else {
                unreachable!()
            };
            match x.as_str() {
                "MapCons" => {
                    let k = self.into_name(es[0]);
                    let v = self.into_type(es[1]);
                    id = es[2];
                    ts.insert(k, v);
                }
                "MapNil" => break,
                _ => unreachable!(),
            }
        }
        ts
    }

    fn into_type(&mut self, id: usize) -> Type {
        match self.termdag.get(id) {
            EggExpr::App(x, es) => match x.as_str() {
                "Type::Cons" => {
                    let x = self.into_symbol(es[1]);
                    let ts = self.into_type_list(es[2]);
                    Type::Cons(Name::from(x), vec![])
                }
                "Type::Tuple" => {
                    let ts = self.into_type_list(es[1]);
                    Type::Tuple(ts)
                }
                "Type::Record" => {
                    let ts = self.into_type_map(es[1]);
                    Type::Record(ts)
                }
                "Type::Array" => todo!(),
                "Type::Never" => todo!(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn into_char(&mut self, id: usize) -> char {
        let x = self.into_string(id);
        x.as_str().chars().next().unwrap()
    }

    fn into_bool(&mut self, id: usize) -> bool {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::Bool(x)) => x,
            _ => unreachable!(),
        }
    }

    fn into_int(&mut self, id: usize) -> i64 {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::Int(x)) => x,
            _ => unreachable!(),
        }
    }

    fn into_float(&mut self, id: usize) -> f64 {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::F64(x)) => x.into_inner(),
            _ => unreachable!(),
        }
    }

    fn into_string(&mut self, id: usize) -> String {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::String(x)) => x.to_string(),
            _ => unreachable!(),
        }
    }

    fn into_name(&mut self, id: usize) -> Name {
        match self.termdag.get(id) {
            EggExpr::Lit(Literal::String(x)) => Name::from(Symbol::from(x.as_str())),
            _ => unreachable!(),
        }
    }

    fn into_expr_list(&mut self, mut id: usize) -> Vec<Expr> {
        let mut es0 = vec![];
        loop {
            let EggExpr::App(x, es1) = self.termdag.get(id) else {
                unreachable!()
            };
            match x.as_str() {
                "ListCons" => {
                    let e0 = &es1[0];
                    id = es1[1];
                    es0.push(self.into_expr(*e0));
                }
                "ListNil" => {
                    break;
                }
                _ => unreachable!(),
            }
        }
        es0
    }

    fn into_expr_map(&mut self, mut id: usize) -> Map<Name, Expr> {
        let mut es0 = Map::new();
        loop {
            let EggExpr::App(x, es1) = self.termdag.get(id) else {
                unreachable!()
            };
            match x.as_str() {
                "MapCons" => {
                    let k = self.into_name(es1[0]);
                    let v = self.into_expr(es1[1]);
                    id = es1[2];
                    es0.insert(k, v);
                }
                "MapNil" => {
                    break;
                }
                _ => unreachable!(),
            }
        }
        es0
    }
}
