use std::collections::HashMap;

use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtImpl;
use crate::ast::StmtVar;
use crate::ast::TraitBound;
use crate::ast::Type;
use crate::builtins::Fun;
use crate::builtins::Record;
use crate::builtins::Tuple;
use crate::builtins::Value;
use crate::builtins::Variant;
use crate::diag::Report;

#[derive(Debug, Default)]
pub struct Context {
    pub stack: Stack,
    pub report: Report,
}

#[derive(Debug)]
pub struct Stack {
    defs: HashMap<Name, StmtDef>,
    impls: HashMap<Wrapper<TraitBound>, StmtImpl>,
    locals: Vec<Scope>,
}

impl Default for Stack {
    fn default() -> Self {
        Stack {
            defs: HashMap::default(),
            impls: HashMap::default(),
            locals: vec![Scope::new()],
        }
    }
}

impl Stack {
    fn new() -> Stack {
        Self::default()
    }

    fn bind_def(&mut self, name: Name, def: StmtDef) {
        self.defs.insert(name, def);
    }

    fn bind_val(&mut self, name: Name, val: Value) {
        self.locals.last_mut().unwrap().0.push((name, val));
    }

    fn bind_impl(&mut self, head: TraitBound, imp: StmtImpl) {
        self.impls.insert(Wrapper(head), imp);
    }

    fn get_def(&self, name: &Name) -> &StmtDef {
        self.defs.get(name).unwrap()
    }

    fn get_val(&self, name: &Name) -> &Value {
        self.locals
            .iter()
            .rev()
            .find_map(|scope| scope.0.iter().find(|(n, _)| n == name))
            .map(|(_, b)| b)
            .unwrap()
    }
}

#[derive(Debug)]
struct Wrapper<T>(T);

impl std::hash::Hash for Wrapper<TraitBound> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Wrapper(&self.0).hash(state);
    }
}

impl<'a> std::hash::Hash for Wrapper<&'a TraitBound> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
        self.0.ts.iter().for_each(|t| Wrapper(t).hash(state));
    }
}

impl<'a> std::hash::Hash for Wrapper<&'a Type> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self.0).hash(state);
        match &self.0 {
            Type::Unresolved(_) => unreachable!(),
            Type::Cons(x, ts) => {
                x.hash(state);
                ts.iter().for_each(|t| Wrapper(t).hash(state));
            }
            Type::Alias(_, _) => unreachable!(),
            Type::Assoc(tb, x, _) => {
                let t = &tb.xts.iter().find(|(n, _)| n == x).unwrap().1;
                Wrapper(t).hash(state);
            }
            Type::Var(_) => unreachable!(),
            Type::Generic(x) => x.hash(state),
            Type::Fun(ts, t) => {
                ts.iter().for_each(|t| Wrapper(t).hash(state));
                Wrapper(t.as_ref()).hash(state);
            }
            Type::Tuple(ts) => ts.iter().for_each(|t| Wrapper(t).hash(state)),
            Type::Record(xts) => xts.iter().for_each(|(x, t)| {
                x.hash(state);
                Wrapper(t).hash(state);
            }),
            Type::Array(t, n) => {
                Wrapper(t.as_ref()).hash(state);
                n.hash(state);
            }
            Type::Never => {}
            Type::Hole => unreachable!(),
            Type::Err => unreachable!(),
        }
    }
}

impl PartialEq for Wrapper<TraitBound> {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name && self.0.ts == other.0.ts
    }
}

impl<'a> PartialEq for Wrapper<&'a TraitBound> {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name && self.0.ts == other.0.ts
    }
}

impl<'a> PartialEq for Wrapper<&'a Type> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Type::Unresolved(_), _) => unreachable!(),
            (_, Type::Unresolved(_)) => unreachable!(),
            (Type::Cons(x1, ts1), Type::Cons(x2, ts2)) => x1 == x2 && ts1 == ts2,
            (Type::Alias(_, _), _) | (_, Type::Alias(_, _)) => unreachable!(),
            (Type::Assoc(tb1, x1, _), Type::Assoc(tb2, x2, _)) => {
                Wrapper(tb1) == Wrapper(tb2) && x1 == x2
            }
            (_, Type::Assoc(_, _, _)) => unreachable!(),
            (Type::Var(_), _) => unreachable!(),
            (_, Type::Var(_)) => unreachable!(),
            (Type::Generic(x1), Type::Generic(x2)) => x1 == x2,
            (Type::Fun(ts1, t1), Type::Fun(ts2, t2)) => ts1 == ts2 && t1 == t2,
            (Type::Tuple(ts1), Type::Tuple(ts2)) => ts1 == ts2,
            (Type::Record(xts1), Type::Record(xts2)) => xts1 == xts2,
            (Type::Array(t1, n1), Type::Array(t2, n2)) => t1 == t2 && n1 == n2,
            (Type::Never, Type::Never) => true,
            (Type::Hole, _) => unreachable!(),
            (_, Type::Hole) => unreachable!(),
            (Type::Err, Type::Err) => true,
            _ => false,
        }
    }
}

impl Eq for Wrapper<TraitBound> {}

#[derive(Debug)]
struct Scope(Vec<(Name, Value)>);

