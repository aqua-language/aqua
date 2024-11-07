use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Splice<'a> {
    Text(&'a str, Range<usize>),
    Delim(&'a str, Range<usize>),
    Err(&'a str, Range<usize>),
}

impl<'a> Splice<'a> {
    pub fn range(&self) -> &Range<usize> {
        match self {
            Splice::Text(_, range) => range,
            Splice::Delim(_, range) => range,
            Splice::Err(_, range) => range,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Splice::Text(s, _) => s,
            Splice::Delim(s, _) => s,
            Splice::Err(s, _) => s,
        }
    }
}

pub struct SpliceIterator<'a> {
    string: &'a str,
    pos: usize,
}

impl<'a> SpliceIterator<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string, pos: 0 }
    }
}

impl<'a> Iterator for SpliceIterator<'a> {
    type Item = Splice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.string.len() {
            return None;
        }

        if self.string[self.pos..].starts_with("${") {
            let start = self.pos + 1;
            if let Some(i) = self.string[start..].find('}') {
                let end = start + 1 + i;
                let range = start..end;
                self.pos = end;
                Some(Splice::Delim(&self.string[range.clone()], range))
            } else {
                let end = self.string.len();
                let range = start..end;
                self.pos = end;
                Some(Splice::Err(&self.string[range.clone()], range))
            }
        } else if let Some(i) = self.string[self.pos..].find("${") {
            let start = self.pos;
            let end = self.pos + i;
            let range = start..end;
            self.pos = end;
            Some(Splice::Text(&self.string[range.clone()], range))
        } else {
            let start = self.pos;
            let end = self.string.len();
            let range = start..end;
            self.pos = end;
            Some(Splice::Text(&self.string[range.clone()], range))
        }
    }
}
