use crate::syntax::source::SourceId;

impl ariadne::Span for Span {
    fn start(&self) -> usize {
        self.start().unwrap() as usize
    }

    fn end(&self) -> usize {
        self.end().unwrap() as usize
    }

    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        match self {
            Span::Source(id, _, _) => id,
            Span::Generated => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum Span {
    Source(SourceId, u32, u32),
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
    pub fn new(file: SourceId, range: std::ops::Range<u32>) -> Span {
        Span::Source(file, range.start, range.end)
    }

    pub fn file(&self) -> Option<SourceId> {
        match self {
            Span::Source(file, _, _) => Some(*file),
            Span::Generated => None,
        }
    }

    pub fn start(&self) -> Option<u32> {
        match self {
            Span::Source(_, start, _) => Some(*start),
            Span::Generated => None,
        }
    }

    pub fn end(&self) -> Option<u32> {
        match self {
            Span::Source(_, _, end) => Some(*end),
            Span::Generated => None,
        }
    }

    pub fn contains(&self, other: &Span) -> bool {
        match (self, other) {
            (Span::Source(file1, start1, end1), Span::Source(file2, start2, end2)) => {
                file1 == file2 && start1 <= start2 && end1 >= end2
            }
            _ => false,
        }
    }

    pub fn shift(&self, offset: u32) -> Span {
        match self {
            Span::Source(file, start, end) => Span::Source(*file, start + offset, end + offset),
            Span::Generated => Span::Generated,
        }
    }

    pub fn shrink(&self, offset: u32) -> Span {
        match self {
            Span::Source(file, start, end) => Span::Source(*file, start + offset, end - offset),
            Span::Generated => Span::Generated,
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
