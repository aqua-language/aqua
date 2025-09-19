use std::rc::Rc;

use runtime::prelude::Send;
use runtime::prelude::Sync;

use crate::ast::Impl;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::collections::ordmap::OrdMap;
use crate::traversal::visitor::Visitor;

#[derive(Debug, Default, Clone, Send, Sync)]
pub struct Context {
    pub defs: OrdMap<Name, Rc<StmtDef>>,
    pub structs: OrdMap<Name, Rc<StmtStruct>>,
    pub enums: OrdMap<Name, Rc<StmtEnum>>,
    pub traits: OrdMap<Name, Rc<StmtTrait>>,
    pub types: OrdMap<Name, Rc<StmtType>>,
    pub trait_impls: OrdMap<Name, Vec<Rc<StmtImpl>>>,
    pub type_impls: Vec<Rc<StmtImpl>>,
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }
}

impl Visitor for Context {
    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Local(_) => {}
            Stmt::Def(s) => {
                self.defs.insert(s.name, s.clone());
            }
            Stmt::Impl(s) => match &s.head {
                Impl::Trait(tr) => {
                    self.trait_impls
                        .entry(tr.x)
                        .or_insert_with(Vec::new)
                        .push(s.clone());
                }
                Impl::Type(_) => {
                    self.type_impls.push(s.clone());
                }
                Impl::Path(..) => {}
                Impl::Err => {}
                Impl::Var(..) => {}
                Impl::Unknown => {}
            },
            Stmt::Expr(_) => {}
            Stmt::Struct(s) => {
                self.structs.insert(s.name, s.clone());
            }
            Stmt::Enum(s) => {
                self.enums.insert(s.name, s.clone());
            }
            Stmt::Type(s) => {
                self.types.insert(s.name, s.clone());
            }
            Stmt::Trait(s) => {
                self.traits.insert(s.name, s.clone());
            }
            Stmt::Err(_) => {}
        }
    }
}
