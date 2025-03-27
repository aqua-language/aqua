use crate::collections::set::Set;
use crate::mir::BasicBlock;
use crate::mir::BlockId;
use crate::mir::Function;
use crate::mir::Local;
use crate::mir::Name;
use crate::mir::Operation;
use crate::mir::Stmt;
use crate::mir::Terminator;
use crate::mir::Type;
use crate::report::span::Span;

impl Stmt {
    pub fn new(op: Operation) -> Stmt {
        Stmt {
            op,
            live_in: Set::new(),
            live_out: Set::new(),
        }
    }
}

impl Function {
    pub fn new(
        span: Span,
        name: Name,
        params: Vec<Local>,
        locals: Vec<Local>,
        ty: Type,
        blocks: Vec<BasicBlock>,
    ) -> Self {
        Self {
            span,
            name,
            params,
            locals,
            ty,
            blocks,
            domtree: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
            postorder: Vec::new(),
            preorder: Vec::new(),
            reverse_postorder_number: Vec::new(),
        }
    }
}

impl BasicBlock {
    pub fn new(
        id: BlockId,
        stmts: Vec<Stmt>,
        terminator: Option<Terminator>,
    ) -> Self {
        Self {
            id,
            stmts,
            terminator,
            live_in: Set::new(),
            live_out: Set::new(),
            dom: Set::new(),
        }
    }
}
