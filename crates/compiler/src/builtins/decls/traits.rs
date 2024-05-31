use crate::ast::BuiltinType;
use crate::Compiler;

mod add;
mod clone;
mod copy;
mod debug;
mod deep_clone;
mod display;
mod div;
mod eq;
mod into_iterator;
mod iterator;
mod mul;
mod neg;
mod not;
mod ord;
mod partial_eq;
mod partial_ord;
mod serde;
mod sub;

impl Compiler {
    pub(super) fn declare_traits(&mut self) {
        self.declare_add();
        self.declare_clone();
        self.declare_copy();
        self.declare_debug();
        self.declare_deep_clone();
        self.declare_display();
        self.declare_div();
        self.declare_eq();
        self.declare_into_iterator();
        self.declare_iterator();
        self.declare_mul();
        self.declare_neg();
        self.declare_not();
        self.declare_ord();
        self.declare_partial_eq();
        self.declare_partial_ord();
        self.declare_serde();
        self.declare_sub();
    }
}
