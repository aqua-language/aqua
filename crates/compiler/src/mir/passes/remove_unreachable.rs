use crate::mir;
use crate::mir::Terminator;

impl mir::Function {
    pub fn remove_unreachable(&mut self) {
        let mut visited = vec![false; self.blocks.len()];
        self.dfs(&mut visited, 0);

        let mut block_map = vec![0; self.blocks.len()];
        let mut new_blocks = Vec::with_capacity(visited.iter().filter(|&&v| v).count());

        for block in &self.blocks {
            if visited[block.id] {
                block_map[block.id] = new_blocks.len();
                new_blocks.push(block.clone());
            }
        }

        self.blocks = new_blocks;

        for block in &mut self.blocks {
            block.id = block_map[block.id];
            match &mut block.terminator {
                Some(Terminator::Goto(b)) => *b = block_map[*b],
                Some(Terminator::ConditionalGoto(_, b0, b1)) => {
                    *b0 = block_map[*b0];
                    *b1 = block_map[*b1];
                }
                _ => {}
            }
        }

        self.successors = vec![];
        self.predecessors = vec![];
    }

    fn dfs(&self, visited: &mut Vec<bool>, b: usize) {
        if visited[b] {
            return;
        }
        visited[b] = true;

        match &self.blocks[b].terminator {
            &Some(Terminator::Goto(b)) => {
                self.dfs(visited, b);
            }
            &Some(Terminator::ConditionalGoto(_, b0, b1)) => {
                self.dfs(visited, b0);
                self.dfs(visited, b1);
            }
            _ => {}
        }
    }

    pub fn with_remove_unreachable(mut self) -> Self {
        self.remove_unreachable();
        self
    }
}
