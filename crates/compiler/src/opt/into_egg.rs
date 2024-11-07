#![allow(unused)]

use std::borrow::Borrow;
use std::num::ParseIntError;

use egglog::ast::Literal;
use egglog::EGraph;
use ordered_float::OrderedFloat;
use symbol_table::GlobalSymbol;
use symbol_table::GlobalSymbol as EggSymbol;

use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::symbol::Symbol;

pub type EggExpr = egglog::ast::GenericExpr<Symbol, Symbol, ()>;

pub struct Context;

impl Expr {
    fn into_egg(&self, ctx: &mut Context) -> EggExpr {
        ctx.from_expr(self)
    }
}

impl Context {
    pub fn new() -> Self {
        Context
    }

    pub fn from_program(&mut self, p: &Program) -> EggExpr {
        p.stmts
            .iter()
            .rev()
            .fold(call("Program", vec![]), |acc, stmt| {
                let e0 = self.from_stmt(stmt);
                call("StmtListCons", vec![e0, acc])
            })
    }

    pub fn from_stmt(&mut self, stmt: &Stmt) -> EggExpr {
        match stmt {
            Stmt::Def(s) => {
                if let ExprBody::UserDefined(b) = &s.body {
                    let xts = self.from_type_map(&s.params);
                    let t = self.from_type(&s.ty);
                    let x = symbol(s.name.data);
                    let e = self.from_expr(b);
                    call("StmtDef", vec![lit(Literal::String(x))])
                } else {
                    todo!()
                }
            }
            Stmt::Impl(_) => todo!(),
            Stmt::Struct(_) => todo!(),
            Stmt::Enum(_) => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn from_type(&mut self, t: &Type) -> EggExpr {
        match t {
            Type::Cons(x, _) => {
                let x1 = symbol(x.data);
                call("TypeCons", vec![lit(Literal::String(x1))])
            }
            Type::Lambda(ts, t) => {
                let t0 = self.from_type_list(ts);
                let t1 = self.from_type(t);
                call("TypeLambda", vec![t0, t1])
            }
            Type::Tuple(ts) => {
                let t = self.from_type_list(ts);
                call("TypeTuple", vec![t])
            }
            Type::Record(xts) => {
                let t = self.from_type_map(xts);
                call("TypeRecord", vec![t])
            }
            Type::Array(_, _) => todo!(),
            Type::Never => todo!(),
            Type::Paren(_) => todo!(),
            Type::Err => todo!(),
            Type::Unknown => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn from_type_list(&mut self, ts: &[Type]) -> EggExpr {
        ts.iter()
            .rev()
            .fold(call("TypeListNil", vec![]), |acc, t0| {
                let t1 = self.from_type(t0);
                call("TypeListCons", vec![t1, acc])
            })
    }

    pub fn from_type_map(&mut self, xts: &[(Name, Type)]) -> EggExpr {
        xts.iter()
            .rev()
            .fold(call("TypeMapNil", vec![]), |acc, (x0, t0)| {
                let x1 = lit(Literal::String(symbol(x0.data)));
                let t1 = self.from_type(t0);
                call("TypeMapCons", vec![x1, t1, acc])
            })
    }

    pub fn from_expr(&mut self, e: &Expr) -> EggExpr {
        let t = self.from_type(&e.type_of());
        match e {
            Expr::Int(_, _, s) => {
                let i = s.as_str().parse::<i64>().unwrap();
                let e0 = lit(Literal::Int(i));
                call("ExprInt", vec![t, e0])
            }
            Expr::Float(_, _, s) => {
                let f = s.as_str().parse::<f64>().unwrap();
                let e0 = lit(Literal::F64(OrderedFloat(f)));
                call("ExprFloat", vec![t, e0])
            }
            Expr::Bool(_, _, v) => {
                let e0 = lit(Literal::Bool(*v));
                call("ExprBool", vec![t, e0])
            }
            Expr::String(_, _, v) => {
                let e0 = lit(Literal::String(GlobalSymbol::new(v.as_str())));
                call("ExprString", vec![t, e0])
            }
            Expr::Char(_, _, v) => {
                let e0 = lit(Literal::String(GlobalSymbol::new(v.to_string())));
                call("ExprChar", vec![t, e0])
            }
            Expr::Tuple(_, _, es) => {
                let e = es
                    .iter()
                    .rev()
                    .fold(call("ExprListNil", vec![]), |acc, e0| {
                        let e1 = self.from_expr(e0);
                        call("ExprListCons", vec![e1, acc])
                    });
                call("ExprTuple", vec![e])
            }
            Expr::Record(_, _, xes) => {
                let e = xes
                    .iter()
                    .rev()
                    .fold(call("MapNil", vec![]), |acc, (x0, e0)| {
                        let e1 = lit(Literal::String(symbol(x0.data)));
                        let e2 = self.from_expr(e0);
                        call("MapCons", vec![e1, e2, acc])
                    });
                call("ExprRecord", vec![e])
            }
            Expr::Enum(_, _, _, _, _, _) => todo!(),
            Expr::Field(_, _, e, x) => {
                let e1 = self.from_expr(e);
                let e2 = lit(Literal::String(symbol(x.data)));
                call("ExprField", vec![e1, e2])
            }
            Expr::Update(_, _, _, _, _) => todo!(),
            Expr::Index(_, _, e, i) => {
                let e1 = self.from_expr(e);
                let e2 = lit(Literal::Int(i.data as i64));
                call("ExprIndex", vec![e1, e2])
            }
            Expr::Var(_, _, x) => {
                let x1 = symbol(x.data);
                call("ExprVar", vec![lit(Literal::String(x1))])
            }
            Expr::Def(_, _, x, _) => {
                let x1 = symbol(x.data);
                call("ExprDef", vec![lit(Literal::String(x1))])
            }
            Expr::Call(_, _, e, es) => {
                let e0 = self.from_expr(e);
                let e1 = es
                    .iter()
                    .rev()
                    .fold(call("ExprListNil", vec![]), |acc, e0| {
                        let e1 = self.from_expr(e0);
                        call("ExprListCons", vec![e1, acc])
                    });
                call("ExprCall", vec![e0, e1])
            }
            Expr::Closure(_, _, xts0, xts1, _, e) => todo!(),
            Expr::IfElse(_, _, e0, e1, e2) => {
                let e0 = self.from_expr(e0);
                let e1 = self.from_expr(e1);
                let e2 = self.from_expr(e2);
                call("ExprIfElse", vec![e0, e1, e2])
            }
            Expr::Array(_, _, _) => todo!(),
            Expr::Lambda(_, _, _, _, _) => todo!(),
            Expr::LetIn(_, _, x, _, e0, e1) => {
                let e0 = self.from_expr(e0);
                let e1 = self.from_expr(e1);
                let x1 = symbol(x.data);
                call("ExprLetIn", vec![lit(Literal::String(x1)), e0, e1])
            }
            _ => unreachable!(),
        }
    }
}

fn lit(l: Literal) -> EggExpr {
    EggExpr::Lit((), l)
}

fn call(s: impl Into<Symbol>, es: Vec<EggExpr>) -> EggExpr {
    EggExpr::Call((), s.into(), es)
}

fn symbol(s: Symbol) -> EggSymbol {
    GlobalSymbol::new(s.as_str())
}
