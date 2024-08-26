use std::rc::Rc;
use ariadne::Source;

#[derive(Default, Clone)]
pub struct Sources(Vec<(String, Source<Rc<str>>)>);

impl Sources {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add(&mut self, name: impl ToString, data: impl Into<Rc<str>>) -> u16 {
        let id = self.0.len() as u16;
        self.0.push((name.to_string(), Source::from(data.into())));
        id
    }
}

impl std::ops::Index<u16> for Sources {
    type Output = Source<Rc<str>>;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize].1
    }
}

impl std::fmt::Debug for Sources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self.0.iter().map(|(x, _)| x).collect::<Vec<_>>();
        f.debug_struct("Sources").field("sources", &names).finish()
    }
}

impl ariadne::Cache<u16> for Sources {
    type Storage = Rc<str>;

    fn fetch(&mut self, id: &u16) -> Result<&Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
        self.0
            .get(*id as usize)
            .map(|(_, s)| s)
            .ok_or_else(|| Box::new(format!("Source with id {} not found in cache", id)) as Box<_>)
    }

    fn display<'b>(&self, id: &'b u16) -> Option<Box<dyn std::fmt::Display + 'b>> {
        self.0
            .get(*id as usize)
            .map(|x| Box::new(x.0.clone()) as Box<_>)
    }
}
