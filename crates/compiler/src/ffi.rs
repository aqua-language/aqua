use crate::ast;
use libffi::middle::Type;

pub struct Ffi {
    stack: Stack,
}

impl Default for Ffi {
    fn default() -> Self {
        Self::new()
    }
}

struct Stack(Vec<Scope>);

impl Stack {
    fn new() -> Self {
        Self(vec![Scope::new()])
    }
}

struct Scope(Vec<Binding>);

impl Scope {
    fn new() -> Self {
        Self(Vec::new())
    }
}

enum Binding {
    Cons(Type),
}

impl Ffi {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn ty(&mut self, t: ast::Type) -> Type {
        match t {
            ast::Type::Unresolved(_) => unreachable!(),
            ast::Type::Cons(_, _) => todo!(),
            ast::Type::Alias(_, _) => unreachable!(),
            ast::Type::Assoc(_, _, _, _) => unreachable!(),
            ast::Type::Var(_) => unreachable!(),
            ast::Type::Generic(_) => unreachable!(),
            ast::Type::Fun(_, _) => todo!(),
            ast::Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| self.ty(t.clone())).collect::<Vec<_>>();
                Type::structure(ts)
            }
            ast::Type::Record(xts) => {
                let xts = xts
                    .iter()
                    .map(|(_, t)| self.ty(t.clone()))
                    .collect::<Vec<_>>();
                Type::structure(xts)
            }
            ast::Type::Hole => unreachable!(),
            ast::Type::Err => unreachable!(),
            ast::Type::Array(_, _) => todo!(),
            ast::Type::Never => unreachable!(),
        }
    }
}
