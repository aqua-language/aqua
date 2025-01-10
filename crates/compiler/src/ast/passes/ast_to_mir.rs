use std::rc::Rc;

use crate::ast;
use crate::ast::Expr;
use crate::ast::Index;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::PlaceElem;
use crate::ast::StmtDef;
use crate::ast::Type;
use crate::collections::set::Set;
use crate::mir;
use crate::mir::BasicBlock;
use crate::mir::BlockId;
use crate::mir::Constant;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Stmt;
use crate::mir::Terminator;
use crate::syntax::span::Span;

pub struct Context {
    func: mir::Function,
    temp_counter: usize,
    stack: Vec<Scope>,
    loops: Vec<(BlockId, BlockId)>,
}

#[derive(Debug)]
pub struct Scope {
    locals: Vec<Local>,
    subst: Vec<(Local, Local)>,
}

impl Context {
    pub fn new(function: mir::Function) -> Context {
        Context {
            func: function,
            temp_counter: 0,
            stack: vec![],
            loops: vec![],
        }
    }
}

impl StmtDef {
    pub fn into_mir(self) -> mir::Function {
        let func = mir::Function {
            name: self.name,
            params: self.params,
            locals: vec![],
            ty: self.ty.clone(),
            blocks: vec![BasicBlock {
                id: 0,
                terminator: None,
                stmts: vec![],
                live_in: Set::new(),
                live_out: Set::new(),
                dom: Set::new(),
            }],
            domtree: vec![],
            successors: vec![],
            predecessors: vec![],
            postorder: vec![],
            preorder: vec![],
            reverse_postorder_number: vec![],
        };
        let mut ctx = Context::new(func);
        let l0 = ctx.new_local(self.span, self.ty);
        ctx.scoped(|ctx| {
            let (b1, o1) = ctx.lower_expr(&self.body.as_udf().unwrap(), 0);
            ctx.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                Place::from(l0.clone()),
                Rvalue::Use(o1.clone()),
            )));
            ctx.func.blocks[b1]
                .terminator
                .get_or_insert(Terminator::Return);
            (b1, o1)
        });
        ctx.func
    }
}

impl Context {
    fn push_scope(&mut self) {
        self.stack.push(Scope {
            locals: vec![],
            subst: vec![],
        });
    }

    fn push_loop(&mut self, b_continue: BlockId, b_break: BlockId) {
        self.loops.push((b_continue, b_break))
    }

    fn get_loop(&mut self) -> (BlockId, BlockId) {
        *self.loops.last().unwrap()
    }

    fn get_return_local(&mut self) -> &Local {
        self.func.locals.first().unwrap()
    }

    fn pop_loop(&mut self) {
        self.loops.pop();
    }

    fn rename(&mut self, l1: Local, l2: Local) {
        self.stack.last_mut().unwrap().subst.push((l1, l2));
    }

    fn lookup(&self, l0: Local) -> Option<&Local> {
        self.stack.iter().rev().find_map(|scope| {
            scope
                .subst
                .iter()
                .find_map(|(l1, l2)| if l0.name == l1.name { Some(l2) } else { None })
        })
    }

    fn scoped(&mut self, f: impl FnOnce(&mut Self) -> (BlockId, Operand)) -> (BlockId, Operand) {
        self.push_scope();
        let (b, o) = f(self);
        self.pop_scope(b);
        (b, o)
    }

    fn pop_scope(&mut self, b: mir::BlockId) {
        let scope = self.stack.pop().unwrap();
        for l in scope.locals.into_iter().rev() {
            self.func.blocks[b]
                .stmts
                .push(Stmt::new(Operation::StorageDead(l)));
        }
    }

