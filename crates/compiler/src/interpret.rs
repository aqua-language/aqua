use std::collections::HashMap;

use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
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
    pub stack: Stack,
    pub report: Report,
}

#[derive(Debug)]
pub struct Stack {
    defs: HashMap<Name, StmtDef>,
    locals: Vec<Scope>,
}

impl Default for Stack {
    fn default() -> Self {
        Stack {
            defs: HashMap::default(),
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
        self.locals.last_mut().unwrap().0.insert(name, val);
    }

    fn update_val(&mut self, name: Name, val: Value) {
        self.locals
            .iter_mut()
            .rev()
            .find_map(|scope| scope.0.get_mut(&name))
            .map(|v| *v = val);
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
struct Scope(Map<Name, Value>);

impl Scope {
    fn new() -> Scope {
        Scope(Map::new())
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
            Stmt::Impl(_) => {}
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
        let value = self.eval_expr(&s.expr);
        self.stack.bind_val(s.name, value);
    }

    fn decl_stmt_def(&mut self, s: &StmtDef) {
        self.stack.bind_def(s.name, s.clone());
    }

    /// Todo: Remove this once we can compile to SSA.
    fn stmt_expr(&mut self, e: &Expr) {
        self.eval_expr(e);
    }

    fn eval_block(&mut self, b: &Block) -> Value {
        self.scoped(|this| {
            b.stmts.iter().for_each(|s| this.stmt(s));
            this.eval_expr(&b.expr)
        })
    }

    pub fn eval_expr(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Path(_, _, _) => unreachable!(),
            Expr::Int(_, t, v) => {
                let Type::Cons(x, _) = t else {
                    unreachable!();
                };
                match x.data.as_str() {
                    "i8" => Value::I8(v.as_str().parse().unwrap()),
                    "i16" => Value::I16(v.as_str().parse().unwrap()),
                    "i32" => Value::I32(v.as_str().parse().unwrap()),
                    "i64" => Value::I64(v.as_str().parse().unwrap()),
                    "u8" => Value::U8(v.as_str().parse().unwrap()),
                    "u16" => Value::U16(v.as_str().parse().unwrap()),
                    "u32" => Value::U32(v.as_str().parse().unwrap()),
                    "u64" => Value::U64(v.as_str().parse().unwrap()),
                    _ => unreachable!(),
                }
            }
            Expr::Float(_, t, v) => {
                let Type::Cons(x, _) = t else {
                    unreachable!();
                };
                match x.data.as_str() {
                    "f32" => Value::F32(v.as_str().parse().unwrap()),
                    "f64" => Value::F64(v.as_str().parse().unwrap()),
                    _ => unreachable!(),
                }
            }
            Expr::Bool(_, _, b) => Value::Bool(*b),
            Expr::Char(_, _, v) => Value::Char(*v),
            Expr::String(_, _, v) => Value::from(runtime::prelude::String::from(v.as_str())),
            Expr::Struct(_, _, _, _, xes) => {
                let xvs = xes.iter().map(|(n, e)| (*n, self.eval_expr(e))).collect();
                Value::from(Record::new(xvs))
            }
            Expr::Tuple(_, _, es) => {
                let vs = es.iter().map(|e| self.eval_expr(e)).collect();
                Value::from(Tuple::new(vs))
            }
            Expr::Record(_, _, xes) => {
                let xvs = xes.iter().map(|(n, e)| (*n, self.eval_expr(e))).collect();
                Value::from(Record::new(xvs))
            }
            Expr::Enum(_, _, _, _, x1, e) => Variant::new(*x1, self.eval_expr(e)).into(),
            Expr::Field(_, _, e, x) => self.eval_expr(e).as_record()[x].clone(),
            Expr::Index(_, _, e, i) => self.eval_expr(e).as_tuple()[i].clone(),
            Expr::Var(_, _, x) => self.stack.get_val(x).clone(),
            Expr::Def(_, _, x, ts) => Value::from(Fun::new(*x, ts.clone())),
            Expr::TraitMethod(_, _, _, _, _) => unreachable!(),
            Expr::Call(_, _, e, es) => {
                let f = self.eval_expr(e).as_function();
                let vs = es.iter().map(|e| self.eval_expr(e)).collect::<Vec<_>>();
                let s = self.stack.get_def(&f.x).clone();
                match s.body {
                    StmtDefBody::UserDefined(e) => self.scoped(|ctx| {
                        for (x, v) in s.params.keys().zip(vs) {
                            ctx.stack.bind_val(*x, v)
                        }
                        ctx.eval_expr(&e)
                    }),
                    StmtDefBody::Builtin(b) => (b.fun)(self, &f.ts, &vs),
                }
            }
            Expr::Block(_, _, b) => self.eval_block(b),
            // TODO: Make unreachable once we can compile to SSA.
            Expr::Query(..) => unreachable!(),
            Expr::QueryInto(..) => unreachable!(),
            Expr::Match(_, _, _, _) => unreachable!(),
            Expr::Array(_, _, _) => unreachable!(),
            Expr::Assign(_, _, e0, e1) => match e0.as_ref() {
                Expr::Var(_, _, x) => {
                    let v = self.eval_expr(e1);
                    self.stack.update_val(*x, v.clone());
                    v
                }
                _ => unreachable!(),
            },
            Expr::Return(_, _, _) => unreachable!(),
            Expr::Continue(_, _) => unreachable!(),
            Expr::Break(_, _) => unreachable!(),
            Expr::While(_, _, e0, b) => {
                while self.eval_expr(e0).as_bool() {
                    self.eval_block(b);
                }
                Value::from(Tuple::new(vec![]))
            }
            Expr::Fun(_, _, _, _, _) => unreachable!(),
            Expr::Err(_, _) => unreachable!(),
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => unreachable!(),
            Expr::Unresolved(_, _, _, _) => unreachable!(),
            Expr::InfixBinaryOp(_, _, _, _, _) => todo!(),
            Expr::PrefixUnaryOp(_, _, _, _) => todo!(),
            Expr::PostfixUnaryOp(_, _, _, _) => todo!(),
            Expr::Annotate(_, _, _) => unreachable!(),
            Expr::Paren(_, _, _) => unreachable!(),
            Expr::Dot(_, _, _, _, _, _) => unreachable!(),
            Expr::IfElse(_, _, e0, b0, b1) => {
                if self.eval_expr(e0).as_bool() {
                    self.eval_block(b0)
                } else {
                    self.eval_block(b1)
                }
            }
            Expr::IntSuffix(_, _, _, _) => unreachable!(),
            Expr::FloatSuffix(_, _, _, _) => unreachable!(),
            Expr::LetIn(_, _, _, _, _, _) => todo!(),
            Expr::Update(_, _, _, _, _) => todo!(),
        }
    }
}
