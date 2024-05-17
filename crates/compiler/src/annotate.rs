use std::rc::Rc;

use crate::ast::Arm;
use crate::ast::Block;
use crate::ast::Bound;
use crate::ast::Expr;
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
use crate::ast::StmtType;
use crate::ast::StmtTypeBody;
use crate::ast::StmtVar;
use crate::ast::TraitBound;
use crate::ast::StmtTraitDef;
use crate::ast::Type;
use crate::ast::UnresolvedPatField;
use crate::infer::Context;

// Replace all holes with fresh type variables.
fn map<T>(ts: &[T], f: impl FnMut(&T) -> T) -> Vec<T> {
    ts.iter().map(f).collect()
}

impl Type {
    pub fn annotate(&self, ctx: &mut Context) -> Type {
        match self {
            Type::Cons(x, ts) => {
                let ts = map(ts, |t| t.annotate(ctx));
                Type::Cons(*x, ts)
            }
            Type::Assoc(b, x1, ts1) => {
                let b = b.annotate(ctx);
                let ts1 = map(ts1, |t| t.annotate(ctx));
                Type::Assoc(b, *x1, ts1)
            }
            Type::Var(x) => Type::Var(*x),
            Type::Hole => ctx.new_tyvar(),
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
            Type::Unresolved(p) => {
                let p = p.annotate(ctx);
                Type::Unresolved(p)
            }
            Type::Array(t, n) => {
                let t = t.annotate(ctx);
                let n = *n;
                Type::Array(Rc::new(t), n)
            }
            Type::Never => Type::Never,
        }
    }
}

impl Path {
    pub fn annotate(&self, ctx: &mut Context) -> Path {
        let segments = self.segments.iter().map(|seg| seg.annotate(ctx)).collect();
        Path::new(segments)
    }
}

impl Segment {
    pub fn annotate(&self, ctx: &mut Context) -> Segment {
        let x = self.name;
        let ts = self.ts.iter().map(|t| t.annotate(ctx)).collect();
        let xts = self
            .xts
            .iter()
            .map(|(x, t)| (*x, t.annotate(ctx)))
            .collect();
        Segment::new(self.span, x, ts, xts)
    }
}

impl Program {
    pub fn annotate(&self, ctx: &mut Context) -> Program {
        let stmts = self.stmts.iter().map(|s| s.annotate(ctx)).collect();
        Program::new(stmts)
    }
}

impl Stmt {
    pub fn annotate(&self, ctx: &mut Context) -> Stmt {
        match self {
            Stmt::Var(v) => {
                let v = v.annotate(ctx);
                Stmt::Var(Rc::new(v))
            }
            Stmt::Def(d) => {
                let d = d.annotate(ctx);
                Stmt::Def(Rc::new(d))
            }
            Stmt::Impl(i) => {
                let i = i.annotate(ctx);
                Stmt::Impl(Rc::new(i))
            }
            Stmt::Expr(e) => {
                let e = e.annotate(ctx);
                Stmt::Expr(Rc::new(e))
            }
            Stmt::Struct(s) => {
                let s = s.annotate(ctx);
                Stmt::Struct(Rc::new(s))
            }
            Stmt::Enum(e) => {
                let e = e.annotate(ctx);
                Stmt::Enum(Rc::new(e))
            }
            Stmt::Type(s) => {
                let s = s.annotate(ctx);
                Stmt::Type(Rc::new(s))
            }
            Stmt::Trait(s) => {
                let s = s.annotate(ctx);
                Stmt::Trait(Rc::new(s))
            }
            Stmt::Err(_) => todo!(),
        }
    }
}

impl StmtStruct {
    pub fn annotate(&self, ctx: &mut Context) -> StmtStruct {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let fields = self
            .fields
            .iter()
            .map(|(x, t)| (*x, t.annotate(ctx)))
            .collect();
        StmtStruct::new(span, name, generics, fields)
    }
}

impl StmtType {
    pub fn annotate(&self, ctx: &mut Context) -> StmtType {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let ty = self.body.annotate(ctx);
        StmtType::new(span, name, generics, ty)
    }
}

impl StmtTypeBody {
    pub fn annotate(&self, ctx: &mut Context) -> StmtTypeBody {
        match self {
            StmtTypeBody::Builtin(b) => StmtTypeBody::Builtin(b.clone()),
            StmtTypeBody::UserDefined(t) => {
                let t = t.annotate(ctx);
                StmtTypeBody::UserDefined(t)
            }
        }
    }
}

impl StmtEnum {
    pub fn annotate(&self, ctx: &mut Context) -> StmtEnum {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let variants = self
            .variants
            .iter()
            .map(|(x, t)| (*x, t.annotate(ctx)))
            .collect();
        StmtEnum::new(span, name, generics, variants)
    }
}

impl StmtVar {
    pub fn annotate(&self, ctx: &mut Context) -> StmtVar {
        let span = self.span;
        let x = self.name;
        let t = self.ty.annotate(ctx);
        let e = self.expr.annotate(ctx);
        StmtVar::new(span, x, t, e)
    }
}

