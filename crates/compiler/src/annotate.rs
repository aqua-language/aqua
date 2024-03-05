use std::rc::Rc;

use crate::ast::Body;
use crate::ast::Bound;
use crate::ast::Expr;
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
use crate::infer::Context;

// Replace all holes with fresh type variables.
fn map<T>(ts: &[T], f: impl FnMut(&T) -> T) -> Vec<T> {
    ts.iter().map(f).collect()
}

impl Type {
    pub fn annotate(&self, ctx: &mut Context) -> Type {
        match self {
            Type::Cons(x, ts) => {
                let x = x.clone();
                let ts = map(ts, |t| t.annotate(ctx));
                Type::Cons(x, ts)
            }
            Type::Assoc(x0, ts0, x1, ts1) => {
                let x0 = x0.clone();
                let ts0 = map(ts0, |t| t.annotate(ctx));
                let x1 = x1.clone();
                let ts1 = map(ts1, |t| t.annotate(ctx));
                Type::Assoc(x0, ts0, x1, ts1)
            }
            Type::Var(x) => {
                let x = x.clone();
                Type::Var(x)
            }
            Type::Hole => ctx.new_tyvar(),
            Type::Err => Type::Err,
            Type::Generic(x) => {
                let x = x.clone();
                Type::Generic(x)
            }
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
                let xts = xts
                    .iter()
                    .map(|(x, t)| (x.clone(), t.annotate(ctx)))
                    .collect();
                Type::Record(xts)
            }
            Type::Alias(x, ts) => {
                let x = x.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Type::Alias(x, ts)
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

impl UnresolvedPath {
    pub fn annotate(&self, ctx: &mut Context) -> UnresolvedPath {
        let segments = self
            .segments
            .iter()
            .map(|(x, ts)| (x.clone(), ts.iter().map(|t| t.annotate(ctx)).collect()))
            .collect();
        UnresolvedPath::new(segments)
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
                Stmt::Var(v)
            }
            Stmt::Def(d) => {
                let d = d.annotate(ctx);
                Stmt::Def(d)
            }
            Stmt::Impl(i) => {
                let i = i.annotate(ctx);
                Stmt::Impl(i)
            }
            Stmt::Expr(e) => {
                let e = e.annotate(ctx);
                Stmt::Expr(e)
            }
            Stmt::Struct(s) => {
                let s = s.annotate(ctx);
                Stmt::Struct(s)
            }
            Stmt::Enum(e) => {
                let e = e.annotate(ctx);
                Stmt::Enum(e)
            }
            Stmt::Type(s) => {
                let s = s.annotate(ctx);
                Stmt::Type(s)
            }
            Stmt::Trait(s) => {
                let s = s.annotate(ctx);
                Stmt::Trait(s)
            }
            Stmt::Err(_) => todo!(),
        }
    }
}

impl StmtStruct {
    pub fn annotate(&self, ctx: &mut Context) -> StmtStruct {
        let span = self.span;
        let name = self.name.clone();
        let generics = self.generics.clone();
        let fields = self
            .fields
            .iter()
            .map(|(x, t)| (x.clone(), t.annotate(ctx)))
            .collect();
        StmtStruct::new(span, name, generics, fields)
    }
}

impl StmtType {
    pub fn annotate(&self, ctx: &mut Context) -> StmtType {
        let span = self.span;
        let name = self.name.clone();
        let generics = self.generics.clone();
        let ty = self.ty.annotate(ctx);
        StmtType::new(span, name, generics, ty)
    }
}

impl StmtEnum {
    pub fn annotate(&self, ctx: &mut Context) -> StmtEnum {
        let span = self.span;
        let name = self.name.clone();
        let generics = self.generics.clone();
        let variants = self
            .variants
            .iter()
            .map(|(x, t)| (x.clone(), t.annotate(ctx)))
            .collect();
        StmtEnum::new(span, name, generics, variants)
    }
}

impl StmtVar {
    pub fn annotate(&self, ctx: &mut Context) -> StmtVar {
        let span = self.span;
        let x = self.name.clone();
        let t = self.ty.annotate(ctx);
        let e = self.expr.annotate(ctx);
        StmtVar::new(span, x, t, e)
    }
}

impl StmtDef {
    pub fn annotate(&self, ctx: &mut Context) -> StmtDef {
        let span = self.span;
        let name = self.name.clone();
        let generics = self.generics.clone();
        let preds = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let params = self.params.iter().map(|p| p.annotate(ctx)).collect();
        let ty = self.ty.annotate(ctx);
        let body = self.body.annotate(ctx);
        StmtDef::new(span, name, generics, preds, params, ty, body)
    }
}

impl Body {
    pub fn annotate(&self, ctx: &mut Context) -> Body {
        match self {
            Body::Expr(e) => {
                let e = e.annotate(ctx);
                Body::Expr(e)
            }
            Body::Builtin => todo!(),
        }
    }
}

impl StmtImpl {
    pub fn annotate(&self, ctx: &mut Context) -> StmtImpl {
        let span = self.span;
        let generics = self.generics.clone();
        let head = self.head.annotate(ctx);
        let body = self.where_clause.iter().map(|p| p.annotate(ctx)).collect();
        let defs = self.defs.iter().map(|d| d.annotate(ctx)).collect();
        let tys = self.types.iter().map(|t| t.annotate(ctx)).collect();
        StmtImpl::new(span, generics, head, body, defs, tys)
    }
}

impl StmtTrait {
    fn annotate(&self, ctx: &mut Context) -> StmtTrait {
        let span = self.span;
        let generics = self.generics.clone();
        let name = self.name.clone();
        let body = self.bounds.iter().map(|p| p.annotate(ctx)).collect();
        let defs = self.defs.iter().map(|d| d.annotate(ctx)).collect();
        let assocs = self.types.clone();
        StmtTrait::new(span, name, generics, body, defs, assocs)
    }
}

impl TraitDef {
    pub fn annotate(&self, ctx: &mut Context) -> TraitDef {
        let span = self.span;
        let name = self.name.clone();
        let generics = self.generics.clone();
        let preds = self.bounds.iter().map(|p| p.annotate(ctx)).collect();
        let params = self.params.iter().map(|p| p.annotate(ctx)).collect();
        let ty = self.ty.annotate(ctx);
        TraitDef::new(span, name, generics, preds, params, ty)
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
            Expr::Var(_, _, x) => {
                let x = x.clone();
                Expr::Var(s, t, x)
            }
            Expr::Def(_, _, x, ts) => {
                let x = x.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Expr::Def(s, t, x, ts)
            }
            Expr::Call(_, _, e, es) => {
                let e = Rc::new(e.annotate(ctx));
                let es = es.iter().map(|e| e.annotate(ctx)).collect();
                Expr::Call(s, t, e, es)
            }
            Expr::Block(_, _, ss, e) => {
                let ss = ss.iter().map(|s| s.annotate(ctx)).collect();
                let e = Rc::new(e.annotate(ctx));
                Expr::Block(s, t, ss, e)
            }
            Expr::Query(..) => todo!(),
            Expr::Struct(_, _, x, ts, xes) => {
                let x = x.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let xes = xes
                    .iter()
                    .map(|(x, e)| (x.clone(), e.annotate(ctx)))
                    .collect();
                Expr::Struct(s, ctx.new_tyvar(), x, ts, xes)
            }
            Expr::Enum(_, _, x0, ts0, x1, e) => {
                let x0 = x0.clone();
                let ts0 = ts0.iter().map(|t| t.annotate(ctx)).collect();
                let x1 = x1.clone();
                let e = Rc::new(e.annotate(ctx));
                Expr::Enum(s, t, x0, ts0, x1, e)
            }
            Expr::Field(_, _, e, x) => {
                let e = Rc::new(e.annotate(ctx));
                let x = x.clone();
                Expr::Field(s, t, e, x)
            }
            Expr::Tuple(_, _, es) => {
                let es = es.iter().map(|e| e.annotate(ctx)).collect();
                Expr::Tuple(s, t, es)
            }
            Expr::Assoc(_, _, x0, ts0, x1, ts1) => {
                let x0 = x0.clone();
                let ts0 = ts0.iter().map(|t| t.annotate(ctx)).collect();
                let x1 = x1.clone();
                let ts1 = ts1.iter().map(|t| t.annotate(ctx)).collect();
                Expr::Assoc(s, t, x0, ts0, x1, ts1)
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
                let ps = ps.iter().map(|p| p.annotate(ctx)).collect();
                let t1 = t1.annotate(ctx);
                let e = Rc::new(e.annotate(ctx));
                Expr::Fun(s, t, ps, t1, e)
            }
            Expr::Match(_, _, e, pes) => {
                let e = Rc::new(e.annotate(ctx));
                let pes = pes
                    .iter()
                    .map(|(p, e)| (p.annotate(ctx), e.annotate(ctx)))
                    .collect();
                Expr::Match(s, t, e, pes)
            }
            Expr::While(_, _, e0, e1) => {
                let e0 = Rc::new(e0.annotate(ctx));
                let e1 = Rc::new(e1.annotate(ctx));
                Expr::While(s, t, e0, e1)
            }
            Expr::Record(_, _, xes) => {
                let xes = xes
                    .iter()
                    .map(|(x, e)| (x.clone(), e.annotate(ctx)))
                    .collect();
                Expr::Record(s, t, xes)
            }
            Expr::Value(_, _) => todo!(),
            Expr::Infix(_, _, _, _, _) => unreachable!(),
            Expr::Postfix(_, _, _, _) => unreachable!(),
            Expr::Prefix(_, _, _, _) => unreachable!(),
        }
    }
}

