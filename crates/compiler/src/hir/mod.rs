use std::rc::Rc;

use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::collections::ordmap::OrdMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hir {
    pub stmts: Vec<Stmt>,
    pub defs: OrdMap<Name, Rc<StmtDef>>,
    pub structs: OrdMap<Name, Rc<StmtStruct>>,
    pub enums: OrdMap<Name, Rc<StmtEnum>>,
    pub traits: OrdMap<Name, Rc<StmtTrait>>,
    pub types: OrdMap<Name, Rc<StmtType>>,
    pub trait_impls: OrdMap<Name, Vec<Rc<StmtImpl>>>,
    pub type_impls: Vec<Rc<StmtImpl>>,

    pub trait_methods: OrdMap<Name, Vec<Rc<StmtTrait>>>,
    pub type_methods: OrdMap<Name, Vec<Rc<StmtTrait>>>,
}
