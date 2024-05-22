use std::rc::Rc;

use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::Program;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::TraitBound;
use crate::ast::Type;
use crate::ast::UnresolvedPatField;
use crate::diag::Report;
use crate::lexer::Span;
use crate::map::Map;

#[derive(Debug)]
pub struct Stack(Vec<Scope>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Map<Name, Binding>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Binding {
    Enum(Rc<StmtEnum>),
    Struct(Rc<StmtStruct>),
    Type(Rc<StmtType>),
    Trait(Rc<StmtTrait>),
    Def(Rc<StmtDef>),
    Var,
    Generic,
}

impl Stack {
    fn bind(&mut self, name: Name, binding: Binding) {
        self.0.last_mut().unwrap().0.insert(name, binding);
    }

    fn get(&self, x: &Name) -> Option<Binding> {
        self.0.iter().rev().find_map(|s| s.0.get(x)).cloned()
    }

    fn traits(&self) -> impl Iterator<Item = &StmtTrait> {
        self.0
            .iter()
            .rev()
            .flat_map(|s| s.0.iter())
            .filter_map(|(_, b)| match b {
                Binding::Trait(s) => Some(s.as_ref()),
                _ => None,
            })
    }
}

#[derive(Debug)]
pub struct Context {
    stack: Stack,
    pub report: Report,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack: Stack(vec![Scope(Map::new())]),
            report: Report::new(),
        }
    }

    pub fn resolve(&mut self, p: &Program) -> Program {
        p.stmts.iter().for_each(|stmt| self.declare_stmt(stmt));
        let stmts = p.stmts.iter().map(|stmt| self.stmt(stmt)).collect();
        Program::new(stmts)
    }

    fn scoped<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.stack.0.push(Scope(Map::new()));
        let t = f(self);
        self.stack.0.pop();
        t
    }

    fn not_found(&mut self, name: &Name, expected: &'static str) {
        self.report.err(
            name.span,
            format!("Name `{}` not found.", name),
            format!("Expected {}.", expected),
        );
    }

    fn unexpected(&mut self, name: &Name, found: &'static str, expected: &'static str) {
        self.report.err(
            name.span,
            format!("Unexpected {found} `{name}`."),
            format!("Expected {expected}."),
        );
    }

    #[track_caller]
    fn wrong_arity(&mut self, name: &Name, found: usize, expected: usize) {
        #[cfg(not(feature = "explicit"))]
        let file = "";
        #[cfg(feature = "explicit")]
        let file = format!("{}: ", std::panic::Location::caller());
        self.report.err(
            name.span,
            format!("{file}Wrong number of type arguments. Found {found}, expected {expected}",),
            format!("Expected {} arguments.", expected),
        );
    }

    fn expected_assoc(&mut self, kind: &'static str, x: &Name) {
        self.report.err(
            x.span,
            format!("Expected an associated {kind} `{x}::<{kind}>`.",),
            format!("Expected an associated {kind}."),
        );
    }

    fn unexpected_assoc(&mut self, kind0: &'static str, kind1: &'static str, x0: &Name, x1: &Name) {
        self.report.err(
            x1.span,
            format!("Found unexpected associated {kind1} `{x0}::{x1}`.",),
            format!("{kind0} `{x0}` has no associated {kind1} `{x1}`.",),
        );
    }

    fn unexpected_named_type_args(&mut self, x: &Name) {
        self.report.err(
            x.span,
            format!("Unexpected named type arguments for `{x}`.",),
            "Named type arguments can only occur in trait bounds.",
        );
    }

    fn wrong_fields<A, B>(
        &mut self,
        name: &Name,
        found: Option<&Map<Name, A>>,
        expected: &Map<Name, B>,
    ) {
        let expected = comma_sep(expected.keys());
        if let Some(found) = found {
            let found = comma_sep(found.keys());
            self.report.err(
                name.span,
                format!(
                    "Wrong fields provided. Found {name}({found}), expected {name}({expected})",
                ),
                format!("Expected {name}({expected}) fields."),
            );
        } else {
            self.report.err(
                name.span,
                format!("Wrong fields provided. Found {name}, expected {name}({expected})",),
                format!("Expected {name}({expected}) fields."),
            );
        }
    }

    fn wrong_items<'a>(
        &mut self,
        kind: &'static str,
        name: &Name,
        found: impl IntoIterator<Item = &'a Name>,
        expected: impl IntoIterator<Item = &'a Name>,
    ) {
        let found = comma_sep(found);
        let expected = comma_sep(expected);
        self.report.err(
            name.span,
            format!("Wrong {kind}s implemented for {name}. Found {{ {found} }}, expected {{ {expected} }}",),
            format!("Expected {{ {expected} }}."),
        );
    }

    #[allow(dead_code)]
    fn wrong_variant<T>(&mut self, name: &Name, found: &(Name, T), expected: &[Name]) {
        let found = &found.0;
        let expected = comma_sep(expected.iter());
        self.report.err(
            name.span,
            format!("Wrong variant provided. Found {found}, expected {expected}",),
            format!("Expected one of {{ {expected} }} variants."),
        );
    }

    fn expected_name(&mut self, e: &Expr) {
        self.report.err(
            e.span(),
            "Expected a field label.",
            "Only `<name> = <expr>` is allowed.",
        );
    }

    fn declare_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => self.stack.bind(s.name, Binding::Def(s.clone())),
            Stmt::Trait(s) => self.stack.bind(s.name, Binding::Trait(s.clone())),
            Stmt::Impl(_) => {}
            Stmt::Struct(s) => self.stack.bind(s.name, Binding::Struct(s.clone())),
            Stmt::Enum(s) => self.stack.bind(s.name, Binding::Enum(s.clone())),
            Stmt::Type(s) => self.stack.bind(s.name, Binding::Type(s.clone())),
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }

    fn stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(Rc::new(self.stmt_var(s))),
            Stmt::Def(s) => Stmt::Def(Rc::new(self.stmt_def(s))),
            Stmt::Trait(s) => Stmt::Trait(Rc::new(self.stmt_trait(s))),
            Stmt::Impl(s) => Stmt::Impl(Rc::new(self.stmt_impl(s))),
            Stmt::Struct(s) => Stmt::Struct(Rc::new(self.stmt_struct(s))),
            Stmt::Enum(s) => Stmt::Enum(Rc::new(self.stmt_enum(s))),
            Stmt::Type(s) => Stmt::Type(Rc::new(self.stmt_type(s))),
            Stmt::Expr(s) => Stmt::Expr(Rc::new(self.expr(s))),
            Stmt::Err(s) => Stmt::Err(*s),
        }
    }

    fn stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let name = s.name;
        let ty = self.ty(&s.ty);
        let expr = self.expr(&s.expr);
        self.stack.bind(name, Binding::Var);
        StmtVar::new(span, name, ty, expr)
    }

    fn stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        self.scoped(|ctx| {
            let span = s.span;
            let name = s.name;
            s.generics
                .iter()
                .for_each(|g| ctx.stack.bind(*g, Binding::Generic));
            let generics = s.generics.clone();
            let where_clause = s.where_clause.iter().map(|p| ctx.bound(p)).collect();
            let params = s.params.iter().map(|p| ctx.param(p)).collect();
            let ty = ctx.ty(&s.ty);
            let body = ctx.body(&s.body);
            StmtDef::new(span, name, generics, params, ty, where_clause, body)
        })
    }

    fn body(&mut self, body: &StmtDefBody) -> StmtDefBody {
        match body {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(self.expr(e)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }

    fn stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|g| ctx.stack.bind(*g, Binding::Generic));
            let span = s.span;
            let name = s.name;
            let generics = s.generics.clone();
            let body = s.where_clause.iter().map(|p| ctx.bound(p)).collect();
            let defs = s.defs.iter().map(|d| Rc::new(ctx.trait_def(d))).collect();
            let types = s.types.clone();
            StmtTrait::new(span, name, generics, body, defs, types)
        })
    }

    fn trait_def(&mut self, s: &StmtTraitDef) -> StmtTraitDef {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|g| ctx.stack.bind(*g, Binding::Generic));
            let span = s.span;
            let name = s.name;
            let generics = s.generics.clone();
            let params = s.params.iter().map(|p| ctx.param(p)).collect();
            let ty = ctx.ty(&s.ty);
            let where_clause = s.where_clause.iter().map(|p| ctx.bound(p)).collect();
            StmtTraitDef::new(span, name, generics, params, ty, where_clause)
        })
    }

    fn stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|generic| ctx.stack.bind(*generic, Binding::Generic));
            let span = s.span;
            let generics = s.generics.clone();
            let head = ctx.head(&s.head, &s.defs, &s.types);
            let body = s
                .where_clause
                .iter()
                .map(|p| ctx.bound(p))
                .collect::<Vec<_>>();
            let defs = s.defs.iter().map(|d| Rc::new(ctx.stmt_def(d))).collect();
            let types = s.types.iter().map(|t| Rc::new(ctx.stmt_type(t))).collect();
            StmtImpl::new(span, generics, head, body, defs, types)
        })
    }

    fn stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|generic| ctx.stack.bind(*generic, Binding::Generic));
            let span = s.span;
            let name = s.name;
            let generics = s.generics.clone();
            let fields = s.fields.iter().map(|(x, t)| (*x, ctx.ty(t))).collect();
            StmtStruct::new(span, name, generics, fields)
        })
    }

    fn stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|generic| ctx.stack.bind(*generic, Binding::Generic));
            let span = s.span;
            let name = s.name;
            let generics = s.generics.clone();
            let variants = s.variants.iter().map(|(x, t)| (*x, ctx.ty(t))).collect();
            StmtEnum::new(span, name, generics, variants)
        })
    }

    fn stmt_type(&mut self, s: &StmtType) -> StmtType {
        self.scoped(|ctx| {
            s.generics
                .iter()
                .for_each(|g| ctx.stack.bind(*g, Binding::Generic));
            let span = s.span;
            let name = s.name;
            let generics = s.generics.clone();
            let ty = ctx.stmt_type_body(&s.body);
            StmtType::new(span, name, generics, ty)
        })
    }

    fn stmt_type_body(&mut self, s: &StmtTypeBody) -> StmtTypeBody {
        match s {
            StmtTypeBody::UserDefined(t) => StmtTypeBody::UserDefined(self.ty(t)),
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
        }
    }

    fn param(&mut self, (x, t): &(Name, Type)) -> (Name, Type) {
        self.stack.bind(*x, Binding::Var);
        let t = self.ty(t);
        (*x, t)
    }

    fn path(&mut self, p: &Path) -> Path {
        let segments = p
            .segments
            .iter()
            .map(|seg| {
                let x = seg.name;
                let ts = seg.ts.iter().map(|t| self.ty(t)).collect();
                let xts = seg.xts.iter().map(|(x, t)| (*x, self.ty(t))).collect();
                Segment::new(seg.span, x, ts, xts)
            })
            .collect();
        Path::new(segments)
    }

    fn ty(&mut self, t: &Type) -> Type {
        match t {
            Type::Unresolved(p) => self.resolve_type_path(p),
            Type::Hole => Type::Hole,
            Type::Generic(x) => Type::Generic(*x),
            Type::Fun(ts, t) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                let t = self.ty(t);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, self.ty(t))).collect();
                Type::Record(xts)
            }
            Type::Array(t, n) => {
                let t = self.ty(t);
                let n = *n;
                Type::Array(Rc::new(t), n)
            }
            Type::Never => Type::Never,
            Type::Err => Type::Err,
            Type::Cons(x, ts) => Type::Cons(*x, ts.iter().map(|t| self.ty(t)).collect()),
            Type::Alias(..) => unreachable!(),
            Type::Assoc(..) => unreachable!(),
            Type::Var(_) => unreachable!(),
        }
    }

    fn expr(&mut self, e: &Expr) -> Expr {
        let t = self.ty(e.ty());
        let s = e.span();
        match e {
            Expr::Unresolved(_s, _t, path) => self.resolve_expr_path(s, t, path),
            Expr::Int(s, t, v) => {
                let t = self.ty(t);
                Expr::Int(*s, t, *v)
            }
            Expr::Float(s, t, v) => {
                let t = self.ty(t);
                Expr::Float(*s, t, *v)
            }
            Expr::Bool(s, t, v) => {
                let t = self.ty(t);
                Expr::Bool(*s, t, *v)
            }
            Expr::Tuple(s, t, es) => {
                let t = self.ty(t);
                let es = es.iter().map(|e| self.expr(e)).collect();
                Expr::Tuple(*s, t, es)
            }
            Expr::Call(s, t, e, es) => self.expr_call(*s, t, e, es),
            Expr::String(_, _, v) => Expr::String(s, t, *v),
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                Expr::Field(s, t, Rc::new(e), *x)
            }
            Expr::Block(_, _, b) => {
                let b = self.block(b);
                Expr::Block(s, t, b)
            }
            Expr::Query(..) => {
                todo!()
            }
            Expr::Index(_, _, e, i) => {
                let e = self.expr(e);
                let i = *i;
                Expr::Index(s, t, Rc::new(e), i)
            }
            Expr::Array(_, _, es) => {
                let es = es.iter().map(|e| self.expr(e)).collect();
                Expr::Array(s, t, es)
            }
            Expr::Err(_, _) => Expr::Err(s, t),
            Expr::Assign(_, _, e0, e1) => {
                let e0 = self.expr(e0);
                let e1 = self.expr(e1);
                let e1 = self.lvalue(&e1);
                Expr::Assign(s, t, Rc::new(e0), Rc::new(e1))
            }
            Expr::Return(_, _, e) => {
                let e = self.expr(e);
                Expr::Return(s, t, Rc::new(e))
            }
            Expr::Continue(_, _) => Expr::Continue(s, t),
            Expr::Break(_, _) => Expr::Break(s, t),
            Expr::Fun(_, _, ps, t1, e) => {
                let ps = ps.iter().map(|p| self.param(p)).collect();
                let t1 = self.ty(t1);
                let e = self.expr(e);
                Expr::Fun(s, t, ps, t1, Rc::new(e))
            }
            Expr::Match(_, _, e, xps) => {
                let e = self.expr(e);
                let xps = xps.iter().map(|arm| self.arm(arm)).collect();
                Expr::Match(s, t, Rc::new(e), xps)
            }
            Expr::While(_, _, e, b) => {
                let e = self.expr(e);
                let b = self.block(b);
                Expr::While(s, t, Rc::new(e), b)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes.iter().map(|(x, e)| (*x, self.expr(e))).collect();
                Expr::Record(s, t, xes)
            }
            Expr::Char(_, _, c) => Expr::Char(s, t, *c),
            Expr::Value(_, _) => unreachable!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Def(..) => unreachable!(),
            Expr::Var(..) => unreachable!(),
            Expr::Struct(..) => unreachable!(),
            Expr::Enum(..) => unreachable!(),
            Expr::Assoc(..) => unreachable!(),
        }
    }

    fn block(&mut self, b: &Block) -> Block {
        self.scoped(|ctx| {
            let span = b.span;
            b.stmts.iter().for_each(|s| ctx.declare_stmt(s));
            let stmts = b.stmts.iter().map(|s| ctx.stmt(s)).collect();
            let e = ctx.expr(&b.expr);
            Block::new(span, stmts, e)
        })
    }

    fn arm(&mut self, arm: &Arm) -> Arm {
        self.scoped(|ctx| {
            let span = arm.span;
            let p = ctx.pat(&arm.p);
            let e = ctx.expr(&arm.e);
            Arm::new(span, p, e)
        })
    }

    fn fill_type_args(args: Vec<Type>, expected: usize) -> Vec<Type> {
        args.is_empty()
            .then(|| (0..expected).map(|_| Type::Hole).collect())
            .unwrap_or(args)
    }

    fn expr_call(&mut self, s: Span, t1: &Type, e: &Expr, es: &[Expr]) -> Expr {
        let t1 = self.ty(t1);
        if let Expr::Unresolved(_s, _t, path) = e {
            let path = self.path(path);
            let mut iter = path.segments.into_iter();
            let seg0 = iter.next().unwrap();
            match self.stack.get(&seg0.name) {
                Some(Binding::Struct(stmt)) => {
                    let Some(ts0) = seg0.try_instantiate(stmt.generics.len()) else {
                        self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                        return Expr::Err(s, t1);
                    };
                    if seg0.has_named_args() {
                        self.unexpected_named_type_args(&seg0.name);
                        return Expr::Err(s, t1);
                    }
                    if let Some(seg1) = iter.next() {
                        self.unexpected_assoc("Struct", "field", &seg0.name, &seg1.name);
                        return Expr::Err(s, t1);
                    }
                    let xes: Map<_, _> = es.iter().flat_map(|e| self.expr_field(e)).collect();
                    if !fields_are_defined(&stmt.fields, &xes) {
                        self.wrong_fields(&seg0.name, Some(&xes), &stmt.fields);
                        return Expr::Err(s, t1);
                    }
                    return Expr::Struct(s, t1, seg0.name, ts0, xes);
                }
                Some(Binding::Enum(stmt)) => {
                    if !seg0.has_optional_arity(stmt.generics.len()) {
                        self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                        return Expr::Err(s, t1);
                    }
                    if seg0.has_named_args() {
                        self.unexpected_named_type_args(&seg0.name);
                        return Expr::Err(s, t1);
                    }
                    let ts0 = seg0.instantiate_unnamed(stmt.generics.len());
                    let Some(seg1) = iter.next() else {
                        self.expected_assoc("variant", &seg0.name);
                        return Expr::Err(s, t1);
                    };
                    if !stmt.variants.contains_key(&seg1.name) {
                        self.unexpected_assoc("Enum", "variant", &seg0.name, &seg1.name);
                        return Expr::Err(s, t1);
                    }
                    if !seg1.ts.is_empty() {
                        self.wrong_arity(&seg1.name, seg1.ts.len(), 0);
                        return Expr::Err(s, t1);
                    }
                    if let Some(seg2) = iter.next() {
                        self.unexpected_assoc("Enum", "item", &seg1.name, &seg2.name);
                        return Expr::Err(s, t1);
                    }
                    let es = es.iter().map(|e| self.expr(e)).collect::<Vec<_>>();
                    let e = match es.len() {
                        1 => es.into_iter().next().unwrap(),
                        _ => Expr::Tuple(s, Type::Hole, es),
                    };
                    return Expr::Enum(s, t1, seg0.name, ts0, seg1.name, Rc::new(e));
                }
                _ => {}
            }
        }
        let e = self.expr(e);
        let es = es.iter().map(|e| self.expr(e)).collect();
        Expr::Call(s, t1, Rc::new(e), es)
    }

    fn expr_field(&mut self, e: &Expr) -> Option<(Name, Expr)> {
        let s = e.span();
        let t = self.ty(e.ty());
        match e {
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                Some((*x, e))
            }
            Expr::Assign(_, _, e0, e1) => {
                let e1 = self.expr(e1);
                if let Expr::Unresolved(_, _, path) = &**e0 {
                    let mut iter = path.segments.iter();
                    let seg0 = iter.next().unwrap();
                    if !seg0.ts.is_empty() {
                        self.wrong_arity(&seg0.name, seg0.ts.len(), 0);
                        return None;
                    }
                    if let Some(seg1) = iter.next() {
                        self.unexpected_assoc("Assignment", "item", &seg0.name, &seg1.name);
                        return None;
                    }
                    Some((seg0.name, e1))
                } else {
                    self.expected_name(e0);
                    None
                }
            }
            Expr::Unresolved(_, _, path) => {
                let mut iter = path.segments.iter();
                let seg0 = iter.next().unwrap();
                match self.stack.get(&seg0.name) {
                    Some(Binding::Var) => {
                        if !seg0.ts.is_empty() {
                            self.wrong_arity(&seg0.name, seg0.ts.len(), 0);
                            return None;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Variable", "item", &seg0.name, &seg1.name);
                            return None;
                        }
                        Some((seg0.name, Expr::Var(s, t, seg0.name)))
                    }
                    Some(b) => {
                        self.unexpected(&seg0.name, b.name(), "variable");
                        None
                    }
                    None => {
                        self.not_found(&seg0.name, "variable");
                        None
                    }
                }
            }
            _ => {
                self.report.err(
                    e.span(),
                    "Not a field.",
                    "Expected `<name> = <expr>`, `<name>` or `<expr>.<name>`.",
                );
                None
            }
        }
    }

    fn pat(&mut self, p: &Pat) -> Pat {
        let t = self.ty(p.ty());
        let s = p.span();
        match p {
            Pat::Unresolved(_, _, path, args) => self.resolve_pat_path(s, t, path, args),
            Pat::Var(_, _, x) => Pat::Var(s, t, *x),
            Pat::Tuple(_, _, ts) => {
                let ts = ts.iter().map(|t| self.pat(t)).collect();
                Pat::Tuple(s, t, ts)
            }
            Pat::Int(_, _, v) => Pat::Int(s, t, *v),
            Pat::String(_, _, v) => Pat::String(s, t, *v),
            Pat::Wildcard(_, _) => Pat::Wildcard(s, t),
            Pat::Bool(_, _, v) => Pat::Bool(s, t, *v),
            Pat::Err(_, _) => Pat::Err(s, t),
            Pat::Record(_, _, xps) => {
                let xps = xps.iter().map(|(x, p)| (*x, self.pat(p))).collect();
                Pat::Record(s, t, xps)
            }
            Pat::Or(_, _, p0, p1) => {
                let p0 = self.pat(p0);
                let p1 = self.pat(p1);
                Pat::Or(s, t, Rc::new(p0), Rc::new(p1))
            }
            Pat::Char(_, _, c) => Pat::Char(s, t, *c),
            Pat::Struct(..) => unreachable!(),
            Pat::Enum(..) => unreachable!(),
        }
    }

    fn enum_pat_args(
        &mut self,
        args: &Option<Vec<UnresolvedPatField>>,
        is_unit_enum: bool,
    ) -> Option<Vec<Pat>> {
        if args.is_none() && is_unit_enum {
            return Some(vec![]);
        }
        let args = args.as_ref().unwrap();
        let mut ps = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                UnresolvedPatField::Named(x, p) => {
                    self.report.err(
                        x.span,
                        format!("Expected `<pat>`, found `{x} = {p}`.",),
                        "Expected unnamed pattern `<pat>`.",
                    );
                    return None;
                }
                UnresolvedPatField::Unnamed(p) => ps.push(p.clone()),
            }
        }
        Some(ps)
    }

    fn struct_pat_args(
        &mut self,
        args: &Option<Vec<UnresolvedPatField>>,
    ) -> Option<Map<Name, Pat>> {
        if args.is_none() {
            return Some(Map::new());
        }
        let args = args.as_ref().unwrap();
        let mut xps = Map::new();
        for arg in args {
            match arg {
                UnresolvedPatField::Named(x, p) => xps.insert(*x, p.clone()),
                UnresolvedPatField::Unnamed(p) => {
                    self.report.err(
                        p.span(),
                        format!("Expected `{p} = <pat>`.",),
                        format!("Expected named pattern `{p} = <pat>`."),
                    );
                    return None;
                }
            }
        }
        Some(xps)
    }

    // impl Foo[T] { ... }
    #[allow(clippy::type_complexity)]
    fn head(&mut self, bound: &Bound, defs: &[Rc<StmtDef>], types: &[Rc<StmtType>]) -> Bound {
        let span = bound.span();
        match bound {
            Bound::Unresolved(_, path) => self.resolve_head_path(span, path, defs, types),
            Bound::Trait(s, b) => Bound::Trait(*s, b.clone()),
            Bound::Err(_) => Bound::Err(span),
        }
    }

    // impl ... where Foo[T] { ... }
    #[allow(clippy::type_complexity)]
    fn bound(&mut self, bound: &Bound) -> Bound {
        let span = bound.span();
        match bound {
            Bound::Unresolved(_, path) => self.resolve_bound_path(span, path),
            Bound::Trait(..) => unreachable!(),
            Bound::Err(_) => Bound::Err(span),
        }
    }

    // x = e;
    // x.y = e;
    // x[i] = e;
    fn lvalue(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Var(s, t, x) => Expr::Var(*s, t.clone(), *x),
            Expr::Field(s, t, e, x) => {
                let e = self.lvalue(e.as_ref());
                Expr::Field(*s, t.clone(), Rc::new(e), *x)
            }
            Expr::Index(s, t, e, i) => {
                let e = self.lvalue(e.as_ref());
                Expr::Index(*s, t.clone(), Rc::new(e), *i)
            }
            _ => {
                self.report.err(
                    e.span(),
                    "Expression is not an lvalue.",
                    "Only variables, field access, and tuple access are allowed.",
                );
                Expr::Err(e.span(), e.ty().clone())
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn resolve_head_path(
        &mut self,
        span: Span,
        path: &Path,
        found_defs: &[Rc<StmtDef>],
        found_types: &[Rc<StmtType>],
    ) -> Bound {
        let path = self.path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.stack.get(&seg0.name) {
            Some(Binding::Trait(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Bound::Err(span);
                }
                let ts0 = seg0.instantiate_unnamed(stmt.generics.len());
                if seg0.has_named_args() {
                    self.unexpected_named_type_args(&seg0.name);
                    return Bound::Err(span);
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Trait", "item", &seg0.name, &seg1.name);
                    return Bound::Err(span);
                }
                if !defs_are_defined(&stmt.defs, found_defs) {
                    self.wrong_items(
                        "def",
                        &seg0.name,
                        found_defs.iter().map(|def| &def.name),
                        stmt.defs.iter().map(|def| &def.name),
                    );
                    return Bound::Err(span);
                }
                if !types_are_defined(&stmt.types, found_types) {
                    self.wrong_items(
                        "type",
                        &seg0.name,
                        found_types.iter().map(|ty| &ty.name),
                        stmt.types.iter().map(|ty| &ty.name),
                    );
                    return Bound::Err(span);
                }
                let xts = stmt.types.iter().map(|ty| (ty.name, Type::Hole)).collect();
                let b = TraitBound::new(seg0.name, ts0, xts);
                Bound::Trait(span, b)
            }
            Some(b) => {
                self.unexpected(&seg0.name, b.name(), "trait");
                Bound::Err(span)
            }
            None => {
                self.not_found(&seg0.name, "trait");
                Bound::Err(span)
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn resolve_bound_path(&mut self, span: Span, path: &Path) -> Bound {
        let path = self.path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.stack.get(&seg0.name) {
            Some(Binding::Trait(stmt)) => {
                let Some(ts0) = seg0.try_instantiate(stmt.generics.len()) else {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Bound::Err(span);
                };
                let Some(xts0) = seg0.try_instantiate_named(&stmt.types) else {
                    return Bound::Err(span);
                };
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Trait", "item", &seg0.name, &seg1.name);
                    return Bound::Err(span);
                }
                let b = TraitBound::new(seg0.name, ts0, xts0);
                Bound::Trait(span, b)
            }
            Some(b) => {
                self.unexpected(&seg0.name, b.name(), "trait");
                Bound::Err(span)
            }
            None => {
                self.not_found(&seg0.name, "trait");
                Bound::Err(span)
            }
        }
    }

    fn resolve_expr_path(&mut self, s: Span, t: Type, path: &Path) -> Expr {
        let path = self.path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.stack.get(&seg0.name) {
            Some(Binding::Var) => {
                if !seg0.ts.is_empty() {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), 0);
                    return Expr::Err(s, t.clone());
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Variable", "item", &seg0.name, &seg1.name);
                    return Expr::Err(s, t.clone());
                }
                Expr::Var(s, t, seg0.name)
            }
            Some(Binding::Def(stmt)) => {
                if seg0.ts.len() != stmt.generics.len() && !seg0.ts.is_empty() {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Expr::Err(s, t);
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Def", "item", &seg0.name, &seg1.name);
                    return Expr::Err(s, t);
                }
                let ts0 = seg0.instantiate_unnamed(stmt.generics.len());
                Expr::Def(s, t, seg0.name, ts0.clone())
            }
            // Unit Struct
            Some(Binding::Struct(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Expr::Err(s, t.clone());
                }
                if !stmt.fields.is_empty() {
                    self.wrong_fields::<Type, _>(&seg0.name, None, &stmt.fields);
                    return Expr::Err(s, t.clone());
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Struct", "item", &seg0.name, &seg1.name);
                    return Expr::Err(s, t);
                }
                let ts0 = seg0.instantiate_unnamed(stmt.generics.len());
                let t = self.ty(&t);
                let x0 = seg0.name;
                Expr::Struct(s, t, x0, ts0.clone(), Map::new())
            }
            Some(Binding::Trait(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Expr::Err(s, t);
                }
                if seg0.has_named_args() {
                    self.unexpected_named_type_args(&seg0.name);
                    return Expr::Err(s, t);
                }
                let Some(seg1) = iter.next() else {
                    self.expected_assoc("function", &seg0.name);
                    return Expr::Err(s, t);
                };
                let Some(def) = stmt.defs.iter().find(|def| def.name == seg1.name) else {
                    self.unexpected_assoc("Trait", "function", &seg0.name, &seg1.name);
                    return Expr::Err(s, t);
                };
                if !seg1.has_optional_arity(def.generics.len()) {
                    self.wrong_arity(&seg1.name, seg1.ts.len(), def.generics.len());
                    return Expr::Err(s, t);
                }
                let ts0 = seg0.instantiate_unnamed(stmt.generics.len());
                let b = TraitBound::new(seg0.name, ts0, Map::new());
                Expr::Assoc(s, t, b, seg1.name, seg1.ts.clone())
            }
            Some(b) => {
                self.unexpected(&seg0.name, b.name(), "expression");
                Expr::Err(s, t.clone())
            }
            None => {
                for stmt in self.stack.traits() {
                    for def in stmt.defs.iter() {
                        if seg0.name == def.name && seg0.has_optional_arity(def.generics.len()) {
                            return Expr::Assoc(
                                s,
                                t,
                                TraitBound::new(
                                    stmt.name,
                                    vec![Type::Hole; stmt.generics.len()],
                                    Map::new(),
                                ),
                                seg0.name,
                                seg0.instantiate_unnamed(def.generics.len()),
                            );
                        }
                    }
                }
                self.not_found(&seg0.name, "expression");
                Expr::Err(s, t.clone())
            }
        }
    }

    fn resolve_type_path(&mut self, p: &Path) -> Type {
        let p = self.path(p);
        let mut iter = p.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.stack.get(&seg0.name) {
            Some(Binding::Generic) => {
                if seg0.has_args() {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), 0);
                    return Type::Err;
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Generic", "item", &seg0.name, &seg1.name);
                    return Type::Err;
                }
                Type::Generic(seg0.name)
            }
            Some(Binding::Enum(stmt)) => {
                if !seg0.has_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Type::Err;
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Type", "item", &seg0.name, &seg1.name);
                    return Type::Err;
                }
                Type::Cons(seg0.name, seg0.ts.clone())
            }
            Some(Binding::Struct(stmt)) => {
                if !seg0.has_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Type::Err;
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Type", "item", &seg0.name, &seg1.name);
                    return Type::Err;
                }
                Type::Cons(seg0.name, seg0.ts.clone())
            }
            Some(Binding::Type(stmt)) => {
                if !seg0.has_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Type::Err;
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Type", "item", &seg0.name, &seg1.name);
                    return Type::Err;
                }
                match &stmt.body {
                    StmtTypeBody::UserDefined(_) => Type::Alias(seg0.name, seg0.ts.clone()),
                    StmtTypeBody::Builtin(_) => Type::Cons(seg0.name, seg0.ts.clone()),
                }
            }
            Some(Binding::Trait(stmt)) => {
                if !seg0.has_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Type::Err;
                }
                let Some(seg1) = iter.next() else {
                    self.expected_assoc("type", &seg0.name);
                    return Type::Err;
                };
                let Some(t) = stmt.types.iter().find(|t| t.name == seg1.name) else {
                    self.unexpected_assoc("Trait", "type", &seg0.name, &seg1.name);
                    return Type::Err;
                };
                if !seg1.has_arity(t.generics.len()) {
                    self.wrong_arity(&seg1.name, seg1.ts.len(), t.generics.len());
                    return Type::Err;
                }
                let xts = stmt.types.iter().map(|t| (t.name, Type::Hole)).collect();
                let b = TraitBound::new(seg0.name, seg0.ts.clone(), xts);
                Type::Assoc(b, seg1.name, seg1.ts.clone())
            }
            Some(b) => {
                self.unexpected(&seg0.name, b.name(), "type");
                Type::Err
            }
            None => {
                self.not_found(&seg0.name, "type");
                Type::Err
            }
        }
    }

    fn resolve_pat_path(
        &mut self,
        s: Span,
        t: Type,
        path: &Path,
        args: &Option<Vec<UnresolvedPatField>>,
    ) -> Pat {
        let path = self.path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.stack.get(&seg0.name) {
            Some(Binding::Enum(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Pat::Err(s, t.clone());
                }
                let ts0 = Self::fill_type_args(seg0.ts, stmt.generics.len());
                let Some(seg1) = iter.next() else {
                    self.expected_assoc("variant", &seg0.name);
                    return Pat::Err(s, t.clone());
                };
                if !stmt.variants.contains_key(&seg1.name) {
                    self.unexpected_assoc("Enum", "variant", &seg0.name, &seg1.name);
                    return Pat::Err(s, t.clone());
                }
                if !seg1.ts.is_empty() {
                    self.wrong_arity(&seg1.name, seg1.ts.len(), 0);
                    return Pat::Err(s, t.clone());
                }
                if let Some(seg2) = iter.next() {
                    self.unexpected_assoc("Enum", "item", &seg1.name, &seg2.name);
                    return Pat::Err(s, t.clone());
                }
                let Some(ps) = self.enum_pat_args(args, stmt.variants.is_empty()) else {
                    return Pat::Err(s, t.clone());
                };
                let p = match ps.len() {
                    1 => ps.into_iter().next().unwrap(),
                    _ => Pat::Tuple(s, Type::Hole, ps),
                };
                Pat::Enum(s, t, seg0.name, ts0.clone(), seg1.name, Rc::new(p))
            }
            Some(Binding::Struct(stmt)) => {
                if !seg0.has_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), stmt.generics.len());
                    return Pat::Err(s, t.clone());
                }
                let ts0 = Self::fill_type_args(seg0.ts, stmt.generics.len());
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Struct", "item", &seg0.name, &seg1.name);
                    return Pat::Err(s, t.clone());
                }
                let Some(xps) = self.struct_pat_args(args) else {
                    return Pat::Err(s, t.clone());
                };
                if !fields_are_defined(&stmt.fields, &xps) {
                    self.wrong_fields(&seg0.name, Some(&xps), &stmt.fields);
                    return Pat::Err(s, t.clone());
                }
                Pat::Struct(s, t, seg0.name, ts0.clone(), xps)
            }
            Some(b) => {
                self.unexpected(&seg0.name, b.name(), "pattern");
                Pat::Err(s, t.clone())
            }
            None => {
                if !seg0.ts.is_empty() {
                    self.wrong_arity(&seg0.name, seg0.ts.len(), 0);
                    return Pat::Err(s, t.clone());
                }
                if let Some(seg) = iter.next() {
                    self.unexpected_assoc("Struct", "item", &seg0.name, &seg.name);
                    return Pat::Err(s, t.clone());
                }
                Pat::Var(s, t, seg0.name)
            }
        }
    }
}

