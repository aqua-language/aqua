use std::rc::Rc;

use crate::ast::Aggr;
use crate::ast::Ast;
use crate::ast::Block;
use crate::ast::Effect;
use crate::ast::Expr;
use crate::ast::ExprBody;
use crate::ast::Impl;
use crate::ast::Index;
use crate::ast::Local;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Path;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtLocal;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::report::span::Span;
use crate::report::symbol::Symbol;

impl Ast {
    pub fn new(span: Span, stmts: Vec<Stmt>) -> Ast {
        Ast { span, stmts }
    }
}

impl StmtImpl {
    pub fn new(
        span: Span,
        generics: Vec<Name>,
        head: Impl,
        where_clause: Vec<Impl>,
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

    pub fn new_simple(span: Span, head: Impl, defs: Vec<Rc<StmtDef>>) -> StmtImpl {
        StmtImpl {
            span,
            generics: Vec::new(),
            where_clause: Vec::new(),
            types: Vec::new(),
            head,
            defs,
        }
    }
}

impl StmtType {
    pub fn new(span: Span, name: Name, generics: Vec<Name>, body: TypeBody) -> StmtType {
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
        where_clause: Vec<Impl>,
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

    pub fn bound(&self) -> Impl {
        Impl::Trait(Trait::new(
            self.name,
            self.generics
                .iter()
                .map(|x| Type::Generic(*x))
                .collect::<Vec<_>>(),
        ))
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

impl StmtLocal {
    pub fn new(span: Span, local: Local, expr: Option<Expr>) -> StmtLocal {
        StmtLocal { span, local, expr }
    }
}

impl StmtDef {
    pub fn new(
        span: Span,
        name: Name,
        generics: Vec<Name>,
        params: Vec<Local>,
        ty: Type,
        effect: Effect,
        where_clause: Vec<Impl>,
        body: ExprBody,
    ) -> StmtDef {
        StmtDef {
            span,
            name,
            generics,
            params,
            ty,
            effect,
            where_clause,
            body,
        }
    }

    pub fn new_simple(span: Span, name: Name, params: Vec<Local>, ty: Type, expr: Expr) -> StmtDef {
        StmtDef {
            span,
            name,
            generics: Vec::new(),
            params,
            ty,
            effect: Effect::Unknown,
            where_clause: Vec::new(),
            body: ExprBody::UserDefined(Rc::new(expr)),
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
        params: Vec<Local>,
        ty: Type,
        effect: Effect,
        where_clause: Vec<Impl>,
    ) -> Self {
        Self {
            span,
            name,
            generics,
            params,
            ty,
            effect,
            where_clause,
        }
    }
}

impl Block {
    pub fn new(span: Span, stmts: Vec<Stmt>, expr: Option<Expr>) -> Block {
        Block { span, stmts, expr }
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
            x: name,
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
    pub fn with_suffix(self, suffix: impl std::fmt::Display) -> Name {
        Name::new(self.span, self.data.suffix(suffix))
    }
}

impl From<Symbol> for Name {
    fn from(data: Symbol) -> Name {
        Name {
            span: Span::Generated,
            data,
        }
    }
}

impl Index {
    pub fn new(span: Span, index: usize) -> Index {
        Index { span, data: index }
    }
}

impl Aggr {
    pub fn new(local: Local, name: Name, reduce_expr: Expr, filter_expr: Option<Expr>) -> Aggr {
        Aggr {
            local,
            name,
            reduce_expr: Rc::new(reduce_expr),
            filter_expr: filter_expr.map(Rc::new),
        }
    }
}

impl Trait {
    pub fn new(x: Name, ts: Vec<Type>) -> Trait {
        Trait { x, ts }
    }
}
