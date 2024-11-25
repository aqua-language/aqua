use std::fmt::Write;

use crate::ast::Name;
use crate::ast::Trait;
use crate::ast::Type;

pub(crate) struct Mangler(String);

impl Mangler {
    fn new() -> Mangler {
        Mangler(String::new())
    }

    fn finish(self) -> Name {
        self.0.into()
    }

    pub(crate) fn mangle_fun(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type(t));
        m.finish()
    }

    pub(crate) fn mangle_record(xts: &[(Name, Type)]) -> Name {
        let mut m = Mangler::new();
        m.write("Record");
        xts.into_iter().for_each(|(x, t)| {
            m.write(x);
            m.mangle_type(t);
        });
        m.write("End");
        m.finish()
    }

    pub(crate) fn mangle_struct(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type(t));
        m.finish()
    }

    pub(crate) fn mangle_enum(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type(t));
        m.finish()
    }

    pub(crate) fn mangle_trait_impl_def(tr: &Trait, x: &Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write(tr.x);
        tr.ts.iter().for_each(|t| m.mangle_type(t));
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type(t));
        m.finish()
    }

    pub(crate) fn mangle_type_impl_def(t: &Type, x: &Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.mangle_type(t);
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type(t));
        m.finish()
    }

    fn write(&mut self, s: impl std::fmt::Display) {
        write!(&mut self.0, "{s}").expect("Mangling should not fail");
    }

    fn mangle_type(&mut self, t: &Type) {
        match t {
            Type::Path(_) => unreachable!(),
            Type::Builtin(x, ts) => {
                self.write(x);
                ts.into_iter().for_each(|t| self.mangle_type(t));
            }
            Type::Assoc(_, _, _) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Generic(_) => unreachable!(),
            Type::Lambda(ts, t) => {
                self.write("Fun");
                ts.into_iter().for_each(|t| self.mangle_type(t));
                self.mangle_type(t);
                self.write("End");
            }
            Type::Tuple(ts) => {
                self.write("Tuple");
                ts.into_iter().for_each(|t| self.mangle_type(t));
                self.write("End");
            }
            Type::Record(xts) => {
                self.write("Record");
                xts.into_iter().for_each(|(x, t)| {
                    self.write(x);
                    self.mangle_type(t);
                });
                self.write("End");
            }
            Type::Array(t, _i) => {
                self.write("Array");
                self.mangle_type(t);
                self.write("End");
            }
            Type::Struct(_, _) => todo!(),
            Type::Enum(_, _) => todo!(),
            Type::Never => unreachable!(),
            Type::Paren(_) => unreachable!(),
            Type::Err => unreachable!(),
            Type::Unknown => unreachable!(),
            Type::Alias(..) => unreachable!(),
            Type::Ref(_, _) => todo!(),
            Type::RefMut(_, _) => todo!(),
        }
    }
}