    pub fn lower_block(&mut self, b: &ast::Block, b0: mir::BlockId) -> (BlockId, Operand) {
        let b1 = b.stmts.iter().fold(b0, |b1, s| match s {
            ast::Stmt::Local(s) => {
                let (b1, o1) = self.lower_expr(&s.expr, b1);
                let l1 = self.new_storage_local(s.span, s.local.ty.clone(), b1);
                self.rename(s.local.clone(), l1.clone());
                self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                    Place::from(l1.clone()),
                    Rvalue::Use(o1),
                )));
                b1
            }
            ast::Stmt::Expr(e) => {
                let (b1, _) = self.lower_expr(e, b1);
                b1
            }
            _ => unreachable!(),
        });
        if let Some(e) = &b.expr {
            self.lower_expr(e, b1)
        } else {
            (b1, Operand::Constant(Constant::Unit))
        }
    }

    /// TODO: This should return an Option so that we can short-circuit when we return.
    /// Returns the current block and the local that holds the result of the expression.
    pub fn lower_expr(&mut self, e: &Expr, b0: BlockId) -> (BlockId, Operand) {
        match e {
            Expr::Int(_, _, v) => (b0, Operand::Constant(Constant::Int(*v))),
            Expr::Bool(_, _, v) => (b0, Operand::Constant(Constant::Bool(*v))),
            Expr::String(_, _, v) => (b0, Operand::Constant(Constant::String(*v))),
            Expr::Float(_, _, v) => (b0, Operand::Constant(Constant::Float(*v))),
            Expr::Char(_, _, v) => (b0, Operand::Constant(Constant::Char(*v))),
            Expr::Def(_, _, x, ts) => (b0, Operand::Function(x.clone(), ts.clone())),
            Expr::IfElse(s, t, e0, e1, e2) => {
                let (b0, o0) = self.lower_expr(e0, b0);
                let b1_start = self.new_block();
                let b2_start = self.new_block();
                let b3 = self.new_block();
                let l3 = self.new_storage_local(*s, t.clone(), b0);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::ConditionalGoto(o0, b1_start, b2_start));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(e1, b1_start);
                    ctx.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                        Place::from(l3.clone()),
                        Rvalue::Use(l1.clone()),
                    )));
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(Terminator::Goto(b3));
                    (b1, l1)
                });

                self.scoped(|ctx| {
                    let (b2, l2) = ctx.lower_block(e2, b2_start);
                    ctx.func.blocks[b2].stmts.push(Stmt::new(Operation::Assign(
                        Place::from(l3.clone()),
                        Rvalue::Use(l2.clone()),
                    )));
                    ctx.func.blocks[b2]
                        .terminator
                        .get_or_insert(Terminator::Goto(b3));
                    (b2, l2)
                });

                (b3, Operand::from(l3))
            }
            Expr::While(_, _, e, b) => {
                let b_header = self.new_block();
                let b_body = self.new_block();
                let b_after = self.new_block();

                self.push_loop(b_header, b_after);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::Goto(b_header));

                let (b0, l0) = self.lower_expr(e, b_header);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::ConditionalGoto(l0, b_body, b_after));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(b, b_body);
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(Terminator::Goto(b_header));
                    (b1, l1)
                });

                self.pop_loop();

                (b_after, Operand::Constant(Constant::Unit))
            }
            Expr::Loop(_, _, e1) => {
                let b_body = self.new_block();
                let b_after = self.new_block();

                self.push_loop(b_body, b_after);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::Goto(b_body));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(e1, b_body);
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(Terminator::Goto(b_body));
                    (b1, l1)
                });

                self.pop_loop();

                (b_after, Operand::Constant(Constant::Unit))
            }
            Expr::Tuple(s, t, es) => {
                let l0 = self.new_storage_local(*s, t.clone(), b0);
                let b0 = es.iter().enumerate().fold(b0, |b0, (i, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                        Place {
                            span: e.span(),
                            local: l0.clone(),
                            elems: vec![PlaceElem::Index(
                                e.span(),
                                e.ty().clone(),
                                Index::new(e.span(), i),
                            )],
                        },
                        Rvalue::Use(l1),
                    )));
                    b1
                });
                (b0, Operand::from(l0))
            }
            Expr::Ref(s, t, e, m) => {
                let p = e.as_place().unwrap();
                let l = self.new_storage_local(*s, t.clone(), b0);
                let p1 = self.resolve_place(p.clone());
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place::from(l.clone()),
                    Rvalue::Ref {
                        mutable: *m,
                        place: p1.clone(),
                    },
                )));
                (b0, Operand::from(l))
            }
            Expr::Place(_, _, p0) => {
                let p1 = self.resolve_place(p0.clone());
                (b0, Operand::from(p1))
            }
            Expr::Assign(_, _, e0, e1) => {
                let p0 = e0.as_place().unwrap();
                let p0 = self.resolve_place(p0.clone());
                let (b0, l0) = self.lower_expr(e1, b0);
                self.func.blocks[b0]
                    .stmts
                    .push(Stmt::new(Operation::Assign(p0, Rvalue::Use(l0))));
                (b0, Operand::Constant(Constant::Unit))
            }
            Expr::Block(_, _, b) => self.scoped(|ctx| ctx.lower_block(b, b0)),
            Expr::Unit(_, _) => (b0, Operand::Constant(Constant::Unit)),
            Expr::Call(s, t, e, es) => {
                let (b0, o0) = self.lower_expr(e, b0);
                let (b0, os) = es.iter().fold((b0, vec![]), |(b0, mut os), e| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    os.push(l1);
                    (b1, os)
                });
                let l1 = self.new_storage_local(*s, t.clone(), b0);
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Call {
                    dest: Place::from(l1.clone()),
                    func: o0,
                    args: os,
                }));
                (b0, Operand::from(l1))
            }
            Expr::Return(_, _, e0) => {
                let (b0, l0) = self.lower_expr(e0, b0);
                let l1 = self.get_return_local().clone();
                self.func.blocks[b0].stmts.push(Stmt::new(Operation::Assign(
                    Place::from(l1),
                    Rvalue::from(l0),
                )));
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::Return);
                (b0, Operand::Constant(Constant::Unit))
            }
            Expr::Continue(_, _) => {
                let (b_continue, _) = self.get_loop();
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::Goto(b_continue));
                (b0, Operand::Constant(Constant::Unit))
            }
            Expr::Break(_, _) => {
                let (_, b_break) = self.get_loop();
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(Terminator::Goto(b_break));
                (b0, Operand::Constant(Constant::Unit))
            }
            // S { x: e, ... }
            // =>
            // let l: S;
            // l.x = e;
            // ...
            Expr::Struct(s, t, _, _, xes) => {
                let l = self.new_storage_local(*s, t.clone(), b0);
                let b0 = xes.iter().fold(b0, |b0, (n, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                        Place {
                            span: e.span(),
                            local: l.clone(),
                            elems: vec![PlaceElem::Field(e.span(), e.ty().clone(), n.clone())],
                        },
                        Rvalue::Use(l1),
                    )));
                    b1
                });
                (b0, Operand::from(l))
            }
            // record(x: e, ...)
            // =>
            // let l: record(x: T, ...);
            Expr::Record(s, t, xes) => {
                let l = self.new_storage_local(*s, t.clone(), b0);
                let b0 = xes.iter().fold(b0, |b0, (n, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1].stmts.push(Stmt::new(Operation::Assign(
                        Place {
                            span: e.span(),
                            local: l.clone(),
                            elems: vec![PlaceElem::Field(e.span(), e.ty().clone(), n.clone())],
                        },
                        Rvalue::Use(l1),
                    )));
                    b1
                });
                (b0, Operand::from(l))
            }
            Expr::Enum(..) => todo!(),
            Expr::Assoc(..) => todo!(),
            // TODO
            Expr::Array(..) => todo!(),
            // Unreachable
            Expr::Path(..) => unreachable!(),
            Expr::Deref(..) => unreachable!(),
            Expr::IntSuffix(..) => unreachable!(),
            Expr::FloatSuffix(..) => unreachable!(),
            Expr::Closure(..) => unreachable!(),
            Expr::Field(..) => unreachable!(),
            Expr::Index(..) => unreachable!(),
            Expr::Local(..) => unreachable!(),
            Expr::Query(..) => unreachable!(),
            Expr::QueryInto(..) => unreachable!(),
            Expr::Match(..) => unreachable!(),
            Expr::Lambda(..) => unreachable!(),
            Expr::For(..) => unreachable!(),
            Expr::Err(..) => unreachable!(),
            Expr::InfixBinaryOp(..) => unreachable!(),
            Expr::PrefixUnaryOp(..) => unreachable!(),
            Expr::PostfixUnaryOp(..) => unreachable!(),
            Expr::Annotate(..) => unreachable!(),
            Expr::Paren(..) => unreachable!(),
            Expr::Dot(..) => unreachable!(),
            Expr::Anonymous(..) => unreachable!(),
        }
    }

    fn resolve_place(&mut self, p0: Place) -> Place {
        if let Some(l) = self.lookup(p0.local.clone()) {
            let p1 = Place {
                span: p0.span,
                local: l.clone(),
                elems: p0.elems.clone(),
            };
            p1.clone()
        } else {
            p0.clone()
        }
    }

    fn resolve_type(&mut self, t: Type) -> Type {
        match t {
            Type::Tuple(ts) => {
                let ts = ts.into_iter().map(|t| self.resolve_type(t)).collect();
                Type::Tuple(ts)
            }
            Type::Ref(loans, t, m) => {
                let loans = loans
                    .into_iter()
                    .map(|l| Loan {
                        place: self.resolve_place(l.place),
                        mutable: l.mutable,
                    })
                    .collect();
                let t = self.resolve_type(t.as_ref().clone());
                Type::Ref(loans, Rc::new(t), m)
            }
            _ => t,
        }
    }

    fn new_storage_local(&mut self, s: Span, t: Type, b: BlockId) -> Local {
        let l = self.new_local(s, t);
        self.func.blocks[b]
            .stmts
            .push(Stmt::new(Operation::StorageLive(l.clone())));
        l
    }

    fn new_local(&mut self, s: Span, t: Type) -> Local {
        let id = self.temp_counter;
        self.temp_counter += 1;
        let l = Local {
            span: s,
            name: Name::new(s, format!("_{}", id)),
            ty: self.resolve_type(t),
            mutable: false,
        };
        self.func.locals.push(l.clone());
        if let Some(scope) = self.stack.last_mut() {
            scope.locals.push(l.clone());
        }
        l
    }

    fn new_block(&mut self) -> BlockId {
        let block_id = self.func.blocks.len();
        self.func.blocks.push(BasicBlock {
            id: block_id,
            stmts: Vec::new(),
            terminator: None,
            live_in: Set::new(),
            live_out: Set::new(),
            dom: Set::new(),
        });
        block_id
    }
}

impl From<Place> for Operand {
    fn from(place: Place) -> Operand {
        if place.ty().is_copy() {
            Operand::Copy(place)
        } else {
            Operand::Move(place)
        }
    }
}

impl From<Local> for Place {
    fn from(local: Local) -> Place {
        Place {
            span: local.span,
            local,
            elems: vec![],
        }
    }
}

impl From<Local> for Operand {
    fn from(local: Local) -> Operand {
        let place = Place::from(local);
        if place.local.ty.is_copy() {
            Operand::Copy(place)
        } else {
            Operand::Move(place)
        }
    }
}

impl From<Operand> for Rvalue {
    fn from(op: Operand) -> Rvalue {
        Rvalue::Use(op)
    }
}
