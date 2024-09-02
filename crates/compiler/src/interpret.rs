use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::builtins::types::function::Fun;
use crate::builtins::types::record::Record;
use crate::builtins::types::tuple::Tuple;
use crate::builtins::types::variant::Variant;
use crate::builtins::value::Value;
use crate::declare;
use crate::diag::Report;
use crate::package;
use crate::traversal::visitor::AcceptVisitor;

#[derive(Debug, Default)]
pub struct Context {
    pub stack: Stack,
    pub report: Report,
    pub decls: declare::Context,
    pub workspace: package::Workspace,
}

#[derive(Debug)]
pub struct Stack(Vec<Scope>);

impl Default for Stack {
    fn default() -> Self {
        Stack(vec![Scope::new()])
    }
}

impl Stack {
    fn new() -> Stack {
        Self::default()
    }

    pub fn bind(&mut self, name: Name, val: Value) {
        self.0.last_mut().unwrap().0.insert(name, val);
    }

    fn update(&mut self, name: Name, val: Value) {
        self.0
            .iter_mut()
            .rev()
            .find_map(|scope| scope.0.get_mut(&name))
            .map(|v| *v = val);
    }

    fn get(&self, name: &Name) -> &Value {
        self.0
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
            decls: declare::Context::default(),
            workspace: package::Workspace::new(),
        }
    }

    pub fn scoped(&mut self, f: impl FnOnce(&mut Self) -> Value) -> Value {
        self.stack.0.push(Scope::new());
        let v = f(self);
        self.stack.0.pop();
        v
    }

    pub fn interpret(&mut self, p: &Program) {
        p.visit(&mut self.decls);
        p.stmts.iter().for_each(|stmt| self.stmt(stmt));
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
        self.stack.bind(s.name, value);
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
            Expr::Var(_, _, x) => self.stack.get(x).clone(),
            Expr::Def(_, _, x, _) => {
                let s = self.decls.defs.get(x).unwrap().clone();
                Fun::new(s.params.clone(), s.body.clone()).into()
            }
            Expr::TraitMethod(_, _, _, _, _) => unreachable!(),
            Expr::Call(_, _, e, es) => {
                let f = self.eval_expr(e).as_function();
                let vs = es.iter().map(|e| self.eval_expr(e)).collect::<Vec<_>>();
                match f.body {
                    ExprBody::UserDefined(e) => self.scoped(|ctx| {
                        for (x, v) in f.params.keys().zip(vs) {
                            ctx.stack.bind(*x, v)
                        }
                        ctx.eval_expr(&e)
                    }),
                    ExprBody::Builtin(b) => (b.fun)(self, &vs),
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
                    self.stack.update(*x, v.clone());
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
                Tuple::new(vec![]).into()
            }
            Expr::Fun(_, _, xts, _, e) => {
                Fun::new(xts.clone(), ExprBody::UserDefined(e.clone())).into()
            }
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
            Expr::Anonymous(_, _) => unreachable!(),
        }
    }
}
