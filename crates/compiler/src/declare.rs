use std::rc::Rc;

use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::Trait;
use crate::traversal::visitor::Visitor;

#[derive(Debug, Default)]
pub struct Context {
    pub defs: Map<Name, Rc<StmtDef>>,
    pub structs: Map<Name, Rc<StmtStruct>>,
    pub enums: Map<Name, Rc<StmtEnum>>,
    pub traits: Map<Name, Rc<StmtTrait>>,
    pub types: Map<Name, Rc<StmtType>>,
    pub trait_impls: Map<Name, Vec<Rc<StmtImpl>>>,
}

impl Visitor for Context {
    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.defs.insert(s.name, s.clone());
            }
            Stmt::Impl(s) => match s.head {
                Trait::Cons(x, _, _) => {
                    self.trait_impls
                        .entry(x)
                        .or_insert_with(Vec::new)
                        .push(s.clone());
                }
                Trait::Type(_) => {}
                Trait::Path(_, _) => {}
                Trait::Err => {}
                Trait::Var(_) => {}
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
            Stmt::Err(_) => todo!(),
        }
    }
}