impl Binding {
    fn name(&self) -> &'static str {
        match self {
            Binding::Enum(..) => "enum",
            Binding::Struct(..) => "struct",
            Binding::Generic => "generic",
            Binding::Type(..) => "type",
            Binding::Trait(..) => "trait",
            Binding::Var => "variable",
            Binding::Def(..) => "definition",
        }
    }
}

impl Segment {
    fn has_optional_arity(&self, arity: usize) -> bool {
        self.ts.is_empty() || self.ts.len() == arity
    }

    fn has_arity(&self, arity: usize) -> bool {
        self.ts.len() == arity
    }

    fn has_unnamed_args(&self) -> bool {
        !self.ts.is_empty()
    }

    fn has_named_args(&self) -> bool {
        !self.xts.is_empty()
    }

    fn has_args(&self) -> bool {
        self.has_unnamed_args() || self.has_named_args()
    }

    fn try_instantiate(&self, arity: usize) -> Option<Vec<Type>> {
        if self.ts.is_empty() {
            Some(vec![Type::Hole; arity])
        } else if self.ts.len() == arity {
            Some(self.ts.clone())
        } else {
            None
        }
    }

    fn try_instantiate_named(&self, expected: &[Rc<StmtTraitType>]) -> Option<Map<Name, Type>> {
        if self
            .xts
            .iter()
            .all(|(x, _)| expected.iter().any(|s| *x == s.name))
        {
            // If all named types are defined, return the named types.
            let xts = expected
                .iter()
                .filter(|s| !self.xts.contains_key(&s.name))
                .cloned()
                .map(|s| (s.name, Type::Hole));
            Some(self.xts.clone().into_iter().chain(xts).collect())
        } else {
            None
        }
    }

