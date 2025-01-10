use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Place;
use crate::ast::Type;
use crate::collections::set::Set;
use crate::syntax::symbol::Symbol;

pub type BlockId = usize;

#[derive(Debug, Clone)]
pub struct Mir {
    pub locals: Vec<Local>,
    pub blocks: Vec<BasicBlock>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Name,
    pub params: Vec<Local>,
    pub locals: Vec<Local>,
    pub ty: Type,
    pub blocks: Vec<BasicBlock>,
    // Analysis data
    pub domtree: Vec<Vec<BlockId>>,
    pub successors: Vec<Vec<BlockId>>,
    pub predecessors: Vec<Vec<BlockId>>,
    pub postorder: Vec<BlockId>,
    pub preorder: Vec<BlockId>,
    pub reverse_postorder_number: Vec<BlockId>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Terminator>,
    pub live_in: Set<Place>,
    pub live_out: Set<Place>,
    pub dom: Set<BlockId>,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub op: Operation,
    pub live_in: Set<Place>,
    pub live_out: Set<Place>,
}

impl Stmt {
    pub fn new(op: Operation) -> Stmt {
        Stmt {
            op,
            live_in: Set::new(),
            live_out: Set::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Assign(Place, Rvalue),
    // Marks a place as live. This is necessary since the MIR can contain mutable variables.
    // With only Assign, we cannot distinguish between a place that is initialized and a place that
    // is mutated.
    StorageLive(Local),
    // Marks a place as dead. This is necessary since the MIR must know when variables go out of
    // scope.
    StorageDead(Local),
    Call {
        dest: Place,
        func: Operand,
        args: Vec<Operand>,
    },
    Noop,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return,
    Goto(BlockId),
    ConditionalGoto(Operand, BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    Ref { mutable: bool, place: Place },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Constant(Constant),
    Copy(Place),
    Move(Place),
    Function(Name, Vec<Type>),
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(Symbol),
    Bool(bool),
    String(Symbol),
    Float(Symbol),
    Char(char),
    Unit,
}