impl Scope {
    fn new() -> Scope {
        Scope(Vec::new())
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack: Stack::new(),
            report: Report::new(),
        }
    }

    fn scoped(&mut self, f: impl FnOnce(&mut Self) -> Value) -> Value {
        self.stack.locals.push(Scope::new());
        let v = f(self);
        self.stack.locals.pop();
        v
    }

    pub fn interpret(&mut self, p: &Program) {
        p.stmts.iter().for_each(|stmt| self.decl_stmt(stmt));
        p.stmts.iter().for_each(|stmt| self.stmt(stmt));
    }

    fn decl_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => self.decl_stmt_def(s),
            Stmt::Trait(_) => {}
            Stmt::Impl(s) => self.decl_stmt_impl(s),
            Stmt::Struct(_) => {}
            Stmt::Enum(_) => {}
            Stmt::Type(_) => {}
            Stmt::Expr(_) => {}
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(_) => {}
            Stmt::Trait(_) => {}
            Stmt::Impl(_) => {}
            Stmt::Struct(_) => {}
            Stmt::Enum(_) => {}
            Stmt::Type(_) => {}
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) {
        let value = self.expr(&s.expr);
        self.stack.bind_val(s.name.clone(), value);
    }

    fn decl_stmt_def(&mut self, s: &StmtDef) {
        self.stack.bind_def(s.name.clone(), s.clone());
    }

    fn decl_stmt_impl(&mut self, s: &StmtImpl) {
        let Bound::Trait(_, tb) = &s.head else {
            unreachable!();
        };
        self.stack.bind_impl(tb.clone(), s.clone());
    }

    fn stmt_expr(&mut self, _: &Expr) {
        todo!()
    }

    fn block(&mut self, b: &Block) -> Value {
        self.scoped(|this| {
            b.stmts.iter().for_each(|s| this.stmt(s));
            this.expr(&b.expr)
        })
    }

    pub fn expr(&mut self, e: &Expr) -> Value {
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
            Expr::Char(_, _, v) => Value::Char(*v),
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
                let rec = self.expr(e).as_record();
                rec[x].clone()
            }
            Expr::Index(_, _, e, i) => {
                let tup = self.expr(e).as_tuple();
                tup[i].clone()
            }
            Expr::Var(_, _, x) => self.stack.get_val(x).clone(),
            Expr::Def(_, _, x, ts) => Value::from(Fun::new_def(x.clone(), ts.clone())),
            Expr::Assoc(_, _, tb, x, ts) => {
                Value::from(Fun::new_assoc(tb.clone(), x.clone(), ts.clone()))
            }
            Expr::Call(_, _, e, es) => {
                let f = self.expr(e).as_function();
                let vs = es.iter().map(|e| self.expr(e)).collect::<Vec<_>>();
                match f {
                    Fun::Def(x, ts) => {
                        let s = self.stack.get_def(&x).clone();
                        match s.body {
                            StmtDefBody::UserDefined(e) => self.scoped(|ctx| {
                                for (p, v) in s.params.iter().zip(vs) {
                                    ctx.stack.bind_val(p.name.clone(), v)
                                }
                                ctx.expr(&e)
                            }),
                            StmtDefBody::Builtin(builtin) => (builtin.fun)(self, &ts, &vs),
                        }
                    }
                    Fun::Assoc(tb, x, _) => {
                        println!("Looking for: {}", tb);
                        let s = self.stack.impls.get(&Wrapper(tb)).unwrap().clone();
                        println!("Found: {}", s);
                        let s = s.defs.iter().find(|d| d.name == x).unwrap();
                        self.scoped(|ctx| {
                            for (p, v) in s.params.iter().zip(vs) {
                                ctx.stack.bind_val(p.name.clone(), v)
                            }
                            ctx.expr(&e)
                        })
                    }
                }
            }
            Expr::Block(_, _, b) => self.block(b),
            Expr::Query(_, _, _) => todo!(),
            Expr::Match(_, _, _e, _pes) => {
                todo!()
                // let mut iter = pes.iter();
                // let (Pat::Enum(_, _, _, _, x_then, p), e_then) = iter.next().unwrap() else {
                //     unreachable!();
                // };
                // let Pat::Var(_, _, x_var) = p.as_ref() else {
                //     unreachable!();
                // };
                // let (Pat::Wildcard(_, _), e_else) = iter.next().unwrap() else {
                //     unreachable!()
                // };
                // let Value::Variant(v) = self.expr(e) else {
                //     unreachable!();
                // };
                // if *x_then == v.name {
                //     let v = v.value.as_ref().clone();
                //     self.stack.bind(x_var.clone(), Binding::Var(v));
                //     self.expr(e_then)
                // } else {
                //     self.expr(e_else)
                // }
            }
            Expr::Array(_, _, _) => todo!(),
            Expr::Assign(_, _, _, _) => todo!(),
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::While(_, _, e0, b) => {
                while self.expr(e0).as_bool() {
                    self.block(b);
                }
                Value::from(Tuple::new(vec![]))
            }
            Expr::Fun(_, _, _, _, _) => unreachable!(),
            Expr::Err(_, _) => unreachable!(),
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => todo!(),
        }
    }
}
