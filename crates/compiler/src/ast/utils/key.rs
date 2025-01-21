use crate::collections::keyvec::Key;

use crate::ast::Local;
use crate::ast::Name;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::StmtLocal;

impl Key for StmtDef {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtLocal {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.local.name
    }
}

impl Key for StmtEnum {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtType {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtTrait {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtStruct {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtTraitDef {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtTraitType {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for Local {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}
