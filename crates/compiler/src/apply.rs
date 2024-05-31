use std::rc::Rc;

use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::Pat;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::StmtTrait;
use crate::ast::StmtTraitDef;
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::Type;
use crate::ast::TypeVar;
use crate::infer::Context;

fn map<T>(ts: &[T], f: impl FnMut(&T) -> T) -> Vec<T> {
    ts.iter().map(f).collect()
}

impl Program {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Program {
        let stmts = self.stmts.iter().map(|s| s.map_type(f)).collect::<Vec<_>>();
        Program::new(stmts)
    }
}

impl Type {
    pub fn apply(&self, sub: &[(Name, Type)]) -> Type {
        match self {
            Type::Cons(x, ts) => {
                let ts = map(ts, |t| t.apply(sub));
                Type::Cons(*x, ts)
            }
            Type::Var(x, k) => sub
                .iter()
                .find(|(n, _)| n == x)
                .map(|(_, t)| t.apply(sub))
                .unwrap_or_else(|| Type::Var(*x, *k)),
            Type::Assoc(b, x1, ts1) => {
                let b = b.map_type(&mut |t| t.apply(sub));
                let ts1 = ts1.iter().map(|t| t.apply(sub)).collect::<Vec<_>>();
                Type::Assoc(b, *x1, ts1)
            }
            Type::Hole => Type::Hole,
            Type::Err => Type::Err,
            Type::Generic(x) => Type::Generic(*x),
            Type::Fun(ts, t) => {
                let ts = map(ts, |t| t.apply(sub));
                let t = t.apply(sub);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = map(ts, |t| t.apply(sub));
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, t.apply(sub))).collect();
                Type::Record(xts)
            }
            Type::Alias(x, ts) => {
                let ts = map(ts, |t| t.apply(sub));
                Type::Alias(*x, ts)
            }
            Type::Path(_) => unreachable!(),
            Type::Array(t, n) => {
                let t = Rc::new(t.apply(sub));
                let n = *n;
                Type::Array(t, n)
            }
            Type::Never => Type::Never,
        }
    }

    pub fn instantiate(&self, sub: &[(Name, Type)]) -> Type {
        match self {
            Type::Cons(x, ts) => {
                let ts = map(ts, |t| t.instantiate(sub));
                Type::Cons(*x, ts)
            }
            Type::Var(x, k) => Type::Var(*x, *k),
            Type::Assoc(b, x1, ts1) => {
                let b = b.map_type(&mut |b| b.instantiate(sub));
                let ts1 = ts1.iter().map(|t| t.instantiate(sub)).collect::<Vec<_>>();
                Type::Assoc(b, *x1, ts1)
            }
            Type::Hole => Type::Hole,
            Type::Generic(x) => sub
                .iter()
                .find(|(n, _)| n == x)
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| Type::Var(*x, TypeVar::General)),
            Type::Fun(ts, t) => {
                let ts = map(ts, |t| t.instantiate(sub));
                let t = t.instantiate(sub);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = map(ts, |t| t.instantiate(sub));
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, t.instantiate(sub))).collect();
                Type::Record(xts)
            }
            Type::Alias(x, ts) => {
                let ts = map(ts, |t| t.instantiate(sub));
                Type::Alias(*x, ts)
            }
            Type::Err => Type::Err,
            Type::Path(_) => unreachable!(),
            Type::Array(t, n) => {
                let t = Rc::new(t.instantiate(sub));
                let n = *n;
                Type::Array(t, n)
            }
            Type::Never => Type::Never,
        }
    }

    pub fn default(&self, int: &Type, float: &Type) -> Type {
        match self {
            Type::Path(_) => unreachable!(),
            Type::Cons(x, ts) => {
                let ts = map(ts, |t| t.default(int, float));
                Type::Cons(*x, ts)
            }
            Type::Alias(x, ts) => {
                let ts = map(ts, |t| t.default(int, float));
                Type::Alias(*x, ts)
            }
            Type::Assoc(b, x, ts) => {
                let b = b.map_type(&mut |b| b.default(int, float));
                let ts = map(ts, |t| t.default(int, float));
                Type::Assoc(b, *x, ts)
            }
            Type::Var(x, k) => match k {
                TypeVar::General => Type::Var(*x, TypeVar::General),
                TypeVar::Float => float.clone(),
                TypeVar::Int => int.clone(),
            },
            Type::Generic(x) => Type::Generic(*x),
            Type::Fun(ts, t) => {
                let ts = map(ts, |t| t.default(int, float));
                let t = t.default(int, float);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = map(ts, |t| t.default(int, float));
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts
                    .iter()
                    .map(|(x, t)| (*x, t.default(int, float)))
                    .collect();
                Type::Record(xts)
            }
            Type::Array(t, i) => {
                let t = Rc::new(t.default(int, float));
                let i = *i;
                Type::Array(t, i)
            }
            Type::Never => Type::Never,
            Type::Hole => Type::Hole,
            Type::Err => Type::Err,
        }
    }

    pub fn expand_assoc(&self, f: &mut impl FnMut(&Bound, &Name, &Vec<Type>) -> Type) -> Type {
        match self {
            Type::Path(_) => unreachable!(),
            Type::Cons(x, ts) => {
                let ts = ts.iter().map(|t| t.expand_assoc(f)).collect::<Vec<_>>();
                Type::Cons(*x, ts)
            }
            Type::Alias(x, ts) => {
                let ts = ts.iter().map(|t| t.expand_assoc(f)).collect::<Vec<_>>();
                Type::Alias(*x, ts)
            }
            Type::Assoc(b, x, ts) => {
                let b = b.map_type(&mut |t| t.expand_assoc(f));
                let ts = map(ts, |t| t.expand_assoc(f));
                f(&b, x, &ts)
            }
            Type::Var(x, k) => Type::Var(*x, *k),
            Type::Generic(x) => Type::Generic(*x),
            Type::Fun(ts, t) => {
                let ts = ts.iter().map(|t| t.expand_assoc(f)).collect();
                let t = t.expand_assoc(f);
                Type::Fun(ts, Rc::new(t))
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| t.expand_assoc(f)).collect();
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, t.expand_assoc(f))).collect();
                Type::Record(xts)
            }
            Type::Array(t, i) => {
                let t = Rc::new(t.expand_assoc(f));
                let i = *i;
                Type::Array(t, i)
            }
            Type::Never => Type::Never,
            Type::Hole => Type::Hole,
            Type::Err => Type::Err,
        }
    }

    pub fn annotate(&self, ctx: &mut Context) -> Type {
        match self {
            Type::Cons(x, ts) => {
                let ts = map(ts, |t| t.annotate(ctx));
                Type::Cons(*x, ts)
            }
            Type::Assoc(b, x1, ts1) => {
                let b = b.map_type(&mut |t| t.annotate(ctx));
                let ts1 = map(ts1, |t| t.annotate(ctx));
                Type::Assoc(b, *x1, ts1)
            }
            Type::Var(x, k) => Type::Var(*x, *k),
            Type::Hole => ctx.new_tyvar(TypeVar::General),
            Type::Err => Type::Err,
            Type::Generic(x) => Type::Generic(*x),
            Type::Fun(ts, t) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let t = Rc::new(t.annotate(ctx));
                Type::Fun(ts, t)
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Type::Tuple(ts)
            }
            Type::Record(xts) => {
                let xts = xts.iter().map(|(x, t)| (*x, t.annotate(ctx))).collect();
                Type::Record(xts)
            }
            Type::Alias(x, ts) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Type::Alias(*x, ts)
            }
            Type::Path(_) => unreachable!(),
            Type::Array(t, n) => {
                let t = t.annotate(ctx);
                let n = *n;
                Type::Array(Rc::new(t), n)
            }
            Type::Never => Type::Never,
        }
    }
}

