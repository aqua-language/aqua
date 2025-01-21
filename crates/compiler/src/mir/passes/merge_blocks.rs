use crate::mir::Function;
use crate::mir::Terminator;

impl Function {
    pub fn merge_blocks(&mut self) {
        let mut visited = vec![false; self.blocks.len()];

        for i in 0..self.blocks.len() {
            if visited[i] {
                continue;
            }

            let mut chain = Vec::new();
            let mut pred = i;

            while let &Some(Terminator::Goto(succ)) = &self.blocks[pred].terminator {
                if self.predecessors[succ].len() == 1 && !visited[succ] {
                    chain.push(pred);
                    visited[pred] = true;
                    pred = succ;
                } else {
                    break;
                }
            }

            if !chain.is_empty() {
                chain.push(pred);
                visited[pred] = true;

                // Merge the chain into the first block
                let &first = chain.first().unwrap();
                let &second = chain.get(1).unwrap();
                let &last = chain.last().unwrap();
                self.successors[first] = std::mem::take(&mut self.successors[last]);
                self.blocks[first].terminator = std::mem::take(&mut self.blocks[last].terminator);
                self.predecessors[second].clear();

                for &succ in chain.iter().skip(1) {
                    let stmts = std::mem::take(&mut self.blocks[succ].stmts);
                    self.blocks[first].stmts.extend(stmts);
                }
            }
        }
    }

    pub fn with_merge_blocks(mut self) -> Self {
        self.merge_blocks();
        self
    }
}
