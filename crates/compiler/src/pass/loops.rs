// var x = e0;
// var x = e0;
// for y in e0 e1
// =>
// var e0.fold(|x, record(y)| e1)
use crate::ast::Expr;
use crate::ast::Program;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;

use super::Pass;

#[derive(Debug)]
pub struct Context {
    _report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Program) -> Program {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut crate::diag::Report {
        unimplemented!()
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::For(_s, _t, _x, _e0, _e1) => {
                todo!()
            }
            Expr::While(_s, _t, _e0, _e1) => {
                todo!()
            }
            _ => todo!(),
        }
    }
}
