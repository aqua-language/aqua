use std::rc::Rc;

use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Param;
use crate::ast::Pat;
use crate::ast::PatArg;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtType;
use crate::ast::StmtVar;
use crate::ast::TraitDef;
use crate::ast::Type;
use crate::ast::UnresolvedPath;
use crate::diag::Report;
use crate::lexer::Span;

fn names_to_string<T>(names: &[T], f: impl FnMut(&T) -> String) -> String {
    names.iter().map(f).collect::<Vec<_>>().join(", ")
}

impl std::ops::Deref for Context {
    type Target = Stack;
    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}

impl std::ops::DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stack
    }
}

#[derive(Debug)]
pub struct Stack(Vec<Scope>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Vec<(Name, Binding)>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Binding {
    Enum(usize, Vec<Name>),
    Struct(usize, Vec<Name>),
    BuiltinType(usize),
    BuiltinDef(usize),
    Generic,
    TypeAlias(usize),
    Trait(usize, Vec<(Name, usize)>, Vec<(Name, usize)>),
    Var,
    Def(usize),
}

impl Stack {
    fn bind(&mut self, name: impl Into<Name>, binding: Binding) {
        self.0.last_mut().unwrap().0.push((name.into(), binding));
    }

    fn get(&self, x: &Name) -> Option<&Binding> {
        self.0.iter().rev().find_map(|s| {
            s.0.iter()
                .rev()
                .find_map(|(y, b)| if y == x { Some(b) } else { None })
        })
    }

    fn traits(&self) -> impl Iterator<Item = (&Name, usize, &[(Name, usize)], &[(Name, usize)])> {
        self.0
            .iter()
            .rev()
            .flat_map(|s| s.0.iter())
            .filter_map(|(x, b)| match b {
                Binding::Trait(n, xs0, xs1) => Some((x, *n, xs0.as_slice(), xs1.as_slice())),
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
        let mut stack = Stack(vec![Scope(vec![])]);
        // Types
        stack.bind("i32", Binding::BuiltinType(0));
        stack.bind("f32", Binding::BuiltinType(0));
        stack.bind("i64", Binding::BuiltinType(0));
        stack.bind("bool", Binding::BuiltinType(0));
        stack.bind("Vec", Binding::BuiltinType(1));
        stack.bind("VecIterator", Binding::BuiltinType(1));
        stack.bind("Stream", Binding::BuiltinType(1));
        stack.bind("StreamIterator", Binding::BuiltinType(1));
        stack.bind("Option", Binding::BuiltinType(1));
        // Traits
        stack.bind(
            "Iterator",
            Binding::Trait(1, vec![], vec![(Name::from("Item"), 0)]),
        );
        stack.bind(
            "IntoIterator",
            Binding::Trait(
                1,
                vec![],
                vec![(Name::from("Item"), 0), (Name::from("IntoIter"), 0)],
            ),
        );
        stack.bind(
            "Add",
            Binding::Trait(
                2,
                vec![(Name::from("add"), 0)],
                vec![(Name::from("Output"), 0)],
            ),
        );
        stack.bind(
            "Sub",
            Binding::Trait(
                2,
                vec![(Name::from("sub"), 0)],
                vec![(Name::from("Output"), 0)],
            ),
        );
        stack.bind(
            "Mul",
            Binding::Trait(
                2,
                vec![(Name::from("mul"), 0)],
                vec![(Name::from("Output"), 0)],
            ),
        );
        stack.bind(
            "Div",
            Binding::Trait(
                2,
                vec![(Name::from("div"), 0)],
                vec![(Name::from("Output"), 0)],
            ),
        );
        stack.bind(
            "Eq",
            Binding::Trait(
                1,
                vec![(Name::from("eq"), 0), (Name::from("ne"), 0)],
                vec![],
            ),
        );
        stack.bind(
            "Not",
            Binding::Trait(1, vec![(Name::from("not"), 0)], vec![]),
        );
        stack.bind(
            "Ord",
            Binding::Trait(1, vec![(Name::from("cmp"), 0)], vec![]),
        );
        stack.bind(
            "Clone",
            Binding::Trait(1, vec![(Name::from("clone"), 0)], vec![]),
        );
        stack.bind("Copy", Binding::Trait(1, vec![], vec![]));
        stack.bind(
            "Display",
            Binding::Trait(1, vec![(Name::from("to_string"), 0)], vec![]),
        );
        let report = Report::new();
        Context { stack, report }
    }

    fn scoped_block<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.stack.0.push(Scope(vec![]));
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

    fn wrong_arity(&mut self, name: &Name, found: usize, expected: usize) {
        self.report.err(
            name.span,
            format!(
                "Wrong number of type arguments. Found {}, expected {}",
                found, expected
            ),
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

    fn wrong_fields<T>(&mut self, name: &Name, found: &[(Name, T)], expected: &[Name]) {
        let found = names_to_string(found, |(x, _)| format!("{x}"));
        let expected = names_to_string(expected, |x| format!("{x}"));
        self.report.err(
            name.span,
            format!("Wrong fields provided. Found {name}({found}), expected {name}({expected})",),
            format!("Expected {name}({expected}) fields."),
        );
    }

    fn wrong_items(
        &mut self,
        kind: &'static str,
        name: &Name,
        found: &[(Name, usize)],
        expected: &[(Name, usize)],
    ) {
        let found = names_to_string(found, |(x, _)| format!("`{x}`"));
        let expected = names_to_string(expected, |(x, _)| format!("`{x}`"));
        self.report.err(
            name.span,
            format!("Wrong {kind}s implemented for {name}. Found {{ {found} }}, expected {{ {expected} }}",),
            format!("Expected {{ {expected} }}."),
        );
    }

    fn wrong_variant<T>(&mut self, name: &Name, found: &(Name, T), expected: &[Name]) {
        let found = &found.0;
        let expected = names_to_string(expected, |x| format!("{x}"));
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

    pub fn resolve(&mut self, p: &Program) -> Program {
        p.stmts.iter().for_each(|stmt| self.declare_stmt(stmt));
        let stmts = p.stmts.iter().map(|stmt| self.stmt(stmt)).collect();
        Program::new(stmts)
    }

    pub fn declare_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => self.declare_stmt_def(s),
            Stmt::Trait(s) => self.declare_stmt_trait(s),
            Stmt::Impl(_) => {}
            Stmt::Struct(s) => self.declare_stmt_struct(s),
            Stmt::Enum(s) => self.declare_stmt_enum(s),
            Stmt::Type(s) => self.declare_stmt_type(s),
            Stmt::Expr(_) => {}
            Stmt::Err(_) => todo!(),
        }
    }

    pub fn declare_stmt_def(&mut self, s: &StmtDef) {
        let name = s.name.clone();
        self.bind(name, Binding::Def(s.generics.len()));
    }

    pub fn declare_stmt_trait(&mut self, s: &StmtTrait) {
        let name = s.name.clone();
        let def_names = s
            .defs
            .iter()
            .map(|d| (d.name.clone(), d.generics.len()))
            .collect();
        let type_names = s
            .types
            .iter()
            .map(|t| (t.name.clone(), t.generics.len()))
            .collect();
        self.bind(
            name.clone(),
            Binding::Trait(s.generics.len(), def_names, type_names),
        );
    }

    pub fn declare_stmt_struct(&mut self, s: &StmtStruct) {
        let name = s.name.clone();
        let field_names = s.fields.iter().map(|(x, _)| x.clone()).collect();
        self.bind(name.clone(), Binding::Struct(s.generics.len(), field_names));
    }

    pub fn declare_stmt_enum(&mut self, s: &StmtEnum) {
        let name = s.name.clone();
        let variant_names = s.variants.iter().map(|(x, _)| x.clone()).collect();
        self.bind(name.clone(), Binding::Enum(s.generics.len(), variant_names));
    }

    pub fn declare_stmt_type(&mut self, s: &StmtType) {
        let name = s.name.clone();
        self.bind(name.clone(), Binding::TypeAlias(s.generics.len()));
    }

    pub fn stmt(&mut self, s: &Stmt) -> Stmt {
        match s {
            Stmt::Var(s) => Stmt::Var(self.stmt_var(s)),
            Stmt::Def(s) => Stmt::Def(self.stmt_def(s)),
            Stmt::Trait(s) => Stmt::Trait(self.stmt_trait(s)),
            Stmt::Impl(s) => Stmt::Impl(self.stmt_impl(s)),
            Stmt::Struct(s) => Stmt::Struct(self.stmt_struct(s)),
            Stmt::Enum(s) => Stmt::Enum(self.stmt_enum(s)),
            Stmt::Type(s) => Stmt::Type(self.stmt_type(s)),
            Stmt::Expr(s) => Stmt::Expr(self.expr(s)),
            Stmt::Err(_) => todo!(),
        }
    }

    pub fn stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        let span = s.span;
        let name = s.name.clone();
        let ty = self.ty(&s.ty);
        let expr = self.expr(&s.expr);
        self.bind(name.clone(), Binding::Var);
        StmtVar::new(span, name, ty, expr)
    }

    pub fn stmt_def(&mut self, s: &StmtDef) -> StmtDef {
        let span = s.span;
        let name = s.name.clone();
        self.scoped_block(|ctx| {
            let generics = s.generics.clone();
            generics
                .iter()
                .for_each(|g| ctx.bind(g.clone(), Binding::Generic));
            let preds = s.where_clause.iter().map(|p| ctx.bound(p)).collect();
            let params = s.params.iter().map(|p| ctx.param(p)).collect();
            let ty = ctx.ty(&s.ty);
            let body = ctx.body(&s.body);
            StmtDef::new(span, name, generics, preds, params, ty, body)
        })
    }

    pub fn body(&mut self, body: &Body) -> Body {
        match body {
            Body::Expr(e) => Body::Expr(self.expr(e)),
            Body::Builtin => Body::Builtin,
        }
    }

    pub fn stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        let span = s.span;
        let name = s.name.clone();
        self.scoped_block(|ctx| {
            let generics = s.generics.clone();
            generics
                .iter()
                .for_each(|g| ctx.bind(g.clone(), Binding::Generic));
            let body = s.bounds.iter().map(|p| ctx.bound(p)).collect();
            let defs = s.defs.iter().map(|d| ctx.trait_def(d)).collect();
            let types = s.types.clone();
            StmtTrait::new(span, name, generics, body, defs, types)
        })
    }

    pub fn trait_def(&mut self, s: &TraitDef) -> TraitDef {
        let span = s.span;
        let name = s.name.clone();
        let generics = s.generics.clone();
        let preds = s.bounds.iter().map(|p| self.bound(p)).collect();
        let params = s.params.iter().map(|p| self.param(p)).collect();
        let ty = self.ty(&s.ty);
        TraitDef::new(span, name, generics, preds, params, ty)
    }

    pub fn stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        let span = s.span;
        let generics = s.generics.clone();
        self.scoped_block(|ctx| {
            for generic in &generics {
                ctx.bind(generic.clone(), Binding::Generic);
            }
            let defs = s
                .defs
                .iter()
                .map(|d| (d.name.clone(), d.generics.len()))
                .collect();
            let types = s
                .types
                .iter()
                .map(|t| (t.name.clone(), t.generics.len()))
                .collect();
            let head = ctx.head(&s.head, Some(&(defs, types)));
            let body = s
                .where_clause
                .iter()
                .map(|p| ctx.bound(p))
                .collect::<Vec<_>>();
            let defs = s.defs.iter().map(|d| ctx.stmt_def(d)).collect();
            let types = s.types.iter().map(|t| ctx.stmt_type(t)).collect();
            StmtImpl::new(span, generics, head, body, defs, types)
        })
    }

    pub fn stmt_struct(&mut self, s: &StmtStruct) -> StmtStruct {
        let span = s.span;
        let name = s.name.clone();
        self.scoped_block(|ctx| {
            let generics = s.generics.clone();
            for generic in &generics {
                ctx.bind(generic.clone(), Binding::Generic);
            }
            let fields = s
                .fields
                .iter()
                .map(|(x, t)| (x.clone(), ctx.ty(t)))
                .collect();
            StmtStruct::new(span, name.clone(), generics, fields)
        })
    }

    pub fn stmt_enum(&mut self, s: &StmtEnum) -> StmtEnum {
        let span = s.span;
        let name = s.name.clone();
        self.scoped_block(|ctx| {
            let generics = s.generics.clone();
            for generic in &generics {
                ctx.bind(generic.clone(), Binding::Generic);
            }
            let variants = s
                .variants
                .iter()
                .map(|(x, t)| (x.clone(), ctx.ty(t)))
                .collect();
            StmtEnum::new(span, name.clone(), generics, variants)
        })
    }

    pub fn stmt_type(&mut self, s: &StmtType) -> StmtType {
        let span = s.span;
        let name = s.name.clone();
        self.scoped_block(|ctx| {
            let generics = s.generics.clone();
            generics
                .iter()
                .for_each(|g| ctx.bind(g.clone(), Binding::Generic));
            let ty = ctx.ty(&s.ty);
            StmtType::new(span, name, generics, ty)
        })
    }

    pub fn param(&mut self, s: &Param) -> Param {
        let span = s.span;
        let name = s.name.clone();
        let ty = self.ty(&s.ty);
        self.bind(name.clone(), Binding::Var);
        Param::new(span, name, ty)
    }

    pub fn path(&mut self, p: &UnresolvedPath) -> UnresolvedPath {
        let segments = p
            .segments
            .iter()
            .map(|(x, ts)| (x.clone(), ts.iter().map(|t| self.ty(t)).collect::<Vec<_>>()))
            .collect();
        UnresolvedPath::new(segments)
    }

    pub fn ty(&mut self, t: &Type) -> Type {
        match t {
            Type::Cons(..) => unreachable!(),
            Type::Alias(..) => unreachable!(),
            Type::Unresolved(p) => self.resolve_type_path(p),
            Type::Assoc(..) => unreachable!(),
            Type::Hole => Type::Hole,
            Type::Var(_) => unreachable!(),
            Type::Err => Type::Err,
            Type::Generic(x) => Type::Generic(x.clone()),
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
                let xts = xts.iter().map(|(x, t)| (x.clone(), self.ty(t))).collect();
                Type::Record(xts)
            }
            Type::Array(t, n) => {
                let t = self.ty(t);
                let n = *n;
                Type::Array(Rc::new(t), n)
            }
            Type::Never => Type::Never,
        }
    }

    pub fn expr(&mut self, e: &Expr) -> Expr {
        let t = self.ty(e.ty());
        let s = e.span();
        match e {
            Expr::Def(..) => unreachable!(),
            Expr::Var(..) => unreachable!(),
            Expr::Unresolved(_s, _t, path) => self.resolve_expr_path(s, t, path),
            Expr::Int(s, t, v) => {
                let t = self.ty(t);
                let v = v.clone();
                Expr::Int(*s, t, v)
            }
            Expr::Float(s, t, v) => {
                let t = self.ty(t);
                let v = v.clone();
                Expr::Float(*s, t, v)
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
            Expr::Struct(..) => unreachable!(),
            Expr::Enum(..) => unreachable!(),
            Expr::Call(s, t, e, es) => self.call(*s, t, e, es),
            Expr::String(_, _, v) => {
                let v = v.clone();
                Expr::String(s, t, v)
            }
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                let x = x.clone();
                Expr::Field(s, t, Rc::new(e), x)
            }
            Expr::Block(_, _, ss, e) => {
                let ss = ss.iter().map(|s| self.stmt(s)).collect();
                let e = self.expr(e);
                Expr::Block(s, t, ss, Rc::new(e))
            }
            Expr::Query(..) => {
                todo!()
            }
            Expr::Assoc(..) => unreachable!(),
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
                let xps = xps
                    .iter()
                    .map(|(p, e)| (self.pat(p), self.expr(e)))
                    .collect();
                Expr::Match(s, t, Rc::new(e), xps)
            }
            Expr::While(_, _, e0, e1) => {
                let e0 = self.expr(e0);
                let e1 = self.expr(e1);
                Expr::While(s, t, Rc::new(e0), Rc::new(e1))
            }
            Expr::Record(_, _, xes) => {
                let xes = xes.iter().map(|(x, e)| (x.clone(), self.expr(e))).collect();
                Expr::Record(s, t, xes)
            }
            Expr::Value(_, _) => unreachable!(),
            Expr::Infix(_, _, _, _, _) => unreachable!(),
            Expr::Postfix(_, _, _, _) => unreachable!(),
            Expr::Prefix(_, _, _, _) => unreachable!(),
        }
    }

    fn call(&mut self, s: Span, t1: &Type, e: &Expr, es: &[Expr]) -> Expr {
        let t1 = self.ty(t1);
        if let Expr::Unresolved(_s, _t, path) = e {
            let path = self.path(path);
            let mut iter = path.segments.into_iter().peekable();
            let (x0, ts0) = iter.next().unwrap();
            match self.get(&x0) {
                Some(Binding::Struct(n, xs)) => {
                    if ts0.len() != *n && !ts0.is_empty() {
                        self.wrong_arity(&x0, ts0.len(), *n);
                        return Expr::Err(s, t1);
                    }
                    let ts0 = (ts0.is_empty())
                        .then(|| (0..*n).map(|_| Type::Hole).collect())
                        .unwrap_or(ts0);
                    if let Some((x1, _)) = iter.peek() {
                        self.unexpected_assoc("Struct", "field", &x0, x1);
                        return Expr::Err(s, t1);
                    }
                    let xs = xs.clone();
                    let xes: Vec<_> = es.iter().flat_map(|e| self.field(e)).collect();
                    if !Self::fields_are_defined(&xs, &xes) {
                        self.wrong_fields(&x0, &xes, &xs);
                        return Expr::Err(s, t1);
                    }
                    return Expr::Struct(s, t1, x0, ts0, xes);
                }
                Some(Binding::Enum(n, xs)) => {
                    if ts0.len() != *n && !ts0.is_empty() {
                        self.wrong_arity(&x0, ts0.len(), *n);
                        return Expr::Err(s, t1);
                    }
                    let ts0 = (ts0.is_empty())
                        .then(|| (0..*n).map(|_| Type::Hole).collect())
                        .unwrap_or(ts0);
                    if iter.peek().is_none() {
                        self.expected_assoc("variant", &x0);
                        return Expr::Err(s, t1);
                    }
                    let (x1, ts1) = iter.next().unwrap();
                    if !xs.contains(&x1) {
                        self.unexpected_assoc("Enum", "variant", &x0, &x1);
                        return Expr::Err(s, t1);
                    }
                    if !ts1.is_empty() {
                        self.wrong_arity(&x1, ts1.len(), 0);
                        return Expr::Err(s, t1);
                    }
                    if let Some((x2, _)) = iter.next() {
                        self.unexpected_assoc("Enum", "item", &x1, &x2);
                        return Expr::Err(s, t1);
                    }
                    let es = es.iter().map(|e| self.expr(e)).collect::<Vec<_>>();
                    let e = match es.len() {
                        1 => es.into_iter().next().unwrap(),
                        _ => Expr::Tuple(s, Type::Hole, es),
                    };
                    return Expr::Enum(s, t1, x0, ts0, x1, Rc::new(e));
                }
                _ => {}
            }
        }
        let e = self.expr(e);
        let es = es.iter().map(|e| self.expr(e)).collect();
        Expr::Call(s, t1, Rc::new(e), es)
    }

    fn field(&mut self, e: &Expr) -> Option<(Name, Expr)> {
        let s = e.span();
        let t = self.ty(e.ty());
        match e {
            Expr::Field(_, _, e, x) => {
                let e = self.expr(e);
                let x = x.clone();
                Some((x, e))
            }
            Expr::Assign(_, _, e0, e1) => {
                let e1 = self.expr(e1);
                if let Expr::Unresolved(_, _, path) = &**e0 {
                    let mut iter = path.segments.iter();
                    let (x0, ts0) = iter.next().unwrap();
                    if !ts0.is_empty() {
                        self.wrong_arity(x0, ts0.len(), 0);
                        return None;
                    }
                    if iter.next().is_some() {
                        self.report.err(
                            x0.span,
                            format!("Found unexpected associated item `::{x0}`.",),
                            "Fields no associated items.",
                        );
                        return None;
                    }
                    Some((x0.clone(), e1))
                } else {
                    self.expected_name(e0);
                    None
                }
            }
            Expr::Unresolved(_, _, path) => {
                let mut iter = path.segments.iter();
                let (x0, ts0) = iter.next().unwrap();
                match self.get(x0) {
                    Some(Binding::Var) => {
                        if !ts0.is_empty() {
                            self.wrong_arity(x0, ts0.len(), 0);
                            return None;
                        }
                        if let Some((x1, _)) = iter.next() {
                            self.report.err(
                                x1.span,
                                format!("Found unexpected associated item `::{x1}`.",),
                                "Variables have no associated items.",
                            );
                            return None;
                        }
                        Some((x0.clone(), Expr::Var(s, t, x0.clone())))
                    }
                    Some(b) => {
                        self.unexpected(x0, b.name(), "variable");
                        None
                    }
                    None => {
                        self.not_found(x0, "variable");
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
            Pat::Var(_, _, x) => Pat::Var(s, t, x.clone()),
            Pat::Tuple(_, _, ts) => {
                let ts = ts.iter().map(|t| self.pat(t)).collect();
                Pat::Tuple(s, t, ts)
            }
            Pat::Struct(..) => unreachable!(),
            Pat::Enum(..) => unreachable!(),
            Pat::Int(_, _, v) => Pat::Int(s, t, v.clone()),
            Pat::String(_, _, v) => Pat::String(s, t, v.clone()),
            Pat::Wildcard(_, _) => Pat::Wildcard(s, t),
            Pat::Bool(_, _, v) => Pat::Bool(s, t, *v),
            Pat::Err(_, _) => Pat::Err(s, t),
        }
    }

    fn unnamed_pat_args(&mut self, args: Option<Vec<PatArg>>, fieldless: bool) -> Option<Vec<Pat>> {
        if args.is_none() && fieldless {
            return Some(vec![]);
        }
        let args = args.unwrap();
        let mut ps = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                PatArg::Named(x, p) => {
                    self.report.err(
                        x.span,
                        format!("Expected `<pat>`, found `{x} = {p}`.",),
                        "Expected unnamed pattern `<pat>`.",
                    );
                    return None;
                }
                PatArg::Unnamed(p) => ps.push(p.clone()),
            }
        }
        Some(ps)
    }

    fn named_pat_args(&mut self, args: Option<Vec<PatArg>>) -> Option<Vec<(Name, Pat)>> {
        if args.is_none() {
            return Some(vec![]);
        }
        let args = args.unwrap();
        let mut xps = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                PatArg::Named(x, p) => xps.push((x.clone(), p.clone())),
                PatArg::Unnamed(p) => {
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

    #[allow(clippy::type_complexity)]
    pub fn head(
        &mut self,
        bound: &Bound,
        expected: Option<&(Vec<(Name, usize)>, Vec<(Name, usize)>)>,
    ) -> Bound {
        let span = bound.span();
        match bound {
            Bound::Unresolved(_, path) => self.resolve_bound_path(span, path, expected),
            Bound::Trait(_, _, _) => unreachable!(),
            Bound::Err(_) => Bound::Err(span),
        }
    }

    pub fn bound(&mut self, bound: &Bound) -> Bound {
        self.head(bound, None)
    }

    fn lvalue(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Var(s, t, x) => Expr::Var(*s, t.clone(), x.clone()),
            Expr::Field(s, t, e, x) => {
                let e = self.lvalue(e.as_ref());
                Expr::Field(*s, t.clone(), Rc::new(e), x.clone())
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

    fn fields_are_defined<T>(expected: &[Name], provided: &[(Name, T)]) -> bool {
        provided.iter().all(|(x, _)| expected.contains(x))
            && expected
                .iter()
                .all(|x| provided.iter().any(|(y, _)| x == y))
    }

    fn items_are_defined(expected: &[(Name, usize)], provided: &[(Name, usize)]) -> bool {
        provided
            .iter()
            .all(|(x0, n0)| expected.iter().any(|(x1, n1)| x0 == x1 && n0 == n1))
            && expected
                .iter()
                .all(|(x0, n0)| provided.iter().any(|(x1, n1)| x0 == x1 && n0 == n1))
    }

    #[allow(clippy::type_complexity)]
    fn resolve_bound_path(
        &mut self,
        span: Span,
        path: &UnresolvedPath,
        expected: Option<&(Vec<(Name, usize)>, Vec<(Name, usize)>)>,
    ) -> Bound {
        let path = self.path(path);
        let mut iter = path.segments.into_iter().peekable();
        let (x0, ts0) = iter.next().unwrap();
        match self.get(&x0) {
            Some(Binding::Trait(n, expected_defs, expected_types)) => {
                if *n != ts0.len() {
                    self.wrong_arity(&x0.clone(), ts0.len(), *n);
                    return Bound::Err(span);
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Trait", "item", &x0, x1);
                    return Bound::Err(span);
                }
                if let Some((found_defs, found_types)) = expected {
                    if !Self::items_are_defined(expected_defs, found_defs) {
                        let expected_defs = expected_defs.clone();
                        self.wrong_items("def", &x0, found_defs, &expected_defs);
                        return Bound::Err(span);
                    }
                    if !Self::items_are_defined(expected_types, found_types) {
                        let expected_types = expected_types.clone();
                        self.wrong_items("type", &x0, found_types, &expected_types);
                        return Bound::Err(span);
                    }
                }
                Bound::Trait(span, x0.clone(), ts0.clone())
            }
            Some(b) => {
                self.unexpected(&x0, b.name(), "trait");
                Bound::Err(span)
            }
            None => {
                self.not_found(&x0, "trait");
                Bound::Err(span)
            }
        }
    }

    fn resolve_expr_path(&mut self, s: Span, t: Type, path: &UnresolvedPath) -> Expr {
        let path = self.path(path);
        let mut iter = path.segments.into_iter().peekable();
        let (x0, ts0) = iter.next().unwrap();
        match self.get(&x0) {
            Some(Binding::Var) => {
                if !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), 0);
                    return Expr::Err(s, t.clone());
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Variable", "item", &x0, x1);
                    return Expr::Err(s, t.clone());
                }
                Expr::Var(s, t, x0.clone())
            }
            Some(Binding::Def(n)) => {
                if ts0.len() != *n && !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Expr::Err(s, t);
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Def", "item", &x0, x1);
                    return Expr::Err(s, t);
                }
                let ts0 = (ts0.is_empty())
                    .then(|| (0..*n).map(|_| Type::Hole).collect())
                    .unwrap_or(ts0);
                Expr::Def(s, t, x0.clone(), ts0.clone())
            }
            // Unit Struct
            Some(Binding::Struct(n, xs)) => {
                if ts0.len() != *n && !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Expr::Err(s, t.clone());
                }
                if !xs.is_empty() {
                    let xs = xs.clone();
                    self.wrong_fields::<Type>(&x0, &[], &xs);
                    return Expr::Err(s, t.clone());
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Struct", "item", &x0, x1);
                    return Expr::Err(s, t);
                }
                let ts0 = ts0
                    .is_empty()
                    .then(|| (0..*n).map(|_| Type::Hole).collect())
                    .unwrap_or(ts0);
                let t = self.ty(&t);
                let x0 = x0.clone();
                Expr::Struct(s, t, x0, ts0.clone(), vec![])
            }
            Some(Binding::Trait(n, def_names, _)) => {
                if *n != ts0.len() && !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Expr::Err(s, t);
                }
                if iter.peek().is_none() {
                    self.expected_assoc("type", &x0);
                    return Expr::Err(s, t);
                }
                let (x1, ts1) = iter.peek().unwrap();
                let result = def_names.iter().find(|(x2, _)| x2 == x1);
                if result.is_none() {
                    self.unexpected_assoc("Trait", "type", &x0, x1);
                    return Expr::Err(s, t);
                }
                let (_, n2) = result.unwrap();
                if ts1.len() != *n2 && !ts1.is_empty() {
                    self.wrong_arity(x1, ts1.len(), *n2);
                    return Expr::Err(s, t);
                }
                let ts0 = ts0
                    .is_empty()
                    .then(|| (0..*n).map(|_| Type::Hole).collect())
                    .unwrap_or(ts0);
                Expr::Assoc(s, t, x0.clone(), ts0.clone(), x1.clone(), ts1.clone())
            }
            Some(b) => {
                self.unexpected(&x0, b.name(), "expression");
                Expr::Err(s, t.clone())
            }
            None => {
                for (tr_x, tr_n, tr_def_xs, _) in self.stack.traits() {
                    for (tr_def_x, tr_def_n) in tr_def_xs {
                        if &x0 == tr_def_x && (ts0.len() == *tr_def_n || ts0.is_empty()) {
                            let ts0 = ts0
                                .is_empty()
                                .then(|| (0..*tr_def_n).map(|_| Type::Hole).collect())
                                .unwrap_or(ts0);
                            let tr_ts = (0..tr_n).map(|_| Type::Hole).collect::<Vec<_>>();
                            return Expr::Assoc(s, t, tr_x.clone(), tr_ts, tr_def_x.clone(), ts0);
                        }
                    }
                }
                self.not_found(&x0, "expression");
                Expr::Err(s, t.clone())
            }
        }
    }

    fn resolve_type_path(&mut self, p: &UnresolvedPath) -> Type {
        let p = self.path(p);
        let mut iter = p.segments.into_iter().peekable();
        let (x0, ts0) = iter.next().unwrap();
        match self.get(&x0) {
            Some(Binding::Generic) => {
                if !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), 0);
                    return Type::Err;
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Generic", "item", &x0, x1);
                    return Type::Err;
                }
                Type::Generic(x0.clone())
            }
            Some(Binding::Enum(n, _) | Binding::Struct(n, _) | Binding::BuiltinType(n)) => {
                if ts0.len() != *n {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Type::Err;
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Type", "item", &x0, x1);
                    return Type::Err;
                }
                Type::Cons(x0.clone(), ts0.clone())
            }
            Some(Binding::TypeAlias(n)) => {
                if *n != ts0.len() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Type::Err;
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Type", "item", &x0, x1);
                    return Type::Err;
                }
                Type::Alias(x0.clone(), ts0.clone())
            }
            Some(Binding::Trait(n, _, type_names)) => {
                if *n != ts0.len() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Type::Err;
                }
                if iter.peek().is_none() {
                    self.expected_assoc("type", &x0);
                    return Type::Err;
                }
                let (x1, ts1) = iter.next().unwrap();
                let result = type_names.iter().find(|(x2, _)| x2 == &x1);
                if result.is_none() {
                    self.unexpected_assoc("Trait", "type", &x0, &x1);
                    return Type::Err;
                }
                let (_, n2) = result.unwrap();
                if ts1.len() != *n2 {
                    self.wrong_arity(&x1, ts1.len(), *n2);
                    return Type::Err;
                }
                Type::Assoc(x0.clone(), ts0.clone(), x1.clone(), ts1.clone())
            }
            Some(b) => {
                self.unexpected(&x0, b.name(), "type");
                Type::Err
            }
            None => {
                self.not_found(&x0, "type");
                Type::Err
            }
        }
    }

    fn resolve_pat_path(
        &mut self,
        s: Span,
        t: Type,
        path: &UnresolvedPath,
        args: &Option<Vec<PatArg>>,
    ) -> Pat {
        let path = self.path(path);
        let mut iter = path.segments.into_iter().peekable();
        let (x0, ts0) = iter.next().unwrap();
        match self.get(&x0) {
            Some(Binding::Enum(n, xs)) => {
                if ts0.len() != *n && !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Pat::Err(s, t.clone());
                }
                let ts0 = ts0
                    .is_empty()
                    .then(|| (0..*n).map(|_| Type::Hole).collect())
                    .unwrap_or(ts0);
                if iter.peek().is_none() {
                    self.expected_assoc("variant", &x0);
                    return Pat::Err(s, t.clone());
                }
                let (x1, ts1) = iter.next().unwrap();
                if !xs.contains(&x1) {
                    self.unexpected_assoc("Enum", "variant", &x0, &x1);
                    return Pat::Err(s, t.clone());
                }
                if !ts1.is_empty() {
                    self.wrong_arity(&x1, ts1.len(), 0);
                    return Pat::Err(s, t.clone());
                }
                if let Some((x2, _)) = iter.peek() {
                    self.unexpected_assoc("Enum", "item", &x1, x2);
                    return Pat::Err(s, t.clone());
                }
                let args = args.clone();
                let xs = xs.clone();
                let result = self.unnamed_pat_args(args, xs.is_empty());
                if result.is_none() {
                    return Pat::Err(s, t.clone());
                }
                let ps = result.unwrap();
                let p = match ps.len() {
                    1 => ps.into_iter().next().unwrap(),
                    _ => Pat::Tuple(s, Type::Hole, ps),
                };
                Pat::Enum(s, t, x0.clone(), ts0.clone(), x1.clone(), Rc::new(p))
            }
            Some(Binding::Struct(n, xs)) => {
                if ts0.len() != *n && !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), *n);
                    return Pat::Err(s, t.clone());
                }
                let ts0 = ts0
                    .is_empty()
                    .then(|| (0..*n).map(|_| Type::Hole).collect())
                    .unwrap_or(ts0);
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Struct", "item", &x0, x1);
                    return Pat::Err(s, t.clone());
                }
                let args = args.clone();
                let xs = xs.clone();
                let xps = self.named_pat_args(args);
                if xps.is_none() {
                    return Pat::Err(s, t.clone());
                }
                let xps = xps.unwrap();
                if !Self::fields_are_defined(&xs, &xps) {
                    self.wrong_fields(&x0, &xps, &xs);
                    return Pat::Err(s, t.clone());
                }
                Pat::Struct(s, t, x0.clone(), ts0.clone(), xps)
            }
            Some(b) => {
                self.unexpected(&x0, b.name(), "pattern");
                Pat::Err(s, t.clone())
            }
            None => {
                if !ts0.is_empty() {
                    self.wrong_arity(&x0, ts0.len(), 0);
                    return Pat::Err(s, t.clone());
                }
                if let Some((x1, _)) = iter.peek() {
                    self.unexpected_assoc("Struct", "item", &x0, x1);
                    return Pat::Err(s, t.clone());
                }
                Pat::Var(s, t, x0.clone())
            }
        }
    }
}

impl Binding {
    fn name(&self) -> &'static str {
        match self {
            Binding::Enum(_, _) => "enum",
            Binding::Struct(_, _) => "struct",
            Binding::BuiltinType(_) => "type",
            Binding::Generic => "generic",
            Binding::TypeAlias(..) => "type alias",
            Binding::Trait(..) => "trait",
            Binding::Var => "variable",
            Binding::Def(..) => "definition",
            Binding::BuiltinDef(_) => "builtin def",
        }
    }
}
