use crate::ast::Type;
use crate::infer::float_default;
use crate::infer::int_default;
use crate::infer::type_var::TypeVarKind;
use crate::infer::type_var::TypeVarValue;
use crate::infer::Context;
use crate::traversal::visitor::Visitor;

#[derive(Debug)]
pub struct Defaults<'a>(&'a mut Context);

impl Defaults<'_> {
    pub fn new<'a>(ctx: &'a mut Context) -> Defaults<'a> {
        Defaults(ctx)
    }
}

impl<'a> Visitor for Defaults<'a> {
    fn visit_type(&mut self, ty: &Type) {
        match ty {
            Type::Var(x) => {
                if let TypeVarValue::Unknown(k) = self.0.get_type(*x) {
                    match k {
                        TypeVarKind::Int => self.0.union_value(*x, int_default()),
                        TypeVarKind::Float => self.0.union_value(*x, float_default()),
                        _ => self._visit_type(ty),
                    }
                }
            }
            _ => self._visit_type(ty),
        }
    }
}
