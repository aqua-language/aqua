use std::collections::HashSet;

use crate::ast::Place;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;

impl Function {
    pub fn remove_unused_variables(&mut self) {
        let mut used = HashSet::new();

        used.insert(Place::from(self.locals[0].clone()));

        let mut changed = true;
        while changed {
            changed = false;

            for block in &self.blocks {
                for stmt in &block.stmts {
                    match &stmt.op {
                        Operation::Assign(dest, Rvalue::Use(operand)) => {
                            if used.contains(dest) {
                                for p in operand_places(operand) {
                                    changed = used.insert(p);
                                }
                            }
                        }
                        Operation::Assign(dest, Rvalue::Ref { mutable: _, place }) => {
                            if used.contains(dest) {
                                changed = used.insert(place.clone());
                            }
                        }
                        Operation::Call { dest, func, args } => {
                            changed = used.insert(Place::from(dest.clone()));
                            for p in operand_places(func) {
                                changed = used.insert(p);
                            }
                            for arg in args {
                                for p in operand_places(arg) {
                                    changed = used.insert(p);
                                }
                            }
                        }
                        Operation::Live(_) | Operation::Dead(_) => {}
                        Operation::Noop => {}
                    }
                }
            }
        }

        self.locals
            .retain(|l| used.contains(&Place::from(l.clone())));

        for block in &mut self.blocks {
            block.stmts.retain(|stmt| match &stmt.op {
                Operation::Assign(dest, _) => used.contains(dest),
                Operation::Live(local) => used.contains(&Place::from(local.clone())),
                Operation::Dead(local) => used.contains(&Place::from(local.clone())),
                Operation::Call { dest, .. } => used.contains(dest),
                _ => true,
            });
        }
    }

    pub fn with_remove_unused_variables(mut self) -> Self {
        self.remove_unused_variables();
        self
    }
}

fn operand_places(op: &Operand) -> Vec<Place> {
    match op {
        Operand::Constant(_) => vec![],
        Operand::Copy(p) => vec![p.clone()],
        Operand::Move(p) => vec![p.clone()],
        Operand::Function(_, _) => vec![],
    }
}