impl Stmt {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Stmt {
        match self {
            Stmt::Var(s) => Stmt::Var(Rc::new(s.map_type(f))),
            Stmt::Def(s) => Stmt::Def(s.clone()),
            Stmt::Impl(s) => Stmt::Impl(s.clone()),
            Stmt::Expr(s) => Stmt::Expr(Rc::new(s.map_type(f))),
            Stmt::Struct(s) => Stmt::Struct(s.clone()),
            Stmt::Enum(s) => Stmt::Enum(s.clone()),
            Stmt::Type(s) => Stmt::Type(s.clone()),
            Stmt::Trait(s) => Stmt::Trait(s.clone()),
            Stmt::Err(_) => todo!(),
        }
    }
}

impl StmtVar {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtVar {
        let span = self.span;
        let name = self.name;
        let ty = f(&self.ty);
        let expr = self.expr.map_type(f);
        StmtVar::new(span, name, ty, expr)
    }
}

impl StmtDef {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtDef {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let qs = self.where_clause.iter().map(|p| p.map_type(f)).collect();
        let ps = self.params.mapv(&mut *f);
        let t = f(&self.ty);
        let e = self.body.map_type(f);
        StmtDef::new(span, name, generics, ps, t, qs, e)
    }
}

impl StmtDefBody {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtDefBody {
        match self {
            StmtDefBody::UserDefined(e) => StmtDefBody::UserDefined(e.map_type(f)),
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }
}

impl StmtImpl {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtImpl {
        let span = self.span;
        let generics = self.generics.clone();
        let head = self.head.map_type(f);
        let body = self.where_clause.iter().map(|p| p.map_type(f)).collect();
        let defs = self.defs.iter().map(|d| Rc::new(d.map_type(f))).collect();
        let types = self.types.iter().map(|t| Rc::new(t.map_type(f))).collect();
        StmtImpl::new(span, generics, head, body, defs, types)
    }
}

impl StmtType {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtType {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let ty = self.body.map_type(f);
        StmtType::new(span, name, generics, ty)
    }
}

impl StmtTypeBody {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtTypeBody {
        match self {
            StmtTypeBody::UserDefined(t) => StmtTypeBody::UserDefined(f(t)),
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
        }
    }
}

impl StmtStruct {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtStruct {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let fields = self.fields.iter().map(|(x, t)| (*x, f(t))).collect();
        StmtStruct::new(span, name, generics, fields)
    }
}

impl StmtEnum {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtEnum {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let variants = self.variants.iter().map(|(x, t)| (*x, f(t))).collect();
        StmtEnum::new(span, name, generics, variants)
    }
}

impl StmtTrait {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtTrait {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let where_clause = self.where_clause.iter().map(|b| b.map_type(f)).collect();
        let defs = self.defs.iter().map(|d| Rc::new(d.map_type(f))).collect();
        let types = self.types.clone();
        StmtTrait::new(span, name, generics, where_clause, defs, types)
    }
}

impl StmtTraitDef {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> StmtTraitDef {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let params = self.params.iter().map(|(x, t)| (*x, f(t))).collect();
        let ty = f(&self.ty);
        let where_clause = self.where_clause.iter().map(|b| b.map_type(f)).collect();
        StmtTraitDef::new(span, name, generics, params, ty, where_clause)
    }
}

impl Bound {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Bound {
        match self {
            Bound::Path(_, _) => unreachable!(),
            Bound::Trait(s, x, ts, xts) => {
                let ts = map(ts, &mut *f);
                let xts = xts.mapv(&mut |t: &Type| f(t));
                Bound::Trait(*s, *x, ts, xts)
            }
            Bound::Type(s, t) => Bound::Type(*s, Rc::new(f(t.as_ref()))),
            Bound::Err(s) => Bound::Err(*s),
        }
    }
}

impl Expr {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Expr {
        let s = self.span_of();
        let t = f(self.type_of());
        match self {
            Expr::Path(..) => unreachable!(),
            Expr::Int(_, _, v) => Expr::Int(s, t, *v),
            Expr::Float(_, _, v) => Expr::Float(s, t, *v),
            Expr::Bool(_, _, v) => Expr::Bool(s, t, *v),
            Expr::String(_, _, v) => Expr::String(s, t, *v),
            Expr::Var(_, _, x) => Expr::Var(s, t, *x),
            Expr::Def(_, _, x, ts) => {
                let ts = ts.iter().map(f).collect();
                Expr::Def(s, t, *x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = e.map_type(f);
                let es = es.iter().map(|e| e.map_type(f)).collect();
                Expr::Call(s, t, Rc::new(e), es)
            }
            Expr::Block(_, _, b) => {
                let b = b.map_type(f);
                Expr::Block(s, t, b)
            }
            Expr::Query(..) => todo!(),
            Expr::Struct(_, _, x, ts, xts) => {
                let ts = ts.iter().map(&mut *f).collect();
                let xts = xts
                    .iter()
                    .map(&mut |(x, e): &(Name, Expr)| (*x, e.map_type(f)))
                    .collect();
                Expr::Struct(s, t, *x, ts, xts)
            }
            Expr::Enum(_, _, x0, ts, x1, e) => {
                let ts = ts.iter().map(&mut *f).collect();
                let e = e.map_type(f);
                Expr::Enum(s, t, *x0, ts, *x1, Rc::new(e))
            }
            Expr::Field(_, _, e, x) => {
                let e = Rc::new(e.map_type(f));
                Expr::Field(s, t, e, *x)
            }
            Expr::Tuple(_, _, es) => {
                let es = es.iter().map(|e| e.map_type(f)).collect();
                Expr::Tuple(s, t, es)
            }
            Expr::Assoc(_, _, b, x1, ts1) => {
                let b = b.map_type(f);
                let ts1 = ts1.iter().map(f).collect();
                Expr::Assoc(s, t, b, *x1, ts1)
            }
            Expr::Index(_, _, e, i) => {
                let e = Rc::new(e.map_type(f));
                Expr::Index(s, t, e, *i)
            }
            Expr::Array(_, _, es) => {
                let es = es.iter().map(|e| e.map_type(f)).collect();
                Expr::Array(s, t, es)
            }
            Expr::Err(_, _) => Expr::Err(s, t),
            Expr::Assign(_, _, e0, e1) => {
                let e0 = Rc::new(e0.map_type(f));
                let e1 = Rc::new(e1.map_type(f));
                Expr::Assign(s, t, e0, e1)
            }
            Expr::Return(_, _, e) => {
                let e = Rc::new(e.map_type(f));
                Expr::Return(s, t, e)
            }
            Expr::Continue(_, _) => Expr::Continue(s, t),
            Expr::Break(_, _) => Expr::Break(s, t),
            Expr::Fun(_, _, ps, t1, e) => {
                let ps = ps.mapv(&mut *f);
                let t1 = f(t1);
                let e = Rc::new(e.map_type(f));
                Expr::Fun(s, t, ps, t1, e)
            }
            Expr::Match(_, _, e, pes) => {
                let e = Rc::new(e.map_type(f));
                let pes = pes.iter().map(|arm| arm.map_type(f)).collect();
                Expr::Match(s, t, e, pes)
            }
            Expr::While(_, _, e, b) => {
                let e = Rc::new(e.map_type(f));
                let b = b.map_type(f);
                Expr::While(s, t, e, b)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes.iter().map(|(x, e)| (*x, e.map_type(f))).collect();
                Expr::Record(s, t, xes)
            }
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, v) => Expr::Char(s, t, *v),
            Expr::Unresolved(_, _, x, ts) => {
                let ts = ts.iter().map(f).collect();
                Expr::Unresolved(s, t, *x, ts)
            }
        }
    }
}

impl Arm {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Arm {
        let span = self.span;
        let p = self.p.map_type(f);
        let e = self.e.map_type(f);
        Arm::new(span, p, e)
    }
}

impl Block {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Block {
        let span = self.span;
        let stmts = self.stmts.iter().map(|s| s.map_type(f)).collect();
        let expr = self.expr.map_type(f);
        Block::new(span, stmts, expr)
    }
}

impl Pat {
    pub fn map_type(&self, f: &mut impl FnMut(&Type) -> Type) -> Pat {
        let t = f(self.type_of());
        let span = self.span_of();
        match self {
            Pat::Path(..) => unreachable!(),
            Pat::Var(_, _, x) => Pat::Var(span, t, *x),
            Pat::Tuple(_, _, es) => {
                let es = es.iter().map(|e| e.map_type(f)).collect();
                Pat::Tuple(span, t, es)
            }
            Pat::Struct(_, _, x, ts, xps) => {
                let ts = ts.iter().map(&mut *f).collect();
                let xps = xps
                    .iter()
                    .map(&mut |(x, p): &(Name, Pat)| (*x, p.map_type(f)))
                    .collect();
                Pat::Struct(span, t, *x, ts, xps)
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                let ts = ts.iter().map(&mut *f).collect();
                let p = Rc::new(p.map_type(f));
                Pat::Enum(span, t, *x0, ts, *x1, p)
            }
            Pat::Int(_, _, v) => Pat::Int(span, t, *v),
            Pat::String(_, _, v) => Pat::String(span, t, *v),
            Pat::Wildcard(_, _) => Pat::Wildcard(span, t),
            Pat::Bool(_, _, v) => Pat::Bool(span, t, *v),
            Pat::Err(_, _) => Pat::Err(span, t),
            Pat::Record(_, _, xps) => {
                let xps = xps.iter().map(|(x, p)| (*x, p.map_type(f))).collect();
                Pat::Record(span, t, xps)
            }
            Pat::Or(_, _, p0, p1) => {
                let p0 = Rc::new(p0.map_type(f));
                let p1 = Rc::new(p1.map_type(f));
                Pat::Or(span, t, p0, p1)
            }
            Pat::Char(_, _, c) => Pat::Char(span, t, *c),
        }
    }
}
