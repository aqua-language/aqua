#![allow(unused)]
use std::rc::Rc;

use crate::ast;
use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::mir;
use crate::mir::BlockId;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Terminator;
use crate::syntax::span::Span;

impl Function {
    fn do_tree(&self, b: BlockId, loops: &mut Vec<BlockId>) -> Block {
        let merge_nodes = self.domtree[b]
            .clone()
            .into_iter()
            .filter(|&b1| self.is_merge_node(b1))
            .collect::<Vec<_>>();

        if self.is_loop_header(b) {
            loops.push(b);
            let block = self.node_within(b, merge_nodes, loops);
            loops.pop();
            Block {
                span: block.span,
                stmts: vec![Stmt::Expr(Rc::new(Expr::Loop(
                    Span::Generated,
                    Type::Unit,
                    Some(Name::from(b)),
                    Rc::new(block),
                )))],
                expr: None,
            }
        } else {
            self.node_within(b, merge_nodes, loops)
        }
    }

    fn node_within(
        &self,
        b: BlockId,
        mut merge_nodes: Vec<BlockId>,
        loops: &mut Vec<BlockId>,
    ) -> Block {
        if let Some(merge_node) = merge_nodes.pop() {
            let mut block1 = self.do_tree(merge_node, loops);
            let block2 = self.node_within(b, merge_nodes, loops);
            block1.stmts.extend(block2.stmts);
            block1
        } else {
            let mut stmts = Vec::new();

            for s in &self.blocks[b].stmts {
                match &s.op {
                    Operation::Assign(place, rvalue) => {
                        let rhs_expr = match rvalue {
                            Rvalue::Use(op) => self.operand_to_expr(op),
                            Rvalue::Ref { mutable, place } => Expr::Ref(
                                Span::Generated,
                                place.ty().clone(),
                                Rc::new(Expr::Place(
                                    Span::Generated,
                                    place.ty().clone(),
                                    place.clone(),
                                )),
                                *mutable,
                            ),
                        };
                        stmts.push(Stmt::Expr(Rc::new(Expr::Assign(
                            Span::Generated,
                            Type::Unit,
                            Rc::new(Expr::Place(
                                Span::Generated,
                                place.ty().clone(),
                                place.clone(),
                            )),
                            Rc::new(rhs_expr),
                        ))));
                    }
                    Operation::StorageLive(_l) => {
                        todo!()
                        //     stmts.push(Stmt::Local(StmtLocal::new(
                        //         Span::Generated,
                        //         l.clone(),
                        //         None,
                        //     )));
                    }
                    Operation::StorageDead(_) => {}
                    Operation::Call {
                        dest: _,
                        func,
                        args,
                    } => {
                        let Operand::Function(_func_name, _func_ts) = func else {
                            todo!()
                        };
                        let _arg_exprs: Vec<Expr> =
                            args.iter().map(|a| self.operand_to_expr(a)).collect();

                        todo!()
                    }
                    Operation::Noop => {}
                }
            }

            // Handle terminator
            match self.blocks[b].terminator.clone().unwrap() {
                Terminator::Return => {
                    let local = &self.locals[0];
                    stmts.push(Stmt::Expr(Rc::new(Expr::Return(
                        Span::Generated,
                        local.ty.clone(),
                        Rc::new(Expr::Local(
                            Span::Generated,
                            local.ty.clone(),
                            local.name,
                            local.mutable,
                        )),
                    ))));
                }
                Terminator::Goto(l) => {
                    stmts.extend(self.do_branch(b, l, loops).stmts);
                }
                Terminator::ConditionalGoto(cond, t, f) => {
                    let cond_expr = self.operand_to_expr(&cond);
                    let then_block = self.do_branch(b, t, loops);
                    let else_block = self.do_branch(b, f, loops);
                    stmts.push(Stmt::Expr(Rc::new(Expr::IfElse(
                        Span::Generated,
                        then_block.ty().clone(),
                        Rc::new(cond_expr),
                        Rc::new(then_block),
                        Rc::new(else_block),
                    ))));
                }
            };

            Block {
                span: Span::Generated,
                stmts,
                expr: None,
            }
        }
    }

    fn is_loop_header(&self, b: BlockId) -> bool {
        self.predecessors[b]
            .iter()
            .any(|&pred| self.is_backward_edge(pred, b))
    }

    fn is_merge_node(&self, b: BlockId) -> bool {
        self.predecessors[b].len() > 1
    }

    fn is_backward_edge(&self, source: BlockId, target: BlockId) -> bool {
        self.reverse_postorder_number[target] <= self.reverse_postorder_number[source]
    }

    fn do_branch(&self, source: BlockId, target: BlockId, loops: &mut Vec<BlockId>) -> Block {
        if self.is_backward_edge(source, target) {
            Block {
                span: Span::Generated,
                stmts: vec![Stmt::Expr(Rc::new(Expr::Continue(
                    Span::Generated,
                    Type::Unit,
                    Some(Name::from(target)),
                )))],
                expr: None,
            }
        } else if self.is_merge_node(target) {
            if loops.contains(&target) {
                Block {
                    span: Span::Generated,
                    stmts: vec![Stmt::Expr(Rc::new(Expr::Break(
                        Span::Generated,
                        Type::Unit,
                        Some(Name::from(target)),
                    )))],
                    expr: None,
                }
            } else {
                Block {
                    span: Span::Generated,
                    stmts: Vec::new(),
                    expr: None,
                }
            }
        } else {
            self.do_tree(target, loops)
        }
    }

    fn operand_to_expr(&self, op: &Operand) -> Expr {
        match op {
            Operand::Constant(c) => match c {
                mir::Constant::Int(_i) => {
                    todo!()
                }
                mir::Constant::Bool(_b) => {
                    todo!()
                }
                mir::Constant::String(_s) => {
                    todo!()
                }
                mir::Constant::Unit => Expr::Unit(Span::Generated, Type::Unit),
                mir::Constant::Float(_s) => todo!(),
                mir::Constant::Char(_) => todo!(),
            },
            Operand::Copy(p) | Operand::Move(p) => {
                Expr::Place(Span::Generated, p.ty().clone(), p.clone())
            }
            Operand::Function(_x, _ts) => {
                todo!()
            }
        }
    }
}

impl Function {
    pub fn into_ast(self) -> ast::StmtDef {
        todo!()
        // let mut block = Block {
        //     stmts: vec![Stmt::Local(self.locals[0].clone(), None)],
        //     expr: None,
        // };
        // let mut env = Vec::new();
        // block.stmts.extend(self.do_tree(0, &mut env).stmts);
        // ast::Function {
        //     id: self.id.clone(),
        //     params: self.params.clone(),
        //     ty: self.ty.clone(),
        //     block,
        // }
    }
}
