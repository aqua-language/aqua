use std::collections::HashMap;

use crate::mir::Constant;
use crate::mir::Function;
use crate::mir::Operand;
use crate::mir::Operation;
use crate::mir::Rvalue;
use crate::mir::Terminator;

impl Function {
    pub fn constant_propagation(&mut self) {
        let mut map = HashMap::new();

        for b in 0..self.blocks.len() {
            for stmt in &mut self.blocks[b].stmts {
                match &mut stmt.op {
                    Operation::Assign(p, Rvalue::Use(Operand::Constant(c))) => {
                        map.insert(p.clone(), c.clone());
                    }
                    Operation::Assign(p, Rvalue::Use(Operand::Copy(src))) => {
                        if let Some(c) = map.get(src) {
                            stmt.op = Operation::Assign(
                                p.clone(),
                                Rvalue::Use(Operand::Constant(c.clone())),
                            );
                        }
                    }
                    Operation::Assign(p, _) => {
                        map.remove(p);
                    }
                    Operation::Call { dest, .. } => {
                        map.remove(dest);
                    }
                    _ => {}
                }
            }

            if let Some(terminator) = &mut self.blocks[b].terminator {
                match terminator {
                    Terminator::ConditionalGoto(cond, true_block, false_block) => {
                        if let Operand::Copy(place) = cond {
                            if let Some(Constant::Bool(b)) = map.get(place) {
                                *terminator = if *b {
                                    Terminator::Goto(*true_block)
                                } else {
                                    Terminator::Goto(*false_block)
                                };
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
