use std::rc::Rc;

use ariadne::Cache as _;
use ariadne::Source;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(u16);

impl SourceId {
    pub fn inner(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct Cache(Vec<(String, Source<Rc<str>>)>);

impl Cache {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get_pos(&mut self, id: SourceId, i: u32) -> (u32, u32) {
        let source = self.fetch(&id).unwrap();
        let (_, l, c) = source.get_byte_line(i as usize).unwrap();
        (l as u32, c as u32)
    }

    // TODO: Does not work yet for unicode
    pub fn get_byte(&mut self, file: SourceId, (l,c): (u32, u32)) -> u32 {
        let source = self.fetch(&file).unwrap();
        let line = source.line(l as usize).unwrap();
        (line.offset() as u32) + c
    }

    /// Add a new source to the cache.
    pub fn add(&mut self, name: impl ToString, data: impl Into<Rc<str>>) -> SourceId {
        let name = name.to_string();
        let source = Source::from(data.into());
        self.0.push((name, source));
        let id = self.0.len() as u16 - 1;
        SourceId(id as u16)
    }
}

impl<'a> ariadne::Cache<SourceId> for Cache {
    type Storage = Rc<str>;

    fn fetch(&mut self, id: &SourceId) -> Result<&Source<Rc<str>>, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self.0[id.0 as usize].1)
    }

    fn display<'b>(&self, id: &'b SourceId) -> Option<Box<dyn std::fmt::Display + 'b>> {
        Some(Box::new(self.0[id.0 as usize].0.clone()) as Box<_>)
    }
}
