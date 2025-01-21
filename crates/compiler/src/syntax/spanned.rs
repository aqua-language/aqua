use crate::syntax::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub s: Span,
    pub v: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, data: T) -> Spanned<T> {
        Spanned { s: span, v: data }
    }
}
