use super::Expr;

impl Expr {
    pub fn is_braced(&self) -> bool {
        matches!(
            self,
            Expr::Block(..) | Expr::Match(..) | Expr::While(..) | Expr::For(..)
        )
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Expr::Tuple(_, _, v) if v.is_empty())
    }
}