impl Pat {
    pub fn annotate(&self, ctx: &mut Context) -> Pat {
        let s = self.span();
        let t = self.ty().annotate(ctx);
        match self {
            // Pat::Unresolved(_, _, _, _) => todo!(),
            // Pat::Var(_, _, _) => todo!(),
            // Pat::Tuple(_, _, _) => todo!(),
            // Pat::Struct(_, _, _, _, _) => todo!(),
            // Pat::Enum(_, _, _, _, _, _) => todo!(),
            // Pat::Int(_, _, _) => todo!(),
            // Pat::String(_, _, _) => todo!(),
            // Pat::Wildcard(_, _) => todo!(),
            // Pat::Bool(_, _, _) => todo!(),
            // Pat::Err(_, _) => todo!(),
            Pat::Unresolved(_, _, path, args) => {
                let path = path.annotate(ctx);
                let args = args
                    .as_ref()
                    .map(|args| args.iter().map(|arg| arg.annotate(ctx)).collect());
                Pat::Unresolved(s, t, path, args)
            }
            Pat::Var(_, _, x) => {
                let x = x.clone();
                Pat::Var(s, t, x)
            }
            Pat::Tuple(_, _, ps) => {
                let ps = ps.iter().map(|p| p.annotate(ctx)).collect();
                Pat::Tuple(s, t, ps)
            }
            Pat::Struct(_, _, x, ts, xps) => {
                let x = x.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let xps = xps
                    .iter()
                    .map(|(x, p)| (x.clone(), p.annotate(ctx)))
                    .collect();
                Pat::Struct(s, t, x, ts, xps)
            }
            Pat::Enum(_, _, x0, ts, x1, p) => {
                let x0 = x0.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                let p = p.annotate(ctx);
                Pat::Enum(s, t, x0, ts, x1.clone(), Rc::new(p))
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
        }
    }
}

impl PatArg {
    pub fn annotate(&self, ctx: &mut Context) -> PatArg {
        match self {
            PatArg::Named(x, p) => {
                let p = p.annotate(ctx);
                PatArg::Named(x.clone(), p)
            }
            PatArg::Unnamed(p) => {
                let p = p.annotate(ctx);
                PatArg::Unnamed(p)
            }
        }
    }
}

impl Param {
    pub fn annotate(&self, ctx: &mut Context) -> Param {
        let span = self.span;
        let name = self.name.clone();
        let ty = self.ty.annotate(ctx);
        Param::new(span, name, ty)
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
            Bound::Trait(_, x, ts) => {
                let x = x.clone();
                let ts = ts.iter().map(|t| t.annotate(ctx)).collect();
                Bound::Trait(span, x, ts)
            }
            Bound::Err(_) => Bound::Err(span),
        }
    }
}