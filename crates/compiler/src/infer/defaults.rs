use crate::ast::Program;
use crate::ast::StmtDef;
use crate::ast::Trait;
use crate::ast::Type;
use crate::infer::default_float;
use crate::infer::default_int;
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
                if let TypeVarValue::Unknown(k) = self.0.get_value(*x) {
                    match k {
                        TypeVarKind::Int => self.0.union_value(*x, default_int()),
                        TypeVarKind::Float => self.0.union_value(*x, default_float()),
                        _ => self._visit_type(ty),
                    }
                }
            }
            _ => self._visit_type(ty),
        }
    }
}

impl Trait {
    /// Set default values for all type variables in a trait.
    pub fn defaults(&self, ctx: &mut Context) {
        Defaults::new(ctx).visit_bound(self);
    }
}

impl Program {
    /// Set default values for all type variables in a program.
    pub fn defaults(&self, ctx: &mut Context) {
        Defaults::new(ctx).visit_program(self);
    }
}

impl StmtDef {
    /// Set default values for all type variables in a statement definition.
    pub fn defaults(&self, ctx: &mut Context) {
        Defaults::new(ctx).visit_stmt_def(self);
    }
}