impl StmtDef {
    pub fn annotate(&self, ctx: &mut Context) -> StmtDef {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let preds = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let params = self.params.map_values(|t| t.annotate(ctx));
        let ty = self.ty.annotate(ctx);
        let body = self.body.annotate(ctx);
        StmtDef::new(span, name, generics, params, ty, preds, body)
    }
}

impl StmtDefBody {
    pub fn annotate(&self, ctx: &mut Context) -> StmtDefBody {
        match self {
            StmtDefBody::UserDefined(e) => {
                let e = e.annotate(ctx);
                StmtDefBody::UserDefined(e)
            }
            StmtDefBody::Builtin(b) => StmtDefBody::Builtin(b.clone()),
        }
    }
}

impl StmtImpl {
    pub fn annotate(&self, ctx: &mut Context) -> StmtImpl {
        let span = self.span;
        let generics = self.generics.clone();
        let head = self.head.annotate(ctx);
        let body = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let defs = self.defs.iter().map(|d| Rc::new(d.as_ref().annotate(ctx))).collect();
        let tys = self.types.iter().map(|t| Rc::new(t.as_ref().annotate(ctx))).collect();
        StmtImpl::new(span, generics, head, body, defs, tys)
    }
}

impl StmtTrait {
    fn annotate(&self, ctx: &mut Context) -> StmtTrait {
        let span = self.span;
        let generics = self.generics.clone();
        let name = self.name;
        let body = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let defs = self.defs.iter().map(|d| Rc::new(d.as_ref().annotate(ctx))).collect();
        let assocs = self.types.clone();
        StmtTrait::new(span, name, generics, body, defs, assocs)
    }
}

impl StmtTraitDef {
    pub fn annotate(&self, ctx: &mut Context) -> StmtTraitDef {
        let span = self.span;
        let name = self.name;
        let generics = self.generics.clone();
        let where_clause = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let params = self.params.map_values(|t| t.annotate(ctx));
        let ty = self.ty.annotate(ctx);
        StmtTraitDef::new(span, name, generics, params, ty, where_clause)
    }
}

impl Expr {
    pub fn annotate(&self, ctx: &mut Context) -> Expr {
        let s = self.span();
        let t = self.ty().annotate(ctx);
        match self {
            Expr::Unresolved(..) => unreachable!(),
            Expr::Int(_, _, v) => {
                let v = v.clone();
                Expr::Int(s, t, v)
            }
            Expr::Float(_, _, v) => {
                let v = v.clone();
                Expr::Float(s, t, v)
            }
            Expr::Bool(_, _, v) => Expr::Bool(s, t, *v),
            Expr::String(_, _, v) => {
                let v = v.clone();
                Expr::String(s, t, v)
            }
            Expr::Var(_, _, x) => Expr::Var(s, t, *x),
            Expr::Def(_, _, x, ts) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Expr::Def(s, t, *x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = Rc::new(e.annotate(ctx));
                let es = es.iter().map(|e| e.annotate(ctx)).collect();
                Expr::Call(s, t, e, es)
            }
            Expr::Block(_, _, b) => {
                let ss = b.stmts.iter().map(|s| s.annotate(ctx)).collect();
                let e = b.expr.annotate(ctx);
                Expr::Block(s, t, Block::new(b.span, ss, e))
            }
            Expr::Query(..) => todo!(),
            Expr::Struct(_, _, x, ts, xes) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let xes = xes.iter().map(|(x, e)| (*x, e.annotate(ctx))).collect();
                Expr::Struct(s, ctx.new_tyvar(), *x, ts, xes)
            }
            Expr::Enum(_, _, x0, ts0, x1, e) => {
                let ts0 = ts0.iter().map(|t| t.annotate(ctx)).collect();
                let e = Rc::new(e.annotate(ctx));
                Expr::Enum(s, t, *x0, ts0, *x1, e)
            }
            Expr::Field(_, _, e, x) => {
                let e = Rc::new(e.annotate(ctx));
                Expr::Field(s, t, e, *x)
            }
            Expr::Tuple(_, _, es) => {
                let es = es.iter().map(|e| e.annotate(ctx)).collect();
                Expr::Tuple(s, t, es)
            }
            Expr::Assoc(_, _, b, x1, ts1) => {
                let b = b.annotate(ctx);
                let ts1 = ts1.iter().map(|t| t.annotate(ctx)).collect();
                Expr::Assoc(s, t, b, *x1, ts1)
            }
            Expr::Index(_, _, e, i) => {
                let e = Rc::new(e.annotate(ctx));
                Expr::Index(s, t, e, *i)
            }
            Expr::Array(_, _, es) => {
                let es = es.iter().map(|e| e.annotate(ctx)).collect();
                Expr::Array(s, t, es)
            }
            Expr::Err(s, t) => {
                let t = t.annotate(ctx);
                Expr::Err(*s, t)
            }
            Expr::Assign(_, _, e0, e1) => {
                let e0 = Rc::new(e0.annotate(ctx));
                let e1 = Rc::new(e1.annotate(ctx));
                Expr::Assign(s, t, e0, e1)
            }
            Expr::Return(_, _, e) => {
                let e = Rc::new(e.annotate(ctx));
                Expr::Return(s, t, e)
            }
            Expr::Continue(_, _) => Expr::Continue(s, t),
            Expr::Break(_, _) => Expr::Break(s, t),
            Expr::Fun(_, _, ps, t1, e) => {
                let ps = ps.map_values(|t| t.annotate(ctx));
                let t1 = t1.annotate(ctx);
                let e = Rc::new(e.annotate(ctx));
                Expr::Fun(s, t, ps, t1, e)
            }
            Expr::Match(_, _, e, pes) => {
                let e = Rc::new(e.annotate(ctx));
                let pes = pes.iter().map(|arm| arm.annotate(ctx)).collect();
                Expr::Match(s, t, e, pes)
            }
            Expr::While(_, _, e0, b) => {
                let e = Rc::new(e0.annotate(ctx));
                let b = b.annotate(ctx);
                Expr::While(s, t, e, b)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes.iter().map(|(x, e)| (*x, e.annotate(ctx))).collect();
                Expr::Record(s, t, xes)
            }
            Expr::Value(_, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Char(_, _, _) => todo!(),
        }
    }
}

