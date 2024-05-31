use std::rc::Rc;

use crate::lexer::Span;
use crate::symbol::Symbol;

use super::Arm;
use super::Block;
use super::Bound;
use super::Expr;
use super::Index;
use super::Map;
use super::Name;
use super::Pat;
use super::Path;
use super::Program;
use super::Segment;
use super::Stmt;
use super::StmtDef;
use super::StmtDefBody;
use super::StmtEnum;
use super::StmtImpl;
use super::StmtStruct;
use super::StmtTrait;
use super::StmtTraitDef;
use super::StmtTraitType;
use super::StmtType;
use super::StmtTypeBody;
use super::StmtVar;
use super::Type;

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Program {
        Program { stmts }
    }
}

impl StmtImpl {
    pub fn new(
        span: Span,
        generics: Vec<Name>,
        head: Bound,
        where_clause: Vec<Bound>,
        defs: Vec<Rc<StmtDef>>,
        types: Vec<Rc<StmtType>>,
    ) -> StmtImpl {
        StmtImpl {
            span,
            generics,
            where_clause,
            types,
            head,
            defs,
        }
    }
}

impl StmtType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, body: StmtTypeBody) -> StmtType {
        StmtType {
            span,
            name,
            generics,
            body,
        }
    }
}

impl StmtTrait {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        where_clause: Vec<Bound>,
        defs: Vec<Rc<StmtTraitDef>>,
        types: Vec<Rc<StmtTraitType>>,
    ) -> StmtTrait {
        StmtTrait {
            span,
            name,
            generics,
            where_clause,
            defs,
            types,
        }
    }
}

impl StmtTraitType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>) -> Self {
        Self {
            span,
            name,
            generics,
        }
    }
}

impl StmtVar {
    pub fn new(span: Span, name: Name, ty: Type, expr: Expr) -> StmtVar {
        StmtVar {
            span,
            name,
            ty,
            expr,
        }
    }
}

impl StmtDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        params: Map<Name, Type>,
        ty: Type,
        where_clause: Vec<Bound>,
        body: StmtDefBody,
    ) -> StmtDef {
        StmtDef {
            span,
            name,
            generics,
            params,
            ty,
            where_clause,
            body,
        }
    }
}

impl StmtStruct {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, fields: Map<Name, Type>) -> StmtStruct {
        StmtStruct {
            span,
            name,
            generics,
            fields,
        }
    }
}

impl StmtEnum {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, variants: Map<Name, Type>) -> StmtEnum {
        StmtEnum {
            span,
            name,
            generics,
            variants,
        }
    }
}

impl StmtTraitDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        params: Map<Name, Type>,
        ty: Type,
        where_clause: Vec<Bound>,
    ) -> Self {
        Self {
            span,
            name,
            generics,
            params,
            ty,
            where_clause,
        }
    }
}

impl Block {
    pub fn new(span: Span, stmts: Vec<Stmt>, expr: Expr) -> Block {
        Block {
            span,
            stmts,
            expr: Rc::new(expr),
        }
    }
}

impl Bound {
    pub fn span(&self) -> Span {
        match self {
            Bound::Path(s, ..) => *s,
            Bound::Trait(s, ..) => *s,
            Bound::Type(s, ..) => *s,
            Bound::Err(s) => *s,
        }
    }
    pub fn get_type(&self, x: &Name) -> Option<&Type> {
        match self {
            Bound::Path(_, _) => None,
            Bound::Trait(_, _, _, xts) => xts.get(x),
            Bound::Type(_, _) => None,
            Bound::Err(_) => None,
        }
    }
}

impl Arm {
    pub fn new(span: Span, p: Pat, e: Expr) -> Arm {
        Arm { span, p, e }
    }
}

impl Path {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
    pub fn new_name(name: Name) -> Self {
        Self::new(vec![Segment::new_name(name)])
    }
}

impl Segment {
    pub fn new(span: Span, name: Name, ts: Vec<Type>, xts: Map<Name, Type>) -> Self {
        Self {
            span,
            name,
            ts,
            xts,
        }
    }

    pub fn new_name(name: Name) -> Self {
        Self::new(name.span, name, Vec::new(), Map::new())
    }
}

impl Name {
    pub fn new(span: Span, data: impl Into<Symbol>) -> Name {
        Name {
            span,
            data: data.into(),
        }
    }
    pub fn suffix(self, suffix: impl std::fmt::Display) -> Name {
        Name::new(self.span, self.data.suffix(suffix))
    }
}

impl Index {
    pub fn new(span: Span, index: usize) -> Index {
        Index { span, data: index }
    }
}

