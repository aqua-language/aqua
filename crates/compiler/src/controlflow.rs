/// Goal: Remove all control flow constructs from the AST
use crate::ast::Expr;
use crate::ast::Name;
use crate::ast::StmtVar;
use crate::diag::Report;
use crate::traversal::mapper::Mapper;

#[derive(Debug)]
pub struct Context {
    stack: Stack,
    pub report: Report,
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack: Stack::new(),
            report: Report::new(),
        }
    }

    pub fn scope(&mut self) -> &mut Scope {
        self.stack.0.last_mut().unwrap()
    }

    pub fn push_scope(&mut self, kind: ScopeKind) {
        self.stack.0.push(Scope::new(kind));
    }

    pub fn pop_scope(&mut self) {
        self.stack.0.pop();
    }
}

#[derive(Debug)]
struct Stack(Vec<Scope>);

#[derive(Debug)]
pub struct Scope {
    pub kind: ScopeKind,
    pub names: Vec<Name>,
}

impl Scope {
    fn new(kind: ScopeKind) -> Scope {
        Scope {
            kind,
            names: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ScopeKind {
    Top,
    Loop,
    Block,
}

impl Stack {
    fn new() -> Stack {
        Stack(vec![Scope::new(ScopeKind::Top)])
    }
}

impl Mapper for Context {
    fn map_stmt_var(&mut self, s: &StmtVar) -> StmtVar {
        self.scope().names.push(s.name.clone());
        let expr = self.map_expr(&s.expr);
        StmtVar {
            span: s.span,
            name: s.name.clone(),
            ty: s.ty.clone(),
            expr,
        }
    }

    fn map_expr(&mut self, e: &Expr) -> Expr {
        match e {
            Expr::Assign(_, _, _p, _e) => {
                todo!()
                // let p = self.map_place(p);
                // let e = self.map_expr(e);
                // match e {
                //     Expr::Var(_, _, _) => todo!(),
                //     Expr::Field(_, _, _, _) => todo!(),
                //     Expr::Index(_, _, _, _) => todo!(),
                //     _ => todo!(),
                // }
            }
            Expr::Return(_, _, _) => todo!(),
            Expr::Continue(_, _) => todo!(),
            Expr::Break(_, _) => todo!(),
            Expr::While(_, _, _, _) => todo!(),
            Expr::Fun(_, _, _, _, _) => todo!(),
            Expr::For(_, _, _, _, _) => todo!(),
            Expr::Block(s, t, b) => {
                self.push_scope(ScopeKind::Block);
                let b = self.map_block(b);
                self.pop_scope();
                Expr::Block(s.clone(), t.clone(), b)
            }
            _ => self._map_expr(e),
        }
    }
}
