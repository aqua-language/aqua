#![allow(unused)]
use std::collections::HashSet;

use egg::merge_option;
use egg::Analysis;
use egg::DidMerge;
use egg::Id;
use egg::PatternAst;

use crate::lang::AquaLang;

pub struct PartialEval;

#[derive(Debug)]
pub struct Data {
    free: HashSet<Id>,
    constant: Option<(AquaLang, PatternAst<AquaLang>)>,
}

// impl Analysis<AquaLang> for PartialEval {
//     type Data = Data;
//
//     fn make(egraph: &egg::EGraph<AquaLang, Self>, enode: &AquaLang) -> Self::Data {
//         let f = |i: &Id| egraph[*i].data.free.iter().cloned();
//         let mut free = HashSet::default();
//         match enode {
//             AquaLang::Var(v) => {
//                 free.insert(*v);
//             }
//             AquaLang::Let([v, a, b]) => {
//                 free.extend(f(b));
//                 free.remove(v);
//                 free.extend(f(a));
//             }
//             AquaLang::Lambda([v, a]) | AquaLang::Fix([v, a]) => {
//                 free.extend(f(a));
//                 free.remove(v);
//             }
//             _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
//         }
//         let constant = eval(egraph, enode);
//         Data { constant, free }
//     }
//
//     fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> egg::DidMerge {
//         let before_len = to.free.len();
//         // to.free.extend(from.free);
//         to.free.retain(|i| from.free.contains(i));
//         // compare lengths to see if I changed to or from
//         DidMerge(
//             before_len != to.free.len(),
//             to.free.len() != from.free.len(),
//         ) | merge_option(&mut to.constant, from.constant, |a, b| {
//             assert_eq!(a.0, b.0, "Merged non-equal constants");
//             DidMerge(false, false)
//         })
//     }
// }
