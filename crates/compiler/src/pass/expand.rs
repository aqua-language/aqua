use crate::analysis::declare;
use crate::ast::Program;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    decls: crate::analysis::declare::Context,
    fuel: u32,
    report: Report,
}

impl Context {
    pub fn new() -> Context {
        Context {
            decls: declare::Context::new(),
            report: Report::new(),
            fuel: 50,
        }
    }
}

impl Pass for Context {
    fn run(&mut self, program: &Program) -> Program {
        self.decls.visit_program(program);
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Mapper for Context {
    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Alias(x, ts) => {
                let stmt = self.decls.types.get(x).unwrap().instantiate(ts);
                let TypeBody::UserDefined(t1) = stmt.body else {
                    unreachable!()
                };
                if self.fuel > 0 {
                    self.fuel -= 1;
                    let t1 = self.map_type(&t1);
                    self.fuel += 1;
                    t1
                } else {
                    self.report.err(
                        x.span,
                        "Potential cycle detected when trying to expand type alias.",
                        "Type aliases can only be expanded up to 50 times.",
                    );
                    Type::Err
                }
            }
            _ => self._map_type(t),
        }
    }
}
