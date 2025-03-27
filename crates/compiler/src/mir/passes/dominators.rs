use crate::collections::set::Set;
use crate::mir::Function;
use crate::mir::Terminator;

impl Function {
    // Dom(x) = {x} ∪ (⋂{Dom(p) | p ∈ preds(x)})
    pub fn compute_dominators(&mut self) {
        let mut dom = vec![Set::new(); self.blocks.len()];
        dom[0].add(0);
        for b in 1..self.blocks.len() {
            for b_other in 0..self.blocks.len() {
                dom[b].add(b_other);
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
            for b in 1..self.blocks.len() {
                let mut new_dom = Set::new();
                let mut iter = self.predecessors[b].iter();
                if let Some(b_pred) = iter.next() {
                    new_dom = dom[*b_pred].clone();
                    for b_pred in iter {
                        new_dom = new_dom.intersection(&dom[*b_pred]);
                    }
                }
                new_dom.add(b);
                if new_dom != dom[b] {
                    dom[b] = new_dom;
                    changed = true;
                }
            }
        }
        for (b, dom) in dom.into_iter().enumerate() {
            self.blocks[b].dom = dom;
        }
        self.compute_dominator_tree();
    }

    fn compute_dominator_tree(&mut self) {
        let mut idom = vec![None; self.blocks.len()];
        idom[0] = None;

        for b1 in 1..self.blocks.len() {
            idom[b1] = self.blocks[b1]
                .dom
                .iter()
                .copied()
                .filter(|&b2| b1 != b2)
                .find(|b2| {
                    self.blocks[b1]
                        .dom
                        .iter()
                        .copied()
                        .filter(|&b3| b1 != b3 && *b2 != b3)
                        .all(|b3| !self.blocks[b3].dom.contains(&b2))
                });
        }

        let mut dominator_tree = vec![Vec::new(); self.blocks.len()];
        for b in 1..self.blocks.len() {
            if let Some(d) = idom[b] {
                dominator_tree[d].push(b);
            }
        }

        self.domtree = dominator_tree;
    }
    // b ↦ [b1, b2, ..., bn] if bi has a terminator that jumps to b.
    pub fn compute_predecessors(&mut self) {
        let mut preds = vec![Vec::new(); self.blocks.len()];
        for b0 in 0..self.blocks.len() {
            if let Some(t) = &self.blocks[b0].terminator {
                match t {
                    Terminator::Goto(b1) => {
                        preds[*b1].push(b0);
                    }
                    Terminator::IfElse(_, b1, b2) => {
                        preds[*b1].push(b0);
                        preds[*b2].push(b0);
                    }
                    _ => {}
                }
            }
        }
        self.predecessors = preds;
    }

    // b ↦ [b1, b2, ..., bn] if b has a terminator that jumps to bi.
    pub fn compute_successors(&mut self) {
        let mut succs = vec![Vec::new(); self.blocks.len()];
        for b0 in 0..self.blocks.len() {
            if let Some(t) = &self.blocks[b0].terminator {
                match t {
                    Terminator::Goto(b1) => {
                        succs[b0].push(*b1);
                    }
                    Terminator::IfElse(_, b1, b2) => {
                        succs[b0].push(*b1);
                        succs[b0].push(*b2);
                    }
                    _ => {}
                }
            }
        }
        self.successors = succs;
    }

    pub fn compute_postorder(&mut self) {
        let mut visited = vec![false; self.blocks.len()];
        let mut postorder = Vec::with_capacity(self.blocks.len());

        fn dfs(
            b: usize,
            visited: &mut [bool],
            postorder: &mut Vec<usize>,
            successors: &[Vec<usize>],
        ) {
            visited[b] = true;
            for &s in &successors[b] {
                if !visited[s] {
                    dfs(s, visited, postorder, successors);
                }
            }
            postorder.push(b);
        }

        dfs(0, &mut visited, &mut postorder, &self.successors);

        self.postorder = postorder;
    }

    pub fn compute_reverse_postorder_number(&mut self) {
        let mut reverse_postorder_number = vec![0; self.blocks.len()];
        for (i, &b) in self.postorder.iter().rev().enumerate() {
            reverse_postorder_number[b] = i;
        }
        self.reverse_postorder_number = reverse_postorder_number;
    }

    pub fn compute_preorder(&mut self) {
        let mut visited = vec![false; self.blocks.len()];
        let mut preorder = Vec::with_capacity(self.blocks.len());

        fn dfs(
            b: usize,
            visited: &mut [bool],
            preorder: &mut Vec<usize>,
            successors: &[Vec<usize>],
        ) {
            visited[b] = true;
            preorder.push(b);
            for &s in &successors[b] {
                if !visited[s] {
                    dfs(s, visited, preorder, successors);
                }
            }
        }

        dfs(0, &mut visited, &mut preorder, &self.successors);

        self.preorder = preorder;
    }
}
