use std::rc::Rc;

use crate::ast;
use crate::ast::Loan;
use crate::ast::Local;
use crate::ast::Type;
use crate::collections::set::Set;
use crate::mir;
use crate::mir::BlockId;
use crate::report::span::Span;

pub struct Context {
    func: mir::Function,
    temp_counter: usize,
    stack: Vec<Scope>,
    loops: Vec<(Option<ast::Name>, (mir::BlockId, mir::BlockId))>,
}

#[derive(Debug)]
pub struct Scope {
    locals: Vec<mir::Local>,
    subst: Vec<(ast::Local, mir::Local)>,
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

impl ast::StmtDef {
    pub fn into_mir(self) -> mir::Function {
        todo!()
        // let func = mir::Function {
        //     span: self.span,
        //     name: self.name.into(),
        //     params: self.params,
        //     locals: vec![],
        //     ty: self.ty.clone(),
        //     blocks: vec![mir::BasicBlock {
        //         id: 0,
        //         terminator: None,
        //         stmts: vec![],
        //         live_in: Set::new(),
        //         live_out: Set::new(),
        //         dom: Set::new(),
        //     }],
        //     domtree: vec![],
        //     successors: vec![],
        //     predecessors: vec![],
        //     postorder: vec![],
        //     preorder: vec![],
        //     reverse_postorder_number: vec![],
        // };
        // let mut ctx = Context::new(func);
        // let l0 = ctx.new_local(self.span, self.ty);
        // ctx.scoped(|ctx| {
        //     let (b1, o1) = ctx.lower_expr(&self.body.as_udf().unwrap(), 0);
        //     ctx.func.blocks[b1]
        //         .stmts
        //         .push(mir::Stmt::new(mir::Operation::Assign(
        //             mir::Place::from(l0.clone()),
        //             mir::Rvalue::Use(o1.clone()),
        //         )));
        //     ctx.func.blocks[b1]
        //         .terminator
        //         .get_or_insert(mir::Terminator::Return);
        //     (b1, o1)
        // });
        // ctx.func
    }
}

impl Context {
    fn push_scope(&mut self) {
        self.stack.push(Scope {
            locals: vec![],
            subst: vec![],
        });
    }

    fn push_loop(&mut self, l: Option<ast::Name>, b_continue: mir::BlockId, b_break: mir::BlockId) {
        self.loops.push((l, (b_continue, b_break)))
    }

    fn get_loop(&mut self, l0: Option<ast::Name>) -> (mir::BlockId, mir::BlockId) {
        if let Some(l0) = l0 {
            self.loops
                .iter()
                .rev()
                .find_map(|(l1, v)| if *l1 == Some(l0) { Some(*v) } else { None })
                .unwrap()
        } else {
            self.loops.last().unwrap().1
        }
    }

    fn get_return_local(&mut self) -> &mir::Local {
        self.func.locals.first().unwrap()
    }

    fn pop_loop(&mut self) {
        self.loops.pop();
    }

    fn rename(&mut self, l1: ast::Local, l2: mir::Local) {
        self.stack.last_mut().unwrap().subst.push((l1, l2));
    }

    fn lookup(&self, l0: ast::Local) -> Option<&mir::Local> {
        self.stack.iter().rev().find_map(|scope| {
            scope
                .subst
                .iter()
                .find_map(|(l1, l2)| if l0.name == l1.name { Some(l2) } else { None })
        })
    }

    fn scoped(
        &mut self,
        f: impl FnOnce(&mut Self) -> (mir::BlockId, mir::Operand),
    ) -> (mir::BlockId, mir::Operand) {
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
                .push(mir::Stmt::new(mir::Operation::Dead(l)));
        }
    }

