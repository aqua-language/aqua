use append_only_vec::AppendOnlyVec;
use ariadne::Source;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct Cache;

static CACHE: AppendOnlyVec<(String, Source<Arc<str>>)> = AppendOnlyVec::new();

impl SourceId {
    pub fn new(name: impl ToString, data: impl Into<Arc<str>>) -> SourceId {
        let name = name.to_string();
        let source = Source::from(data.into());
        let id = CACHE.push((name, source));
        SourceId(id as u16)
    }
}

impl ariadne::Cache<SourceId> for Cache {
    type Storage = Arc<str>;

    fn fetch(&mut self, id: &SourceId) -> Result<&Source<Arc<str>>, Box<dyn std::fmt::Debug + '_>> {
        Ok(&CACHE[id.0 as usize].1)
    }

    fn display<'b>(&self, id: &'b SourceId) -> Option<Box<dyn std::fmt::Display + 'b>> {
        Some(Box::new(CACHE[id.0 as usize].0.clone()) as Box<_>)
    }
}
