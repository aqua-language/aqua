use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write;
use std::rc::Rc;

use crate::apply::Instantiate;
use crate::ast::Bound;
use crate::ast::Expr;
use crate::ast::Map;
use crate::ast::Name;
use crate::ast::Program;
use crate::ast::Stmt;
use crate::ast::StmtDef;
use crate::ast::StmtDefBody;
use crate::ast::StmtEnum;
use crate::ast::StmtImpl;
use crate::ast::StmtStruct;
use crate::ast::Type;
use crate::traversal::mapper::Mapper;
use crate::traversal::visitor::Visitor;

#[derive(Debug, Default)]
pub struct Context {
    unique: HashSet<Name>,
    defs: HashMap<Name, Rc<StmtDef>>,
    structs: HashMap<Name, Rc<StmtStruct>>,
    enums: HashMap<Name, Rc<StmtEnum>>,
    impls: Vec<Rc<StmtImpl>>,
    stmts: Vec<Stmt>,
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn monomorphise(&mut self, p: &Program) -> Program {
        self.visit_program(p);
        let stmts = p
            .stmts
            .iter()
            .filter_map(|stmt| self.top_stmt(stmt))
            .collect();
        Program::new(stmts)
    }

    fn top_stmt(&mut self, s: &Stmt) -> Option<Stmt> {
        match s {
            Stmt::Var(s) => Some(Stmt::Var(Rc::new(self.map_stmt_var(s)))),
            Stmt::Def(_) => None,
            Stmt::Trait(_) => None,
            Stmt::Impl(_) => None,
            Stmt::Struct(_) => None,
            Stmt::Enum(_) => None,
            Stmt::Type(_) => None,
            Stmt::Expr(e) => Some(Stmt::Expr(Rc::new(self.map_expr(e)))),
            Stmt::Err(_) => unreachable!(),
        }
    }

    fn monomorphise_stmt_def(&mut self, stmt: &StmtDef, ts: &[Type]) -> Name {
        let x = Mangler::mangle_fun(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        }
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_def(stmt);
        let ps = stmt.params.mapv(|t| self.map_type(t));
        let t = self.map_type(&stmt.ty);
        let b = StmtDefBody::UserDefined(self.map_expr(stmt.body.as_expr()));
        let stmt = StmtDef::new(stmt.span, x, vec![], ps, t, vec![], b);
        self.stmts.push(Stmt::Def(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_struct(&mut self, stmt: &StmtStruct, ts: &[Type]) -> Name {
        let x = Mangler::mangle_struct(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        }
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_struct(stmt);
        let span = stmt.span;
        let fields = stmt
            .fields
            .iter()
            .map(|(x, t)| (*x, self.map_type(t)))
            .collect();
        let stmt = StmtStruct::new(span, x, vec![], fields);
        self.stmts.push(Stmt::Struct(Rc::new(stmt)));
        x
    }

    fn monomorphise_stmt_enum(&mut self, stmt: &StmtEnum, ts: &[Type]) -> Name {
        let x = Mangler::mangle_enum(stmt.name, ts);
        if self.unique.contains(&x) {
            return x;
        }
        let gsub = stmt
            .generics
            .clone()
            .into_iter()
            .zip(ts.to_vec())
            .collect::<Map<Name, Type>>();
        let stmt = Instantiate::new(&gsub).map_stmt_enum(stmt);
        let variants = stmt.variants.mapv(|t| self.map_type(t));
        let stmt = StmtEnum::new(stmt.span, x, vec![], variants);
        self.stmts.push(Stmt::Enum(Rc::new(stmt)));
        x
    }

    fn _monomorphise_stmt_impl_def(
        &mut self,
        _stmt: &StmtImpl,
        _ts0: &[Type],
        _x: &Name,
        _ts1: &[Type],
    ) -> Name {
        todo!()
        // let x = Mangler::mangle_impl_def(stmt.name, ts0, *x, ts1);
        // if self.unique.contains(&x) {
        //     return x;
        // }
        // let gsub = stmt
        //     .generics
        //     .clone()
        //     .into_iter()
        //     .zip(ts0.to_vec())
        //     .collect::<Map<Name, Type>>();
        // let stmt = Instantiate::new(&gsub).map_stmt_impl(stmt);
        // let ps = stmt.params.mapv(|t| self.map_type(t));
        // let t = self.map_type(&stmt.ty);
        // let b = StmtDefBody::UserDefined(self.map_expr(stmt.body.as_expr()));
        // let stmt = StmtDef::new(stmt.span, x, vec![], ps, t, vec![], b);
        // self.stmts.push(Stmt::Def(Rc::new(stmt)));
        // x
    }

    fn _resolve(&mut self, _b: &Bound) -> Option<Rc<StmtImpl>> {
        todo!()
        // let snap = self.table.snapshot();
        // let x = self.resolve(b)?;
        // let stmt = self.impls.get(x)?;
        // let gsub = stmt
        //     .generics
        //     .clone()
        //     .into_iter()
        //     .zip(ts.to_vec())
        //     .collect::<Map<Name, Type>>();
        // let stmt = Instantiate::new(&gsub).map_stmt_impl(stmt);
        // Some(Rc::new(stmt))
    }

    fn _matches(&mut self, b0: &Bound, b1: &Bound) -> bool {
        match (b0, b1) {
            (Bound::Path(..), Bound::Path(..)) => unreachable!(),
            _ => todo!(),
        }
    }
}

impl Visitor for Context {
    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Var(_) => {}
            Stmt::Def(s) => {
                self.defs.insert(s.name, s.clone());
            }
            Stmt::Trait(_) => {}
            Stmt::Impl(s) => {
                self.impls.push(s.clone());
            }
            Stmt::Struct(s) => {
                self.structs.insert(s.name, s.clone());
            }
            Stmt::Enum(s) => {
                self.enums.insert(s.name, s.clone());
            }
            Stmt::Type(_) => {}
            Stmt::Expr(_) => {}
            Stmt::Err(_) => {}
        }
    }
}

impl Mapper for Context {
    fn map_expr(&mut self, e: &Expr) -> Expr {
        let t = self.map_type(e.type_of());
        let s = e.span_of();
        match e {
            Expr::Struct(_, _, x, ts, xes) => {
                let stmt = self.structs.get(x).unwrap().clone();
                let x = self.monomorphise_stmt_struct(&stmt, ts);
                let xes = self.map_expr_fields(xes).into();
                Expr::Struct(s, t, x, vec![], xes)
            }
            Expr::Enum(_, _, x, ts, x1, e) => {
                let stmt = self.enums.get(x).unwrap().clone();
                let x = self.monomorphise_stmt_enum(&stmt, ts);
                let e = self.map_expr(e);
                Expr::Enum(s, t, x, vec![], *x1, Rc::new(e))
            }
            Expr::Assoc(_, _, _b, _x, _ts) => {
                todo!()
                // let Bound::Trait(s, x0, ts0, xts) = b else {
                //     unreachable!()
                // };
                // let stmt = self.resolve(x0).unwrap().clone();
                // let stmt = self.impls.get(x0).unwrap().clone();
                // let x = self.monomorphise_stmt_impl_def(stmt, ts0, x, ts1);
                // Expr::Assoc(s, t, b.clone(), x, vec![])
            }
            Expr::Def(_, _, x, ts) => {
                let ts = self.map_types(ts);
                let stmt = self.defs.get(x).unwrap();
                match &stmt.body {
                    StmtDefBody::UserDefined(_) => {
                        let x = self.monomorphise_stmt_def(&stmt.clone(), &ts);
                        Expr::Def(s, t, x, vec![])
                    }
                    StmtDefBody::Builtin(_) => Expr::Def(s, t, *x, ts),
                }
            }
            _ => self._map_expr(e),
        }
    }

