use crate::ast::Local;
use crate::ast::Place;
use crate::collections::set::Set;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Terminator;

impl Operation {
    fn used(&self) -> Vec<Place> {
        match self {
            Operation::Assign(_, rv) => match rv {
                Rvalue::Use(op) => op.used(),
                Rvalue::Ref { place, .. } => {
                    let mut v = vec![place.clone()];
                    for loan in place.local.ty.loans() {
                        v.push(loan.place.clone());
                    }
                    v
                }
            },
            Operation::Call { func, args, .. } => func
                .used()
                .into_iter()
                .chain(args.iter().flat_map(|arg| arg.used().into_iter()))
                .collect(),
            Operation::Live(_) => vec![],
            Operation::Dead(_) => vec![],
            Operation::Noop => vec![],
        }
    }

    fn moved(&self) -> Vec<Place> {
        match self {
            Operation::Assign(_, rv) => match rv {
                Rvalue::Use(op) => op.moved(),
                Rvalue::Ref { place, .. } => {
                    let mut v = vec![place.clone()];
                    for loan in place.local.ty.loans() {
                        v.push(loan.place.clone());
                    }
                    v
                }
            },
            Operation::Call { func, args, .. } => func
                .moved()
                .into_iter()
                .chain(args.iter().flat_map(|arg| arg.moved()))
                .collect(),
            Operation::Live(_) => vec![],
            Operation::Dead(_) => vec![],
            Operation::Noop => vec![],
        }
    }

    fn defined(&self) -> Vec<Place> {
        match self {
            Operation::Assign(p, _) => vec![p.clone()],
            Operation::Call { dest, .. } => vec![dest.clone()],
            Operation::Live(_) => vec![],
            Operation::Dead(_) => vec![],
            Operation::Noop => vec![],
        }
    }

    fn _storage_live(&self) -> Vec<Local> {
        match self {
            Operation::Assign(..) => vec![],
            Operation::Call { .. } => vec![],
            Operation::Live(l) => vec![l.clone()],
            Operation::Dead(_) => vec![],
            Operation::Noop => vec![],
        }
    }

    fn _storage_dead(&self) -> Vec<Local> {
        match self {
            Operation::Assign(_, _) => vec![],
            Operation::Call { .. } => vec![],
            Operation::Live(_) => vec![],
            Operation::Dead(l) => vec![l.clone()],
            Operation::Noop => vec![],
        }
    }
}

impl Operand {
    fn used(&self) -> Vec<Place> {
        match self {
            Operand::Constant(_) | Operand::Function(_, _) => vec![],
            Operand::Copy(p) | Operand::Move(p) => {
                let mut v = vec![p.clone()];
                for loan in p.local.ty.loans() {
                    v.push(loan.place.clone());
                }
                v
            }
        }
    }

    fn moved(&self) -> Vec<Place> {
        match self {
            Operand::Constant(_) | Operand::Function(_, _) => vec![],
            Operand::Copy(_) => vec![],
            Operand::Move(p) => vec![p.clone()],
        }
    }
}

impl Function {
    pub fn compute_liveness(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;

            let old_blocks = self.blocks.clone();

            for block in self.blocks.iter_mut().rev() {
                let mut live_out = Set::new();
                if let Some(terminator) = &block.terminator {
                    // A block's live-out is the union of its successor's live-in
                    match terminator {
                        Terminator::Goto(b) | Terminator::IfElse(_, b, _) => {
                            live_out.extend(old_blocks[*b].live_in.iter().cloned());
                        }
                        Terminator::Return => {}
                    }
                }

                // Compute live-out for each statement in reverse order
                for stmt in block.stmts.iter_mut().rev() {
                    let old_live_out = stmt.live_out.clone();
                    let old_live_in = stmt.live_in.clone();

                    // Update live-out for the statement
                    stmt.live_out = live_out.clone();

                    // Update live-in for the statement
                    let used = stmt.op.used();
                    let moved = stmt.op.moved();
                    let defined = stmt.op.defined();

                    // live_in = (live_out - (defined U moved)) U used
                    stmt.live_in = stmt
                        .live_out
                        .iter()
                        .filter(|v| {
                            !defined.iter().any(|p| p.is_prefix_of(v))
                                && !moved.iter().any(|p| p.is_prefix_of(v))
                        })
                        .cloned()
                        .collect();
                    stmt.live_in.extend(used.iter().cloned());

                    // The live out of the next statement is the live in of the current statement
                    live_out = stmt.live_in.clone();

                    // If any live-in or live-out changed, we need to rerun the analysis
                    if old_live_out != stmt.live_out.clone() || old_live_in != stmt.live_in.clone()
                    {
                        changed = true;
                    }
                }

                // Update block's live-in and live-out
                block.live_out = live_out.clone();
                block.live_in = block
                    .stmts
                    .iter()
                    .flat_map(|stmt| stmt.live_in.iter().cloned())
                    .collect();
            }
        }
    }
}
