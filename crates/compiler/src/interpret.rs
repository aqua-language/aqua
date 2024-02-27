use crate::ast::Body;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::builtins::Fun;
use crate::builtins::Record;
use crate::builtins::Tuple;
use crate::builtins::Value;
use crate::builtins::Variant;
use crate::diag::Report;

#[derive(Debug, Default)]
pub struct Context {
    stack: Stack,
    pub report: Report,
}

#[derive(Debug)]
struct Stack(Vec<Scope>);

impl Default for Stack {
    fn default() -> Self {
        Stack(vec![Scope::new()])
    }
}

impl Stack {
    fn new() -> Stack {
        Self::default()
    }

    fn bind(&mut self, name: Name, binding: Binding) {
        self.0.last_mut().unwrap().0.push((name, binding));
    }

    fn get(&self, name: &Name) -> &Binding {
        self.0
            .iter()
            .rev()
            .find_map(|scope| scope.0.iter().find(|(n, _)| n == name))
            .map(|(_, b)| b)
            .unwrap()
    }
}

#[derive(Debug)]
struct Scope(Vec<(Name, Binding)>);

impl Scope {
    fn new() -> Scope {
        Scope(Vec::new())
    }
}

#[derive(Clone, Debug)]
enum Binding {
    Var(Value),
    Def(Vec<Name>, Body),
    Struct(Vec<(Name, Type)>),
    Enum(Vec<(Name, Type)>),
    Impl(Vec<(Name, Type)>, Vec<(Name, Body)>),
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack: Stack::new(),
            report: Report::new(),
        }
    }

    fn scoped(&mut self, f: impl FnOnce(&mut Self) -> Value) -> Value {
        self.stack.0.push(Scope::new());
        let v = f(self);
        self.stack.0.pop();
        v
    }

    pub fn interpret(&mut self, p: &Program) {
        p.stmts.iter().for_each(|stmt| self.stmt(stmt));
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(s) => self.stmt_def(s),
            Stmt::Trait(_) => {}
            Stmt::Impl(s) => self.stmt_impl(s),
            Stmt::Struct(s) => self.stmt_struct(s),
            Stmt::Enum(s) => self.stmt_enum(s),
            Stmt::Type(s) => self.stmt_type(s),
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Err(_) => todo!(),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) {
        let value = self.expr(&s.expr);
        self.stack.bind(s.name.clone(), Binding::Var(value));
    }

    fn stmt_def(&mut self, s: &StmtDef) {
        let body = s.body.clone();
        let param_names = s.params.iter().map(|p| p.name.clone()).collect();
        self.stack
            .bind(s.name.clone(), Binding::Def(param_names, body));
    }

    fn stmt_impl(&mut self, _: &StmtImpl) {
        todo!()
    }

    fn stmt_struct(&mut self, _: &StmtStruct) {
        todo!()
    }

    fn stmt_enum(&mut self, _: &StmtEnum) {
        todo!()
    }

    fn stmt_type(&mut self, _: &StmtType) {
        todo!()
    }

    fn stmt_expr(&mut self, _: &Expr) {
        todo!()
    }

    fn expr(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Unresolved(_, _, _) => unreachable!(),
            Expr::Int(_, t, v) => {
                let Type::Cons(x, _) = t else {
                    unreachable!();
                };
                match x.data.as_str() {
                    "i8" => Value::I8(v.parse().unwrap()),
                    "i16" => Value::I16(v.parse().unwrap()),
                    "i32" => Value::I32(v.parse().unwrap()),
                    "i64" => Value::I64(v.parse().unwrap()),
                    "u8" => Value::U8(v.parse().unwrap()),
                    "u16" => Value::U16(v.parse().unwrap()),
                    "u32" => Value::U32(v.parse().unwrap()),
                    "u64" => Value::U64(v.parse().unwrap()),
                    _ => unreachable!(),
                }
            }
            Expr::Float(_, t, v) => {
                let Type::Cons(x, _) = t else {
                    unreachable!();
                };
                match x.data.as_str() {
                    "f32" => Value::F32(v.parse().unwrap()),
                    "f64" => Value::F64(v.parse().unwrap()),
                    _ => unreachable!(),
                }
            }
            Expr::Bool(_, _, b) => Value::Bool(*b),
            Expr::String(_, _, v) => Value::from(runtime::prelude::String::from(v.as_str())),
            Expr::Struct(_, _, _, _, xes) => {
                let xvs = xes.iter().map(|(n, e)| (n.clone(), self.expr(e))).collect();
                Value::from(Record::new(xvs))
            }
            Expr::Tuple(_, _, es) => {
                let vs = es.iter().map(|e| self.expr(e)).collect();
                Value::from(Tuple::new(vs))
            }
            Expr::Record(_, _, xes) => {
                let xvs = xes.iter().map(|(n, e)| (n.clone(), self.expr(e))).collect();
                Value::from(Record::new(xvs))
            }
            Expr::Enum(_, _, _, _, x1, e) => {
                let v = self.expr(e);
                Value::from(Variant::new(x1.clone(), v))
            }
            Expr::Field(_, _, e, x) => {
                let Value::Record(xvs) = self.expr(e) else {
                    unreachable!();
                };
                xvs.0.iter().find(|(n, _)| n == x).unwrap().1.clone()
            }
            Expr::Index(_, _, e, i) => {
                let Value::Tuple(vs) = self.expr(e) else {
                    unreachable!();
                };
                vs.0[i.data].clone()
            }
            Expr::Var(_, _, x) => {
                let Binding::Var(v) = self.stack.get(x) else {
                    unreachable!();
                };
                v.clone()
            }
            Expr::Def(_, _, x, ts) => Value::from(Fun::new(x.clone(), ts.clone())),
            Expr::Call(_, _, e, es) => {
                let Value::Fun(f) = self.expr(e) else {
                    unreachable!();
                };
                let vs = es.iter().map(|e| self.expr(e)).collect::<Vec<_>>();
                let Binding::Def(xs, body) = self.stack.get(&f.name).clone() else {
                    unreachable!();
                };
                match body {
                    Body::Expr(e) => self.scoped(|this| {
                        xs.iter()
                            .zip(vs)
                            .for_each(|(x, v)| this.stack.bind(x.clone(), Binding::Var(v)));
                        this.expr(&e)
                    }),
                    Body::Builtin => todo!(),
                }
            }
            Expr::Block(_, _, ss, e) => self.scoped(|this| {
                ss.iter().for_each(|s| this.stmt(s));
                this.expr(e)
            }),
            Expr::Query(_, _, _) => todo!(),
            Expr::Assoc(_, _, _, _, _, _) => todo!(),
            Expr::Match(_, _, e, pes) => {
                let mut iter = pes.iter();
                let (Pat::Enum(_, _, _, _, x_then, p), e_then) = iter.next().unwrap() else {
                    unreachable!();
                };
                let Pat::Var(_, _, x_var) = p.as_ref() else {
                    unreachable!();
                };
                let (Pat::Wildcard(_, _), e_else) = iter.next().unwrap() else {
                    unreachable!()
                };
                let Value::Variant(v) = self.expr(e) else {
                    unreachable!();
                };
                if *x_then == v.name {
                    let v = v.value.as_ref().clone();
                    self.stack.bind(x_var.clone(), Binding::Var(v));
                    self.expr(e_then)
                } else {
                    self.expr(e_else)
                }
            }
            Expr::Array(_, _, _) => todo!(),
            Expr::Assign(_, _, _, _) => todo!(),
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::While(_, _, e0, e1) => {
                while {
                    let Value::Bool(v) = self.expr(e0) else {
                        unreachable!()
                    };
                    v
                } {
                    self.expr(e1);
                }
                Value::from(Tuple::new(vec![]))
            }
            Expr::Fun(_, _, _, _, _) => unreachable!(),
            Expr::Err(_, _) => unreachable!(),
            Expr::Value(_, _) => unreachable!(),
            Expr::Infix(_, _, _, _, _) => unreachable!(),
            Expr::Postfix(_, _, _, _) => unreachable!(),
            Expr::Prefix(_, _, _, _) => unreachable!(),
        }
    }
}
