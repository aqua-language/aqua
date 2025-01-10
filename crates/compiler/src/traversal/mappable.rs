use crate::ast::passes::infer::solver::Constraint;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::QueryOp;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::Type;

use super::mapper::Mapper;

pub(crate) trait Mappable {
    fn map(&self, mapper: &mut impl Mapper) -> Self;
}

impl Mappable for Ast {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_program(self)
    }
}

impl Mappable for Stmt {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_stmt(self)
    }
}

impl Mappable for Expr {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_expr(self)
    }
}

impl Mappable for Path {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_path(self)
    }
}

impl Mappable for Segment {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_segment(self)
    }
}

impl Mappable for Name {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_name(self)
    }
}

impl Mappable for Type {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_type(self)
    }
}

impl Mappable for Pat {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_pattern(self)
    }
}

impl Mappable for QueryOp {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_query_stmt(self)
    }
}

impl Mappable for StmtDef {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_stmt_def(self)
    }
}

impl Mappable for ExprBody {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_stmt_def_body(self)
    }
}

impl Mappable for Impl {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_impl(self)
    }
}

impl Mappable for Vec<Stmt> {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_stmts(self)
    }
}

impl Mappable for Constraint {
    fn map(&self, mapper: &mut impl Mapper) -> Self {
        mapper.map_constraint(self)
    }
}
