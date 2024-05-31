// use runtime::builtins::cell::Cell;

use crate::Compiler;

// pub(crate) enum Iter {
//     FromVec(Vec<crate::builtins::Value>),
//     Map(Cell<Iter>),
//     Filter(Cell<Iter>),
//     Enumerate(Cell<Iter>),
// }

impl Compiler {
    pub(super) fn declare_iterator(&mut self) {}
}

//     .t("Iter", ["T"], [rust("Iter")])
//     .f("map", ["T", "U"], [fun([t("T")], t("U"))], iter(), [rust("Iter::map")])
//     .f("filter", ["T"], [fun([t("T")], bool())], iter(), [rust("Iter::filter")])
//     .f("enumerate", ["T"], [], iter(), [rust("Iter::enumerate")])
//     .f("collect_vec", ["T"], [], vec(t("T")), [rust("Iter::collect_vec")]);
