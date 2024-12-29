use crate::collections::keyvec::Key;

use super::Local;
use super::Name;
use super::StmtDef;
use super::StmtEnum;
use super::StmtStruct;
use super::StmtTrait;
use super::StmtTraitDef;
use super::StmtTraitType;
use super::StmtType;
use super::StmtVar;

impl Key for StmtDef {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
    }
}

impl Key for StmtVar {
    type K = Name;
    fn key(&self) -> &Self::K {
        &self.name
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