    fn map_type(&mut self, t: &Type) -> Type {
        match t {
            Type::Cons(x, ts) => {
                let x = Mangler::mangle_cons_type(*x, ts);
                Type::Cons(x, vec![])
            }
            _ => self._map_type(t),
        }
    }
}

struct Mangler(String);

#[allow(unused)]
impl Mangler {
    fn new() -> Mangler {
        Mangler(String::from("__"))
    }

    fn finish(self) -> Name {
        self.0.into()
    }

    fn mangle_cons_type(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.finish()
    }

    fn mangle_type(t: &Type) -> Name {
        let mut mangle = Mangler::new();
        mangle.mangle_type_rec(t);
        mangle.0.into()
    }

    fn mangle_fun(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write("__F");
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.finish()
    }

    fn mangle_struct(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write("__S");
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.finish()
    }

    fn mangle_enum(x: Name, ts: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write("__E");
        m.write(x);
        ts.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.finish()
    }

    fn mangle_impl_def(x0: Name, ts0: &[Type], x1: Name, ts1: &[Type]) -> Name {
        let mut m = Mangler::new();
        m.write("__I");
        m.write(x0);
        ts0.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.write(x1);
        ts1.into_iter().for_each(|t| m.mangle_type_rec(t));
        m.finish()
    }

    fn write(&mut self, s: impl std::fmt::Display) {
        write!(&mut self.0, "{s}").expect("Mangling should not fail");
    }

    fn mangle_type_rec(&mut self, t: &Type) {
        match t {
            Type::Path(_) => unreachable!(),
            Type::Cons(x, ts) => {
                self.write(x);
                ts.into_iter().for_each(|t| self.mangle_type_rec(t));
            }
            Type::Alias(_, _) => unreachable!(),
            Type::Assoc(_, _, _) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Generic(_) => unreachable!(),
            Type::Fun(ts, t) => {
                self.write("Fun");
                ts.into_iter().for_each(|t| self.mangle_type_rec(t));
                self.mangle_type_rec(t);
                self.write("End");
            }
            Type::Tuple(ts) => {
                self.write("Tuple");
                ts.into_iter().for_each(|t| self.mangle_type_rec(t));
                self.write("End");
            }
            Type::Record(xts) => {
                self.write("Record");
                xts.into_iter().for_each(|(x, t)| {
                    self.write(x);
                    self.mangle_type_rec(t);
                });
                self.write("End");
            }
            Type::Array(t, _i) => {
                self.write("Array");
                self.mangle_type_rec(t);
                self.write("End");
            }
            Type::Never => unreachable!(),
            Type::Paren(_) => unreachable!(),
            Type::Err => unreachable!(),
            Type::Unknown => unreachable!(),
        }
    }
}
