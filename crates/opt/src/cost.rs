use egg::CostFunction;
use egg::Language;

use crate::lang::AquaLang;

pub struct Inverter;

impl CostFunction<AquaLang> for Inverter {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &AquaLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        match enode {
            AquaLang::Symbol(s) if s.as_str() == "x" => 100,
            _ => enode.fold(1, |sum, id| sum.saturating_add(costs(id))),
        }
    }
}
