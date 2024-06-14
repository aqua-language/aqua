
// use smol_str::format_smolstr;
//
// use crate::ast::Expr;
// use crate::ast::Name;
// use crate::traversal::mapper::Mapper;
//
// pub struct Context {
//     uid_counter: u32,
// }
//
// impl Context {
//     pub fn new() -> Self {
//         Context { uid_counter: 0 }
//     }
//
//     fn fresh_name(&mut self) -> Name {
//         self.uid_counter += 1;
//     }
// }
//
// impl Mapper for Context {
//     fn map_expr(&mut self, expr: &Expr) -> Expr {
//         expr
//     }
// }
