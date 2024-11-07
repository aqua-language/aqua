use crate::ast::Block;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtStruct;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::builtins::value::Fun;
use crate::print::Print;

pub trait Codegen<'a>: Print<'a> {
    fn program(&mut self, p: &Program) -> std::fmt::Result;

    fn param(&mut self, xt: &(Name, Type)) -> std::fmt::Result;

    fn stmt(&mut self, s: &Stmt) -> std::fmt::Result;

    fn stmt_var(&mut self, s: &StmtVar) -> std::fmt::Result;

    fn stmt_def(&mut self, s: &StmtDef) -> std::fmt::Result;

    fn stmt_expr(&mut self, s: &Expr) -> std::fmt::Result;

    fn stmt_struct(&mut self, s: &StmtStruct) -> std::fmt::Result;

    fn stmt_enum(&mut self, s: &StmtEnum) -> std::fmt::Result;

    fn stmt_type(&mut self, s: &StmtType) -> std::fmt::Result;

    fn expr(&mut self, e: &Expr) -> std::fmt::Result;

    fn block(&mut self, b: &Block) -> std::fmt::Result;

    fn expr_field(&mut self, xt: &(Name, Expr)) -> std::fmt::Result;

    fn type_field(&mut self, xt: &(Name, Type)) -> std::fmt::Result;

    fn ty(&mut self, t: &Type) -> std::fmt::Result;

    fn fun(&mut self, f: &Fun) -> std::fmt::Result;

    fn fields<T>(
        &mut self,
        items: &[T],
        f: impl Fn(&mut Self, &T) -> std::fmt::Result,
    ) -> std::fmt::Result;
}
