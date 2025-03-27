use runtime::HashMap;

pub use crate::ast::Local;
pub use crate::ast::Name;
pub use crate::ast::Place;
pub use crate::ast::Type;
pub use crate::ast::Index;
pub use crate::ast::PlaceElem;
use crate::collections::set::Set;
use crate::report::span::Span;
use crate::report::symbol::Symbol;

pub mod display;
pub mod passes;
pub mod utils;

pub type BlockId = usize;

#[derive(Debug, Clone)]
pub struct Mir {
    pub enums: HashMap<Name, Type>,
    pub structs: HashMap<Name, Type>,
    pub locals: Vec<Local>,
    pub blocks: Vec<BasicBlock>,
    pub functions: Vec<Function>,
}

impl Mir {
    pub fn new(locals: Vec<Local>, blocks: Vec<BasicBlock>, functions: Vec<Function>) -> Mir {
        Mir {
            enums: HashMap::default(),
            structs: HashMap::default(),
            locals,
            blocks,
            functions,
        }
    }
}

/// A MIR function. The control-flow in a MIR function is represented by a control-flow graph.
#[derive(Debug, Clone)]
pub struct Function {
    pub span: Span,
    pub name: Name,
    pub params: Vec<Local>,
    pub locals: Vec<Local>,
    pub ty: Type,
    pub blocks: Vec<BasicBlock>,
    // Analysis data
    /// Tree of dominators.
    /// * A block `a` dominates a block `b` if all paths to `b` must pass through `a.`
    pub domtree: Vec<Vec<BlockId>>,
    /// Adjacency list of successors.
    /// * A block `a` is a successor of `b` if `b` has a terminator that is a goto to `a`.
    pub successors: Vec<Vec<BlockId>>,
    /// Adjacency list of successors.
    /// * A block `a` is a predecessor of `b` if `a` has a terminator that is a goto to `b`.
    pub predecessors: Vec<Vec<BlockId>>,
    /// Post-order numbering.
    /// * The post-order numbering of a block is the order in which the blocks are visited in a
    ///  depth-first search of the control-flow graph.
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

#[derive(Debug, Clone)]
pub enum Operation {
    Assign(Place, Rvalue),
    // Marks a place as live. This is necessary since the MIR can contain mutable variables.
    // With only Assign, we cannot distinguish between a place that is initialized and a place that
    // is mutated.
    Live(Local),
    // Marks a place as dead. This is necessary since the MIR must know when variables go out of
    // scope.
    Dead(Local),
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
    IfElse(Operand, BlockId, BlockId),
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