impl Block {
    pub fn annotate(&self, ctx: &mut Context) -> Block {
        let ss = self.stmts.iter().map(|s| s.annotate(ctx)).collect();
        let expr = self.expr.annotate(ctx);
        Block::new(self.span, ss, expr)
    }
}

impl Arm {
    pub fn annotate(&self, ctx: &mut Context) -> Arm {
        let p = self.p.annotate(ctx);
        let e = self.e.annotate(ctx);
        Arm::new(self.span, p, e)
    }
}

impl Pat {
    pub fn annotate(&self, ctx: &mut Context) -> Pat {
        let s = self.span();
        let t = self.ty().annotate(ctx);
        match self {
            Pat::Unresolved(_, _, path, args) => {
                let path = path.annotate(ctx);
                let args = args
                    .as_ref()
                    .map(|args| args.iter().map(|arg| arg.annotate(ctx)).collect());
                Pat::Unresolved(s, t, path, args)
            }
            Pat::Var(_, _, x) => Pat::Var(s, t, *x),
            Pat::Tuple(_, _, ps) => {
                let ps = ps.iter().map(|p| p.annotate(ctx)).collect();
                Pat::Tuple(s, t, ps)
            }
            Pat::Struct(_, _, x, ts, xps) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let xps = xps.iter().map(|(x, p)| (*x, p.annotate(ctx))).collect();
                Pat::Struct(s, t, *x, ts, xps)
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let p = p.annotate(ctx);
                Pat::Enum(s, t, *x0, ts, *x1, Rc::new(p))
            }
            Pat::Int(_, _, v) => {
                let v = v.clone();
                Pat::Int(s, t, v)
            }
            Pat::String(_, _, v) => {
                let v = v.clone();
                Pat::String(s, t, v)
            }
            Pat::Wildcard(_, _) => Pat::Wildcard(s, t),
            Pat::Bool(_, _, v) => Pat::Bool(s, t, *v),
            Pat::Err(_, _) => Pat::Err(s, t),
            Pat::Record(_, _, xps) => {
                let xps = xps.iter().map(|(x, p)| (*x, p.annotate(ctx))).collect();
                Pat::Record(s, t, xps)
            }
            Pat::Or(_, _, p0, p1) => {
                let p0 = p0.annotate(ctx);
                let p1 = p1.annotate(ctx);
                Pat::Or(s, t, Rc::new(p0), Rc::new(p1))
            }
            Pat::Char(_, _, c) => Pat::Char(s, t, *c),
        }
    }
}

impl UnresolvedPatField {
    pub fn annotate(&self, ctx: &mut Context) -> UnresolvedPatField {
        match self {
            UnresolvedPatField::Named(x, p) => {
                let p = p.annotate(ctx);
                UnresolvedPatField::Named(*x, p)
            }
            UnresolvedPatField::Unnamed(p) => {
                let p = p.annotate(ctx);
                UnresolvedPatField::Unnamed(p)
            }
        }
    }
}

impl Bound {
    pub fn annotate(&self, ctx: &mut Context) -> Bound {
        let span = self.span();
        match self {
            Bound::Unresolved(_, path) => {
                let path = path.annotate(ctx);
                Bound::Unresolved(span, path)
            }
            Bound::Trait(_, b) => {
                let b = b.annotate(ctx);
                Bound::Trait(span, b)
            }
            Bound::Err(_) => Bound::Err(span),
        }
    }
}

impl TraitBound {
    pub fn annotate(&self, ctx: &mut Context) -> TraitBound {
        let x = self.name;
        let ts = self.ts.iter().map(|t| t.annotate(ctx)).collect();
        let xts = self
            .xts
            .iter()
            .map(|(x, t)| (*x, t.annotate(ctx)))
            .collect();
        TraitBound::new(x, ts, xts)
    }
}
