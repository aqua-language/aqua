use crate::span::Span;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub s: Span,
    pub v: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, data: T) -> Spanned<T> {
        Spanned { s: span, v: data }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            s: self.s,
            v: f(self.v),
        }
    }
}

impl<T> Spanned<Option<Vec<T>>> {
    pub fn flatten(self) -> Spanned<Vec<T>> {
        self.map(|v| v.unwrap_or_default())
    }
}

impl<T> Spanned<Option<T>> {
    pub fn transpose(self) -> Option<Spanned<T>> {
        match self.v {
            Some(value) => Some(Spanned {
                s: self.s,
                v: value,
            }),
            None => None,
        }
    }
}

