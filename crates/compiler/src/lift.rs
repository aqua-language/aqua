use crate::ast::Expr;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtVar;

struct Lift {}

struct Stack(Vec<Scope>);

impl Stack {
    fn new() -> Stack {
        Stack(vec![Scope::new()])
    }
}

struct Scope(Vec<Binding>);

impl Scope {
    fn new() -> Scope {
        Scope(vec![])
    }
}

enum Binding {}

impl Lift {
    fn new() -> Lift {
        Lift {}
    }

    fn lift(&mut self, program: &Program) -> Program {
        let stmts = program.stmts.iter().map(|stmt| self.stmt(stmt)).collect();
        Program::new(stmts)
    }

    fn stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::Var(s) => self.stmt_var(s),
            Stmt::Def(s) => self.stmt_def(s),
            Stmt::Trait(s) => self.stmt_trait(s),
            Stmt::Impl(s) => self.stmt_impl(s),
            Stmt::Struct(s) => self.stmt_struct(s),
            Stmt::Enum(s) => self.stmt_enum(s),
            Stmt::Type(s) => self.stmt_type(s),
            Stmt::Expr(s) => self.stmt_expr(s),
            Stmt::Err(_) => todo!(),
        }
    }

    fn stmt_var(&mut self, _: &StmtVar) -> Stmt {
        todo!()
    }

    fn stmt_def(&mut self, _: &StmtDef) -> Stmt {
        todo!()
    }

    fn stmt_trait(&self, _: &StmtTrait) -> Stmt {
        todo!()
    }

    fn stmt_impl(&self, _: &StmtImpl) -> Stmt {
        todo!()
    }

    fn stmt_struct(&self, _: &StmtStruct) -> Stmt {
        todo!()
    }

    fn stmt_enum(&self, _: &StmtEnum) -> Stmt {
        todo!()
    }

    fn stmt_type(&self, _: &StmtType) -> Stmt {
        todo!()
    }

    fn stmt_expr(&self, _: &Expr) -> Stmt {
        todo!()
    }
}
