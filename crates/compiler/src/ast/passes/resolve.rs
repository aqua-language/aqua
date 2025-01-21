use std::rc::Rc;

use crate::ast::passes::Pass;
use crate::ast::Ast;
use crate::ast::Expr;
use crate::ast::Impl;
use crate::ast::Local;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Path;
use crate::ast::PathPatField;
use crate::ast::Segment;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtTraitType;
use crate::ast::StmtType;
use crate::ast::Trait;
use crate::ast::Type;
use crate::ast::TypeBody;
use crate::collections::map::Map;
use crate::report::Diagnostic;
use crate::report::Report;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

#[derive(Debug)]
pub struct Stack(Vec<Scope>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scope {
    bindings: Vec<(Name, Binding)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Binding {
    Enum(Rc<StmtEnum>),
    Struct(Rc<StmtStruct>),
    Type(Rc<StmtType>),
    Trait(Rc<StmtTrait>),
    Def(Rc<StmtDef>),
    Local(bool),
    Generic,
}

impl Context {
    fn bind(&mut self, name: Name, binding: Binding) {
        let last = self.stack.0.last_mut().unwrap();

        for (name2, _) in last.bindings.iter() {
            if name2 == &name {
                self.report.add(Diagnostic::err2(
                    name2.span,
                    name.span,
                    format!("Name `{}` already defined in scope.", name),
                    "First definition",
                    "Second definition",
                ));
                return;
            }
        }

        last.bindings.push((name, binding));
    }

    fn get(&self, x: &Name) -> Option<Binding> {
        self.stack
            .0
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.iter().find(|(n, _)| n == x))
            .map(|(_, b)| b.clone())
    }
}

#[derive(Debug)]
pub struct Context {
    stack: Stack,
    // Used to track the number of type alias expansions.
    pub report: Report,
}

impl Pass for Context {
    fn run(&mut self, program: &Ast) -> Ast {
        self.map_program(program)
    }

    fn report(&mut self) -> &mut Report {
        &mut self.report
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for Context {
    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Local(_) => {}
            Stmt::Def(s) => self.bind(s.name, Binding::Def(s.clone())),
            Stmt::Trait(s) => self.bind(s.name, Binding::Trait(s.clone())),
            Stmt::Impl(_) => {}
            Stmt::Struct(s) => self.bind(s.name, Binding::Struct(s.clone())),
            Stmt::Enum(s) => self.bind(s.name, Binding::Enum(s.clone())),
            Stmt::Type(s) => self.bind(s.name, Binding::Type(s.clone())),
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }
}

impl Mapper for Context {
    fn enter_scope(&mut self) {
        self.stack.0.push(Scope::default());
    }
    fn exit_scope(&mut self) {
        self.stack.0.pop();
    }

    fn map_generic(&mut self, g: &Name) -> Name {
        self.bind(*g, Binding::Generic);
        self._map_generic(g)
    }

    fn map_local(&mut self, l: &Local) -> Local {
        self.bind(l.name, Binding::Local(l.mutable));
        self._map_local(l)
    }

    fn map_stmt_trait(&mut self, s: &StmtTrait) -> StmtTrait {
        self.enter_scope();
        let span = self.map_span(&s.span);
        let name = self.map_name(&s.name);
        let generics = self.map_generics(&s.generics);
        let where_clause = self.map_impls(&s.where_clause);
        let defs = self.map_trait_defs(&s.defs);
        let types = self.map_trait_types(&s.types);
        self.exit_scope();
        StmtTrait::new(span, name, generics, where_clause, defs, types)
    }

    fn map_stmt_impl(&mut self, s: &StmtImpl) -> StmtImpl {
        self.enter_scope();
        let span = s.span;
        let generics = self.map_generics(&s.generics);
        let head = self.head(&s.head, &s.defs, &s.types);
        let where_clause = self.map_impls(&s.where_clause);
        let defs = self.map_rc_iter(&s.defs, Self::map_stmt_def);
        let types = self.map_rc_iter(&s.types, Self::map_stmt_type);
        self.exit_scope();
        StmtImpl::new(span, generics, head, where_clause, defs, types)
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Path(s, t, path) => {
                let t = self.map_type(&t);
                let path = self.map_path(path);
                let mut iter = path.segments.into_iter();
                let seg0 = iter.next().unwrap();
                match self.get(&seg0.x) {
                    Some(Binding::Local(m)) => {
                        if !seg0.ts.is_empty() {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), 0);
                            return Expr::Err(*s, t.clone());
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Variable", "item", &seg0.x, &seg1.x);
                            return Expr::Err(*s, t.clone());
                        }
                        Expr::Local(*s, t, seg0.x, m)
                    }
                    Some(Binding::Def(stmt)) => {
                        if seg0.ts.len() != stmt.generics.len() && !seg0.ts.is_empty() {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Expr::Err(*s, t);
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Def", "item", &seg0.x, &seg1.x);
                            return Expr::Err(*s, t);
                        }
                        let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                        Expr::Def(*s, t, seg0.x, ts0.clone())
                    }
                    // Unit Struct
                    Some(Binding::Struct(stmt)) => {
                        if !seg0.has_optional_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Expr::Err(*s, t.clone());
                        }
                        if !stmt.fields.is_empty() {
                            self.wrong_fields::<Type, _>(&seg0.x, None, &stmt.fields);
                            return Expr::Err(*s, t.clone());
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Struct", "item", &seg0.x, &seg1.x);
                            return Expr::Err(*s, t);
                        }
                        let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                        let x0 = seg0.x;
                        Expr::Struct(*s, t, x0, ts0.clone(), Map::new())
                    }
                    Some(Binding::Trait(stmt)) => {
                        if !seg0.has_optional_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Expr::Err(*s, t);
                        }
                        if seg0.has_named_args() {
                            self.unexpected_named_type_args(&seg0.x);
                            return Expr::Err(*s, t);
                        }
                        let Some(seg1) = iter.next() else {
                            self.expected_assoc("function", &seg0.x);
                            return Expr::Err(*s, t);
                        };
                        let Some(def) = stmt.defs.iter().find(|def| def.name == seg1.x) else {
                            self.unexpected_assoc("Trait", "function", &seg0.x, &seg1.x);
                            return Expr::Err(*s, t);
                        };
                        if !seg1.has_optional_arity(def.generics.len()) {
                            self.wrong_arity(&seg1.x, seg1.ts.len(), def.generics.len());
                            return Expr::Err(*s, t);
                        }
                        let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                        let ts1 = seg1.create_unnamed_holes(def.generics.len());
                        let b = Impl::Trait(Trait::new(seg0.x, ts0));
                        Expr::Assoc(*s, t, b, seg1.x, ts1)
                    }
                    Some(Binding::Type(stmt)) => {
                        if !seg0.has_optional_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Expr::Err(*s, t.clone());
                        }
                        let x0 = seg0.x;
                        let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                        let Some(seg1) = iter.next() else {
                            self.expected_assoc("function", &seg0.x);
                            return Expr::Err(*s, t);
                        };
                        let ts1 = seg1.ts;
                        let x1 = seg1.x;
                        let b = Impl::Type(Rc::new(Type::Builtin(x0, ts0)));
                        Expr::Assoc(*s, t, b, x1, ts1)
                    }
                    Some(b) => {
                        self.unexpected(&seg0.x, b.name(), "expression");
                        Expr::Err(*s, t.clone())
                    }
                    None => {
                        self.not_found(&seg0.x, "expression");
                        Expr::Err(*s, t.clone())
                    }
                }
            }
            Expr::Call(s, t, e, es) => {
                let t = self.map_type(t);
                if let Expr::Path(_s, _t, path) = e.as_ref() {
                    let path = self.map_path(&path);
                    let mut iter = path.segments.into_iter();
                    let seg0 = iter.next().unwrap();
                    match self.get(&seg0.x) {
                        Some(Binding::Struct(stmt)) => {
                            let Some(ts0) = seg0.try_create_unnamed_holes(stmt.generics.len())
                            else {
                                self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                                return Expr::Err(*s, t);
                            };
                            if seg0.has_named_args() {
                                self.unexpected_named_type_args(&seg0.x);
                                return Expr::Err(*s, t);
                            }
                            if let Some(seg1) = iter.next() {
                                self.unexpected_assoc("Struct", "field", &seg0.x, &seg1.x);
                                return Expr::Err(*s, t);
                            }
                            let xes: Map<_, _> =
                                es.iter().flat_map(|e| self.expr_field(e)).collect();
                            if !fields_are_defined(&stmt.fields, &xes) {
                                self.wrong_fields(&seg0.x, Some(&xes), &stmt.fields);
                                return Expr::Err(*s, t);
                            }
                            return Expr::Struct(*s, t, seg0.x, ts0, xes);
                        }
                        Some(Binding::Enum(stmt)) => {
                            if !seg0.has_optional_arity(stmt.generics.len()) {
                                self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                                return Expr::Err(*s, t);
                            }
                            if seg0.has_named_args() {
                                self.unexpected_named_type_args(&seg0.x);
                                return Expr::Err(*s, t);
                            }
                            let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                            let Some(seg1) = iter.next() else {
                                self.expected_assoc("variant", &seg0.x);
                                return Expr::Err(*s, t);
                            };
                            if !stmt.variants.contains_key(&seg1.x) {
                                self.unexpected_assoc("Enum", "variant", &seg0.x, &seg1.x);
                                return Expr::Err(*s, t);
                            }
                            if !seg1.ts.is_empty() {
                                self.wrong_arity(&seg1.x, seg1.ts.len(), 0);
                                return Expr::Err(*s, t);
                            }
                            if let Some(seg2) = iter.next() {
                                self.unexpected_assoc("Enum", "item", &seg1.x, &seg2.x);
                                return Expr::Err(*s, t);
                            }
                            let es = es.iter().map(|e| self.map_expr(e)).collect::<Vec<_>>();
                            let e = match es.len() {
                                1 => es.into_iter().next().unwrap(),
                                _ => Expr::Tuple(*s, Type::Unknown, es),
                            };
                            return Expr::Enum(*s, t, seg0.x, ts0, seg1.x, Rc::new(e));
                        }
                        _ => {}
                    }
                }
                let e = self.map_expr(e);
                let es = self.map_exprs(es);
                Expr::Call(*s, t, Rc::new(e), es)
            }
            // x = e;
            // x.y = e;
            // x[i] = e;
            Expr::Assign(s, t, e0, e1) => {
                let t = self.map_type(t);
                let e0 = self.map_expr(e0);
                let e0 = if e0.is_place() {
                    e0
                } else {
                    self.report.add(Diagnostic::err(
                        e.span(),
                        "Invalid left-hand side of assignment",
                        "Expected a variable, index, or field expression.",
                    ));
                    Expr::Err(e.span(), e.ty().clone())
                };
                let e1 = self.map_expr(e1);
                Expr::Assign(*s, t, Rc::new(e0), Rc::new(e1))
            }
            _ => self._map_expr(e),
        }
    }

    fn map_top_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        self.visit_stmts(stmts);
        self._map_stmts(stmts)
    }

    fn map_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        self.visit_stmts(stmts);
        self._map_stmts(stmts)
    }

    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Path(p) => {
                let p = self.map_path(p);
                let mut iter = p.segments.into_iter();
                let seg0 = iter.next().unwrap();
                match self.get(&seg0.x) {
                    Some(Binding::Generic) => {
                        if seg0.has_args() {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), 0);
                            return Type::Err;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Generic", "item", &seg0.x, &seg1.x);
                            return Type::Err;
                        }
                        Type::Generic(seg0.x)
                    }
                    Some(Binding::Enum(stmt)) => {
                        if !seg0.has_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Type::Err;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Type", "item", &seg0.x, &seg1.x);
                            return Type::Err;
                        }
                        Type::Enum(seg0.x, seg0.ts.clone())
                    }
                    Some(Binding::Struct(stmt)) => {
                        if !seg0.has_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Type::Err;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Type", "item", &seg0.x, &seg1.x);
                            return Type::Err;
                        }
                        Type::Struct(seg0.x, seg0.ts.clone())
                    }
                    Some(Binding::Type(stmt)) => {
                        if !seg0.has_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Type::Err;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Type", "item", &seg0.x, &seg1.x);
                            return Type::Err;
                        }
                        let stmt = stmt.instantiate(&seg0.ts.clone());
                        match stmt.body {
                            TypeBody::UserDefined(_) => Type::Alias(seg0.x, seg0.ts.clone()),
                            TypeBody::Builtin(_) => Type::Builtin(seg0.x, seg0.ts.clone()),
                        }
                    }
                    Some(Binding::Trait(stmt)) => {
                        if !seg0.has_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Type::Err;
                        }
                        let Some(seg1) = iter.next() else {
                            self.expected_assoc("type", &seg0.x);
                            return Type::Err;
                        };
                        let Some(t) = stmt.types.iter().find(|t| t.name == seg1.x) else {
                            self.unexpected_assoc("Trait", "type", &seg0.x, &seg1.x);
                            return Type::Err;
                        };
                        if !seg1.has_arity(t.generics.len()) {
                            self.wrong_arity(&seg1.x, seg1.ts.len(), t.generics.len());
                            return Type::Err;
                        }
                        let b = Impl::Trait(Trait::new(seg0.x, seg0.ts.clone()));
                        Type::Assoc(b, seg1.x, seg1.ts.clone())
                    }
                    Some(b) => {
                        self.unexpected(&seg0.x, b.name(), "type");
                        Type::Err
                    }
                    None => {
                        self.not_found(&seg0.x, "type");
                        Type::Err
                    }
                }
            }
            Type::Generic(x) => Type::Generic(*x),
            _ => self._map_type(t),
        }
    }

    fn map_pattern(&mut self, p: &Pat) -> Pat {
        match p {
            Pat::Path(s, t, path, args) => {
                let t = self.map_type(t);
                let path = self.map_path(path);
                let mut iter = path.segments.into_iter();
                let seg0 = iter.next().unwrap();
                match self.get(&seg0.x) {
                    Some(Binding::Enum(stmt)) => {
                        if !seg0.has_optional_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Pat::Err(*s, t.clone());
                        }
                        let ts0 = Self::create_type_arg_holes(seg0.ts, stmt.generics.len());
                        let Some(seg1) = iter.next() else {
                            self.expected_assoc("variant", &seg0.x);
                            return Pat::Err(*s, t.clone());
                        };
                        if !stmt.variants.contains_key(&seg1.x) {
                            self.unexpected_assoc("Enum", "variant", &seg0.x, &seg1.x);
                            return Pat::Err(*s, t.clone());
                        }
                        if !seg1.ts.is_empty() {
                            self.wrong_arity(&seg1.x, seg1.ts.len(), 0);
                            return Pat::Err(*s, t.clone());
                        }
                        if let Some(seg2) = iter.next() {
                            self.unexpected_assoc("Enum", "item", &seg1.x, &seg2.x);
                            return Pat::Err(*s, t.clone());
                        }
                        let Some(ps) = self.enum_pat_args(args, stmt.variants.is_empty()) else {
                            return Pat::Err(*s, t.clone());
                        };
                        let p = match ps.len() {
                            1 => ps.into_iter().next().unwrap(),
                            _ => Pat::Tuple(*s, Type::Unknown, ps),
                        };
                        Pat::Enum(*s, t, seg0.x, ts0.clone(), seg1.x, Rc::new(p))
                    }
                    Some(Binding::Struct(stmt)) => {
                        if !seg0.has_arity(stmt.generics.len()) {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                            return Pat::Err(*s, t.clone());
                        }
                        let ts0 = Self::create_type_arg_holes(seg0.ts, stmt.generics.len());
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Struct", "item", &seg0.x, &seg1.x);
                            return Pat::Err(*s, t.clone());
                        }
                        let Some(xps) = self.struct_pat_args(args) else {
                            return Pat::Err(*s, t.clone());
                        };
                        if !fields_are_defined(&stmt.fields, &xps) {
                            self.wrong_fields(&seg0.x, Some(&xps), &stmt.fields);
                            return Pat::Err(*s, t.clone());
                        }
                        Pat::Struct(*s, t, seg0.x, ts0.clone(), xps)
                    }
                    Some(b) => {
                        self.unexpected(&seg0.x, b.name(), "pattern");
                        Pat::Err(*s, t.clone())
                    }
                    None => {
                        if !seg0.ts.is_empty() {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), 0);
                            return Pat::Err(*s, t.clone());
                        }
                        if let Some(seg) = iter.next() {
                            self.unexpected_assoc("Struct", "item", &seg0.x, &seg.x);
                            return Pat::Err(*s, t.clone());
                        }
                        Pat::Local(*s, t, seg0.x, false)
                    }
                }
            }
            Pat::Local(_, _, x, m) => {
                let t = self.map_type(p.ty());
                let s = p.span();
                Pat::Local(s, t, *x, *m)
            }
            _ => self._map_pattern(p),
        }
    }

    // impl ... where Foo[T] { ... }
    #[allow(clippy::type_complexity)]
    fn map_impl(&mut self, bound: &Impl) -> Impl {
        match bound {
            Impl::Path(_span, path) => self.resolve_bound_path(path),
            Impl::Trait(..) => unreachable!(),
            Impl::Type(..) => unreachable!(),
            Impl::Var(..) => unreachable!(),
            Impl::Unknown => Impl::Unknown,
            Impl::Err => Impl::Err,
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            stack: Stack(vec![Scope::default()]),
            report: Report::new(),
        }
    }

    fn not_found(&mut self, name: &Name, expected: &'static str) {
        self.report.add(Diagnostic::err(
            name.span,
            format!("Name `{}` not found.", name),
            format!("Expected {}.", expected),
        ));
    }

    fn unexpected(&mut self, name: &Name, found: &'static str, expected: &'static str) {
        self.report.add(Diagnostic::err(
            name.span,
            format!("Unexpected {found} `{name}`."),
            format!("Expected {expected}."),
        ));
    }

    #[track_caller]
    fn wrong_arity(&mut self, name: &Name, found: usize, expected: usize) {
        #[cfg(not(feature = "explicit"))]
        let file = "";
        #[cfg(feature = "explicit")]
        let file = format!("{}: ", std::panic::Location::caller());
        self.report.add(Diagnostic::err(
            name.span,
            format!("{file}Wrong number of type arguments. Found {found}, expected {expected}",),
            format!("Expected {} arguments.", expected),
        ));
    }

    fn expected_assoc(&mut self, kind: &'static str, x: &Name) {
        self.report.add(Diagnostic::err(
            x.span,
            format!("Expected an associated {kind} `{x}::<{kind}>`.",),
            format!("Expected an associated {kind}."),
        ));
    }

    fn unexpected_assoc(&mut self, kind0: &'static str, kind1: &'static str, x0: &Name, x1: &Name) {
        self.report.add(Diagnostic::err(
            x1.span,
            format!("Found unexpected associated {kind1} `{x0}::{x1}`.",),
            format!("{kind0} `{x0}` has no associated {kind1} `{x1}`.",),
        ));
    }

    fn unexpected_named_type_args(&mut self, x: &Name) {
        self.report.add(Diagnostic::err(
            x.span,
            format!("Unexpected named type arguments for `{x}`.",),
            "Named type arguments can only occur in trait bounds.",
        ));
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
            self.report.add(Diagnostic::err(
                name.span,
                format!(
                    "Wrong fields provided. Found {name}({found}), expected {name}({expected})",
                ),
                format!("Expected {name}({expected}) fields."),
            ));
        } else {
            self.report.add(Diagnostic::err(
                name.span,
                format!("Wrong fields provided. Found {name}, expected {name}({expected})",),
                format!("Expected {name}({expected}) fields."),
            ));
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
        self.report.add(Diagnostic::err(
            name.span,
            format!("Wrong {kind}s implemented for {name}. Found {{ {found} }}, expected {{ {expected} }}",),
            format!("Expected {{ {expected} }}."),
        ));
    }

    #[allow(dead_code)]
    fn wrong_variant<T>(&mut self, name: &Name, found: &(Name, T), expected: &[Name]) {
        let found = &found.0;
        let expected = comma_sep(expected.iter());
        self.report.add(Diagnostic::err(
            name.span,
            format!("Wrong variant provided. Found {found}, expected {expected}",),
            format!("Expected one of {{ {expected} }} variants."),
        ));
    }

    fn expected_name(&mut self, e: &Expr) {
        self.report.add(Diagnostic::err(
            e.span(),
            "Expected a field label.",
            "Only `<name> = <expr>` is allowed.",
        ));
    }

    fn create_type_arg_holes(args: Vec<Type>, expected: usize) -> Vec<Type> {
        args.is_empty()
            .then(|| (0..expected).map(|_| Type::Unknown).collect())
            .unwrap_or(args)
    }

    fn expr_field(&mut self, e: &Expr) -> Option<(Name, Expr)> {
        let s = e.span();
        let t = self.map_type(e.ty());
        match e {
            Expr::Field(_, _, e, x) => {
                let e = self.map_expr(e);
                Some((*x, e))
            }
            Expr::Assign(_, _, e0, e1) => {
                let e1 = self.map_expr(e1);
                if let Expr::Path(_, _, path) = &**e0 {
                    let mut iter = path.segments.iter();
                    let seg0 = iter.next().unwrap();
                    if !seg0.ts.is_empty() {
                        self.wrong_arity(&seg0.x, seg0.ts.len(), 0);
                        return None;
                    }
                    if let Some(seg1) = iter.next() {
                        self.unexpected_assoc("Assignment", "item", &seg0.x, &seg1.x);
                        return None;
                    }
                    Some((seg0.x, e1))
                } else {
                    self.expected_name(e0);
                    None
                }
            }
            Expr::Path(_, _, path) => {
                let mut iter = path.segments.iter();
                let seg0 = iter.next().unwrap();
                match self.get(&seg0.x) {
                    Some(Binding::Local(m)) => {
                        if !seg0.ts.is_empty() {
                            self.wrong_arity(&seg0.x, seg0.ts.len(), 0);
                            return None;
                        }
                        if let Some(seg1) = iter.next() {
                            self.unexpected_assoc("Variable", "item", &seg0.x, &seg1.x);
                            return None;
                        }
                        Some((seg0.x, Expr::Local(s, t, seg0.x, m)))
                    }
                    Some(b) => {
                        self.unexpected(&seg0.x, b.name(), "variable");
                        None
                    }
                    None => {
                        self.not_found(&seg0.x, "variable");
                        None
                    }
                }
            }
            _ => {
                self.report.add(Diagnostic::err(
                    e.span(),
                    "Not a field.",
                    "Expected `<name> = <expr>`, `<name>` or `<expr>.<name>`.",
                ));
                None
            }
        }
    }

    fn enum_pat_args(
        &mut self,
        args: &Option<Vec<PathPatField>>,
        is_unit_enum: bool,
    ) -> Option<Vec<Pat>> {
        if args.is_none() && is_unit_enum {
            return Some(vec![]);
        }
        let args = args.as_ref().unwrap();
        let mut ps = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                PathPatField::Named(x, p) => {
                    self.report.add(Diagnostic::err(
                        x.span,
                        format!("Expected `<pat>`, found `{x} = {p}`.",),
                        "Expected unnamed pattern `<pat>`.",
                    ));
                    return None;
                }
                PathPatField::Unnamed(p) => ps.push(p.clone()),
            }
        }
        Some(ps)
    }

    fn struct_pat_args(&mut self, args: &Option<Vec<PathPatField>>) -> Option<Map<Name, Pat>> {
        if args.is_none() {
            return Some(Map::new());
        }
        let args = args.as_ref().unwrap();
        let mut xps = Map::new();
        for arg in args {
            match arg {
                PathPatField::Named(x, p) => {
                    xps.insert(*x, p.clone());
                }
                PathPatField::Unnamed(p) => {
                    self.report.add(Diagnostic::err(
                        p.span(),
                        format!("Expected `{p} = <pat>`.",),
                        format!("Expected named pattern `{p} = <pat>`."),
                    ));
                    return None;
                }
            }
        }
        Some(xps)
    }

    // impl Foo[T] { ... }
    fn head(&mut self, head: &Impl, defs: &[Rc<StmtDef>], types: &[Rc<StmtType>]) -> Impl {
        match head {
            Impl::Path(_, path) => self.resolve_head_path(path, defs, types),
            Impl::Err => Impl::Err,
            Impl::Unknown => Impl::Unknown,
            Impl::Trait(..) => unreachable!(),
            Impl::Type(..) => unreachable!(),
            Impl::Var(..) => unreachable!(),
        }
    }

    #[allow(clippy::type_complexity)]
    fn resolve_head_path(
        &mut self,
        path: &Path,
        found_defs: &[Rc<StmtDef>],
        found_types: &[Rc<StmtType>],
    ) -> Impl {
        let path = self.map_path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.get(&seg0.x) {
            Some(Binding::Trait(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                    return Impl::Err;
                }
                let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                if seg0.has_named_args() {
                    self.unexpected_named_type_args(&seg0.x);
                    return Impl::Err;
                }
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Trait", "item", &seg0.x, &seg1.x);
                    return Impl::Err;
                }
                if !defs_are_defined(&stmt.defs, found_defs) {
                    self.wrong_items(
                        "def",
                        &seg0.x,
                        found_defs.iter().map(|def| &def.name),
                        stmt.defs.iter().map(|def| &def.name),
                    );
                    return Impl::Err;
                }
                if !types_are_defined(&stmt.types, found_types) {
                    self.wrong_items(
                        "type",
                        &seg0.x,
                        found_types.iter().map(|ty| &ty.name),
                        stmt.types.iter().map(|ty| &ty.name),
                    );
                    return Impl::Err;
                }
                Impl::Trait(Trait::new(seg0.x, ts0))
            }
            Some(Binding::Type(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                    return Impl::Err;
                }
                let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                let t = Type::Builtin(seg0.x, ts0.clone());
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Type", "item", &seg0.x, &seg1.x);
                    return Impl::Err;
                }
                Impl::Type(Rc::new(t))
            }
            Some(Binding::Struct(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                    return Impl::Err;
                }
                let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                let t = Type::Struct(seg0.x, ts0.clone());
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Struct", "item", &seg0.x, &seg1.x);
                    return Impl::Err;
                }
                Impl::Type(Rc::new(t))
            }
            Some(Binding::Enum(stmt)) => {
                if !seg0.has_optional_arity(stmt.generics.len()) {
                    self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                    return Impl::Err;
                }
                let ts0 = seg0.create_unnamed_holes(stmt.generics.len());
                let t = Type::Enum(seg0.x, ts0.clone());
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Enum", "item", &seg0.x, &seg1.x);
                    return Impl::Err;
                }
                Impl::Type(Rc::new(t))
            }
            Some(b) => {
                self.unexpected(&seg0.x, b.name(), "trait");
                Impl::Err
            }
            None => {
                self.not_found(&seg0.x, "trait");
                Impl::Err
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn resolve_bound_path(&mut self, path: &Path) -> Impl {
        let path = self.map_path(path);
        let mut iter = path.segments.into_iter();
        let seg0 = iter.next().unwrap();
        match self.get(&seg0.x) {
            Some(Binding::Trait(stmt)) => {
                let Some(ts0) = seg0.try_create_unnamed_holes(stmt.generics.len()) else {
                    self.wrong_arity(&seg0.x, seg0.ts.len(), stmt.generics.len());
                    return Impl::Err;
                };
                if let Some(seg1) = iter.next() {
                    self.unexpected_assoc("Trait", "item", &seg0.x, &seg1.x);
                    return Impl::Err;
                }
                Impl::Trait(Trait::new(seg0.x, ts0))
            }
            Some(b) => {
                self.unexpected(&seg0.x, b.name(), "trait");
                Impl::Err
            }
            None => {
                self.not_found(&seg0.x, "trait");
                Impl::Err
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
            Binding::Local(..) => "variable",
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

    fn try_create_unnamed_holes(&self, arity: usize) -> Option<Vec<Type>> {
        if self.ts.is_empty() {
            Some(vec![Type::Unknown; arity])
        } else if self.ts.len() == arity {
            Some(self.ts.clone())
        } else {
            None
        }
    }

    fn create_unnamed_holes(&self, n: usize) -> Vec<Type> {
        if self.ts.is_empty() {
            vec![Type::Unknown; n]
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
