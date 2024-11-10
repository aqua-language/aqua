// var x = e0;
// var x = e0;
// for y in e0 e1
// =>
// var e0.fold(|x, record(y)| e1)
use crate::ast::Expr;
use crate::traversal::mapper::Mapper;

struct Context {}

impl Mapper for Context {
    fn map_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::For(s, t, x, e0, e1) => {
                
            }
            Expr::While(s, t, e0, e1) => {}
            _ => todo!()
        }
    }
}
