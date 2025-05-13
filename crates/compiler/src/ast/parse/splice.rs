use crate::report::span::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Splice {
    Text,
    Delim,
    Err,
}

pub struct SpliceIterator<'a> {
    string: &'a str,
    pos: usize,
    span: Span,
}

impl<'a> SpliceIterator<'a> {
    pub fn new(string: &'a str, span: Span) -> Self {
        Self {
            string,
            pos: 0,
            span,
        }
    }
}

impl<'a> Iterator for SpliceIterator<'a> {
    type Item = (Splice, &'a str, Span);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.string.len() {
            return None;
        }

        let file = self.span.file().unwrap();

        if self.string[self.pos..].starts_with("${") {
            let start = self.pos + 1;
            if let Some(i) = self.string[start..].find('}') {
                let end = start + 1 + i;
                self.pos = end;
                let splice = Splice::Delim;
                let text = &self.string[start..end];
                let span = Span::new(file, start as u32..end as u32);
                Some((splice, text, span))
            } else {
                let end = self.string.len();
                self.pos = end;
                let splice = Splice::Err;
                let text = &self.string[start..end];
                let span = Span::new(file, start as u32..end as u32);
                Some((splice, text, span))
            }
        } else if let Some(i) = self.string[self.pos..].find("${") {
            let start = self.pos;
            let end = self.pos + i;
            self.pos = end;
            let splice = Splice::Text;
            let text = &self.string[start..end];
            let span = Span::new(file, start as u32..end as u32);
            Some((splice, text, span))
        } else {
            let start = self.pos;
            let end = self.string.len();
            self.pos = end;
            let splice = Splice::Text;
            let text = &self.string[start..end];
            let span = Span::new(file, start as u32..end as u32);
            Some((splice, text, span))
        }
    }
}