    fn instantiate_unnamed(&self, n: usize) -> Vec<Type> {
        if self.ts.is_empty() {
            vec![Type::Hole; n]
        } else {
            self.ts.clone()
        }
    }
}

fn fields_are_defined<T>(expected: &Map<Name, Type>, provided: &Map<Name, T>) -> bool {
    provided.keys().all(|x| expected.contains_key(x))
        && expected.keys().all(|x| provided.contains_key(x))
}

fn types_are_defined(expected: &[Rc<StmtTraitType>], provided: &[Rc<StmtType>]) -> bool {
    provided.iter().all(|x| {
        expected
            .iter()
            .any(|y| x.name == y.name && x.generics.len() == y.generics.len())
    }) && expected.iter().all(|x| {
        provided
            .iter()
            .any(|y| x.name == y.name && x.generics.len() == y.generics.len())
    })
}

fn defs_are_defined(expected: &[Rc<StmtTraitDef>], provided: &[Rc<StmtDef>]) -> bool {
    provided.iter().all(|x| {
        expected
            .iter()
            .any(|y| x.name == y.name && x.generics.len() == y.generics.len())
    }) && expected.iter().all(|x| {
        provided
            .iter()
            .any(|y| x.name == y.name && x.generics.len() == y.generics.len())
    })
}

fn comma_sep<'a>(names: impl IntoIterator<Item = &'a Name>) -> String {
    names
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
