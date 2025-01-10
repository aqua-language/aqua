use crate::analysis::declare;
use crate::ast::Ast;
use crate::ast::Stmt;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::diag::Diagnostic;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    decls: crate::analysis::declare::Context,
    depth: u32,
    report: Report,
}

impl Context {
    pub fn new() -> Context {
        Context {
            decls: declare::Context::new(),
            report: Report::new(),
            depth: 0,
        }
    }
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
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
                if self.depth < 50 {
                    self.depth += 1;
                    let t1 = self.map_type(&t1);
                    self.depth -= 1;
                    t1
                } else {
                    self.report.add(Diagnostic::err2(
                        x.span,
                        stmt.span,
                        "Potential cycle detected when trying to expand type alias.",
                        "Type aliases can only be nested up to 50 times.",
                        "Tried to expand this type alias.",
                    ));
                    Type::Err
                }
            }
            _ => self._map_type(t),
        }
    }

    // Filter out type aliases
    fn map_program(&mut self, program: &Ast) -> Ast {
        let stmts = program
            .stmts
            .iter()
            .map(|stmt| self.map_stmt(stmt))
            .filter(|s| !matches!(s, Stmt::Type(s) if matches!(s.body, TypeBody::UserDefined(_))))
            .collect();
        Ast::new(program.span, stmts)
    }
}
