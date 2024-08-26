impl ariadne::Span for Span {
    fn start(&self) -> usize {
        *self.start() as usize
    }

    fn end(&self) -> usize {
        *self.end() as usize
    }

    type SourceId = u16;

    fn source(&self) -> &Self::SourceId {
        self.file()
    }
}

#[derive(Clone, Copy, Default)]
pub enum Span {
    Source(u16, u32, u32),
    #[default]
    Generated,
}

impl Ord for Span {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Span {}
impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Span::Source(file, start, end) => write!(f, "{file}:{start}..{end}"),
            Span::Generated => write!(f, "..."),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Span::Source(file, start, end) => write!(f, "{:?}:{}-{}", file, start, end),
            Span::Generated => write!(f, "<builtin>"),
        }
    }
}

impl Span {
    pub fn new(file: u16, range: std::ops::Range<u32>) -> Span {
        Span::Source(file, range.start, range.end)
    }

    pub fn file(&self) -> &u16 {
        match self {
            Span::Source(file, _, _) => file,
            Span::Generated => &0,
        }
    }

    pub fn start(&self) -> &u32 {
        match self {
            Span::Source(_, start, _) => start,
            Span::Generated => &0,
        }
    }

    pub fn end(&self) -> &u32 {
        match self {
            Span::Source(_, _, end) => end,
            Span::Generated => &0,
        }
    }
}

impl std::ops::Add<Span> for Span {
    type Output = Span;

    fn add(self, other: Span) -> Self::Output {
        match (self, other) {
            (Span::Generated, Span::Generated) => Span::Generated,
            (Span::Generated, Span::Source(file, start, end)) => Span::new(file, start..end),
            (Span::Source(file, start, end), Span::Generated) => Span::new(file, start..end),
            (Span::Source(file, start, _), Span::Source(_, _, end)) => Span::new(file, start..end),
        }
    }
}