    pub fn lower_block(
        &mut self,
        b: &ast::Block,
        b0: mir::BlockId,
    ) -> (mir::BlockId, mir::Operand) {
        let b1 = b.stmts.iter().fold(b0, |b1, s| match s {
            ast::Stmt::Local(s) => {
                if let Some(e) = &s.expr {
                    let (b1, o1) = self.lower_expr(&e, b1);
                    let l1 = self.new_storage_local(s.span, s.local.ty.clone(), b1);
                    self.rename(s.local.clone(), l1.clone());
                    self.func.blocks[b1]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place::from(l1.clone()),
                            mir::Rvalue::Use(o1),
                        )));
                    b1
                } else {
                    todo!()
                }
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
            (b1, mir::Constant::Unit.into())
        }
    }

    /// TODO: This should return an Option so that we can short-circuit when we return.
    /// Returns the current block and the local that holds the result of the expression.
    pub fn lower_expr(&mut self, e: &ast::Expr, b0: mir::BlockId) -> (mir::BlockId, mir::Operand) {
        match e {
            ast::Expr::Int(_, _, v) => (b0, mir::Constant::Int(*v).into()),
            ast::Expr::Bool(_, _, v) => (b0, mir::Constant::Bool(*v).into()),
            ast::Expr::String(_, _, v) => (b0, mir::Constant::String(*v).into()),
            ast::Expr::Float(_, _, v) => (b0, mir::Constant::Float(*v).into()),
            ast::Expr::Char(_, _, v) => (b0, mir::Constant::Char(*v).into()),
            ast::Expr::Def(_, _, _x, _ts) => {
                todo!()
                // (b0, mir::Operand::Function(x.clone(), ts.clone()))
            }
            ast::Expr::IfElse(s, t, e0, e1, e2) => {
                let (b0, o0) = self.lower_expr(e0, b0);
                let b1_start = self.new_block();
                let b2_start = self.new_block();
                let b3 = self.new_block();
                let l3 = self.new_storage_local(*s, t.clone(), b0);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::IfElse(o0, b1_start, b2_start));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(e1, b1_start);
                    ctx.func.blocks[b1]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place::from(l3.clone()),
                            mir::Rvalue::Use(l1.clone()),
                        )));
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(mir::Terminator::Goto(b3));
                    (b1, l1)
                });

                self.scoped(|ctx| {
                    let (b2, l2) = ctx.lower_block(e2, b2_start);
                    ctx.func.blocks[b2]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place::from(l3.clone()),
                            mir::Rvalue::Use(l2.clone()),
                        )));
                    ctx.func.blocks[b2]
                        .terminator
                        .get_or_insert(mir::Terminator::Goto(b3));
                    (b2, l2)
                });

                (b3, mir::Operand::from(l3))
            }
            ast::Expr::While(_, _, l, e, b) => {
                let b_header = self.new_block();
                let b_body = self.new_block();
                let b_after = self.new_block();

                self.push_loop(*l, b_header, b_after);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::Goto(b_header));

                let (b0, l0) = self.lower_expr(e, b_header);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::IfElse(l0, b_body, b_after));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(b, b_body);
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(mir::Terminator::Goto(b_header));
                    (b1, l1)
                });

                self.pop_loop();

                (b_after, mir::Operand::Constant(mir::Constant::Unit))
            }
            ast::Expr::Loop(_, _, l, e1) => {
                let b_body = self.new_block();
                let b_after = self.new_block();

                self.push_loop(*l, b_body, b_after);

                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::Goto(b_body));

                self.scoped(|ctx| {
                    let (b1, l1) = ctx.lower_block(e1, b_body);
                    ctx.func.blocks[b1]
                        .terminator
                        .get_or_insert(mir::Terminator::Goto(b_body));
                    (b1, l1)
                });

                self.pop_loop();

                (b_after, mir::Operand::Constant(mir::Constant::Unit))
            }
            ast::Expr::Tuple(s, t, es) => {
                let l0 = self.new_storage_local(*s, t.clone(), b0);
                let b0 = es.iter().enumerate().fold(b0, |b0, (i, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place {
                                span: e.span(),
                                local: l0.clone(),
                                elems: vec![mir::PlaceElem::Index(
                                    e.span(),
                                    e.ty().clone(),
                                    mir::Index::new(e.span(), i),
                                )],
                            },
                            mir::Rvalue::Use(l1),
                        )));
                    b1
                });
                (b0, mir::Operand::from(l0))
            }
            ast::Expr::Ref(s, t, e, m) => {
                let p = e.as_place().unwrap();
                let l = self.new_storage_local(*s, t.clone(), b0);
                let p1 = self.resolve_place(p.clone());
                self.func.blocks[b0]
                    .stmts
                    .push(mir::Stmt::new(mir::Operation::Assign(
                        mir::Place::from(l.clone()),
                        mir::Rvalue::Ref {
                            mutable: *m,
                            place: p1.clone(),
                        },
                    )));
                (b0, mir::Operand::from(l))
            }
            ast::Expr::Place(_, _, p0) => {
                let p1 = self.resolve_place(p0.clone());
                (b0, mir::Operand::from(p1))
            }
            ast::Expr::Assign(_, _, e0, e1) => {
                let p0 = e0.as_place().unwrap();
                let p0 = self.resolve_place(p0.clone());
                let (b0, l0) = self.lower_expr(e1, b0);
                self.func.blocks[b0]
                    .stmts
                    .push(mir::Stmt::new(mir::Operation::Assign(
                        p0,
                        mir::Rvalue::Use(l0),
                    )));
                (b0, mir::Operand::Constant(mir::Constant::Unit))
            }
            ast::Expr::Block(_, _, b) => self.scoped(|ctx| ctx.lower_block(b, b0)),
            ast::Expr::Unit(_, _) => (b0, mir::Operand::Constant(mir::Constant::Unit)),
            ast::Expr::Call(s, t, e, es) => {
                let (b0, o0) = self.lower_expr(e, b0);
                let (b0, os) = es.iter().fold((b0, vec![]), |(b0, mut os), e| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    os.push(l1);
                    (b1, os)
                });
                let l1 = self.new_storage_local(*s, t.clone(), b0);
                self.func.blocks[b0]
                    .stmts
                    .push(mir::Stmt::new(mir::Operation::Call {
                        dest: mir::Place::from(l1.clone()),
                        func: o0,
                        args: os,
                    }));
                (b0, mir::Operand::from(l1))
            }
            ast::Expr::Return(_, _, e0) => {
                let (b0, l0) = self.lower_expr(e0, b0);
                let l1 = self.get_return_local().clone();
                self.func.blocks[b0]
                    .stmts
                    .push(mir::Stmt::new(mir::Operation::Assign(
                        mir::Place::from(l1),
                        mir::Rvalue::from(l0),
                    )));
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::Return);
                (b0, mir::Operand::Constant(mir::Constant::Unit))
            }
            ast::Expr::Continue(_, _, l) => {
                let (b_continue, _) = self.get_loop(*l);
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::Goto(b_continue));
                (b0, mir::Operand::Constant(mir::Constant::Unit))
            }
            ast::Expr::Break(_, _, l) => {
                let (_, b_break) = self.get_loop(*l);
                self.func.blocks[b0]
                    .terminator
                    .get_or_insert(mir::Terminator::Goto(b_break));
                (b0, mir::Operand::Constant(mir::Constant::Unit))
            }
            // S { x: e, ... }
            // =>
            // let l: S;
            // l.x = e;
            // ...
            ast::Expr::Struct(s, t, _, _, xes) => {
                let l = self.new_storage_local(*s, t.clone(), b0);
                let b0 = xes.iter().fold(b0, |b0, (x, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place {
                                span: e.span(),
                                local: l.clone(),
                                elems: vec![mir::PlaceElem::Field(e.span(), e.ty().clone(), *x)],
                            },
                            mir::Rvalue::Use(l1),
                        )));
                    b1
                });
                (b0, mir::Operand::from(l))
            }
            // record(x: e, ...)
            // =>
            // let l: record(x: T, ...);
            ast::Expr::Record(s, t, xes) => {
                let l = self.new_storage_local(*s, t.clone(), b0);
                let b0 = xes.iter().fold(b0, |b0, (x, e)| {
                    let (b1, l1) = self.lower_expr(e, b0);
                    self.func.blocks[b1]
                        .stmts
                        .push(mir::Stmt::new(mir::Operation::Assign(
                            mir::Place {
                                span: e.span(),
                                local: l.clone(),
                                elems: vec![mir::PlaceElem::Field(e.span(), e.ty().clone(), *x)],
                            },
                            mir::Rvalue::Use(l1),
                        )));
                    b1
                });
                (b0, mir::Operand::from(l))
            }
            ast::Expr::Enum(..) => todo!(),
            ast::Expr::Assoc(..) => todo!(),
            // TODO
            ast::Expr::Array(..) => todo!(),
            // Unreachable
            ast::Expr::Path(..) => unreachable!(),
            ast::Expr::Deref(..) => unreachable!(),
            ast::Expr::IntSuffix(..) => unreachable!(),
            ast::Expr::FloatSuffix(..) => unreachable!(),
            ast::Expr::Closure(..) => unreachable!(),
            ast::Expr::Field(..) => unreachable!(),
            ast::Expr::Index(..) => unreachable!(),
            ast::Expr::Local(..) => unreachable!(),
            ast::Expr::Query(..) => unreachable!(),
            ast::Expr::QueryInto(..) => unreachable!(),
            ast::Expr::Match(..) => unreachable!(),
            ast::Expr::Lambda(..) => unreachable!(),
            ast::Expr::For(..) => unreachable!(),
            ast::Expr::Err(..) => unreachable!(),
            ast::Expr::InfixBinaryOp(..) => unreachable!(),
            ast::Expr::PrefixUnaryOp(..) => unreachable!(),
            ast::Expr::PostfixUnaryOp(..) => unreachable!(),
            ast::Expr::Annotate(..) => unreachable!(),
            ast::Expr::Paren(..) => unreachable!(),
            ast::Expr::Dot(..) => unreachable!(),
            ast::Expr::Anonymous(..) => unreachable!(),
        }
    }

    fn resolve_place(&mut self, p0: ast::Place) -> mir::Place {
        if let Some(l) = self.lookup(p0.local.clone()) {
            let p1 = mir::Place {
                span: p0.span,
                local: l.clone(),
                elems: p0.elems.clone(),
            };
            p1.clone()
        } else {
            p0.clone()
        }
    }

    fn resolve_type(&mut self, t: ast::Type) -> mir::Type {
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
            .push(mir::Stmt::new(mir::Operation::Live(l.clone())));
        l
    }

    fn new_local(&mut self, s: Span, t: mir::Type) -> mir::Local {
        let id = self.temp_counter;
        self.temp_counter += 1;
        let l = mir::Local {
            span: s,
            name: mir::Name::new(s, format!("_{}", id)),
            ty: self.resolve_type(t),
            mutable: false,
        };
        self.func.locals.push(l.clone());
        if let Some(scope) = self.stack.last_mut() {
            scope.locals.push(l.clone());
        }
        l
    }

    fn new_block(&mut self) -> mir::BlockId {
        let block_id = self.func.blocks.len();
        self.func.blocks.push(mir::BasicBlock {
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

impl From<mir::Place> for mir::Operand {
    fn from(place: mir::Place) -> mir::Operand {
        // if place.ty().is_copy() {
        //     mir::Operand::Copy(place)
        // } else {
        mir::Operand::Move(place)
        // }
    }
}

impl From<mir::Local> for mir::Place {
    fn from(local: mir::Local) -> mir::Place {
        mir::Place {
            span: local.span,
            local,
            elems: vec![],
        }
    }
}

impl From<ast::Local> for mir::Operand {
    fn from(local: ast::Local) -> mir::Operand {
        // if local.ty.is_copy() {
        //     let place = mir::Place::from(name);
        //     mir::Operand::Copy(place)
        // } else {
        let place = mir::Place::from(local);
        mir::Operand::Move(place)
        // }
    }
}

impl From<mir::Operand> for mir::Rvalue {
    fn from(op: mir::Operand) -> mir::Rvalue {
        mir::Rvalue::Use(op)
    }
}

impl From<mir::Constant> for mir::Operand {
    fn from(c: mir::Constant) -> mir::Operand {
        mir::Operand::Constant(c)
    }
}
