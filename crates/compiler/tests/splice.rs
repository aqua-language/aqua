use compiler::ast::parse::splice::Splice;
use compiler::ast::parse::splice::SpliceIterator;
use compiler::report::source::Cache;
use compiler::report::span::Span;

struct TestSplicer {
    iter: SpliceIterator<'static>,
}

impl TestSplicer {
    fn new(input: &'static str) -> Self {
        let mut cache = Cache::new();
        let file = cache.add("test", input);
        let span = Span::new(file, 0..input.len() as u32);
        let iter = SpliceIterator::new(input, span);
        Self { iter }
    }
}

impl Iterator for TestSplicer {
    type Item = (Splice, &'static str, std::ops::Range<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((splice, text, span)) = self.iter.next() {
            let span = span;
            let start = span.start().unwrap();
            let end = span.end().unwrap();
            Some((splice, text, start..end))
        } else {
            None
        }
    }
}

#[test]
fn test_basic_extraction() {
    let input = "a b c ${x+y} d e f ${z}";
    let mut iter = TestSplicer::new(input);

    assert_eq!(iter.next(), Some((Splice::Text, "a b c ", 0..6)));
    assert_eq!(iter.next(), Some((Splice::Delim, "{x+y}", 7..12)));
    assert_eq!(iter.next(), Some((Splice::Text, " d e f ", 12..19)));
    assert_eq!(iter.next(), Some((Splice::Delim, "{z}", 20..23)));
    assert_eq!(iter.next(), None);

    assert_eq!(&input[0..6], "a b c ");
    assert_eq!(&input[7..12], "{x+y}");
    assert_eq!(&input[12..19], " d e f ");
    assert_eq!(&input[20..23], "{z}");
}

#[test]
fn test_starts_with_delim() {
    let input = "${start} and then some text";
    let mut iter = TestSplicer::new(input);

    assert_eq!(iter.next(), Some((Splice::Delim, "{start}", 1..8)));
    assert_eq!(
        iter.next(),
        Some((Splice::Text, " and then some text", 8..27))
    );
    assert_eq!(iter.next(), None);

    assert_eq!(&input[1..8], "{start}");
    assert_eq!(&input[8..27], " and then some text");
}

#[test]
fn test_ends_with_delim() {
    let input = "Some text before ${end}";
    let mut iter = TestSplicer::new(input);

    assert_eq!(
        iter.next(),
        Some((Splice::Text, "Some text before ", 0..17))
    );
    assert_eq!(iter.next(), Some((Splice::Delim, "{end}", 18..23)));
    assert_eq!(iter.next(), None);

    assert_eq!(&input[0..17], "Some text before ");
    assert_eq!(&input[18..23], "{end}");
}

#[test]
fn test_only_text() {
    let input = "Just some plain text without delimiters";
    let mut iter = TestSplicer::new(input);

    assert_eq!(
        iter.next(),
        Some((
            Splice::Text,
            "Just some plain text without delimiters",
            0..39
        ))
    );
    assert_eq!(iter.next(), None);

    assert_eq!(&input[0..39], "Just some plain text without delimiters");
}

#[test]
fn test_only_delim() {
    let input = "${only_delim}";
    let mut iter = TestSplicer::new(input);

    assert_eq!(iter.next(), Some((Splice::Delim, "{only_delim}", 1..13)));
    assert_eq!(iter.next(), None);

    assert_eq!(&input[1..13], "{only_delim}");
}

#[test]
fn test_unbalanced_opening_delim() {
    let input = "This will error ${unbalanced";
    let mut iter = TestSplicer::new(input);

    // Attempt to iterate should trigger the panic
    assert_eq!(iter.next(), Some((Splice::Text, "This will error ", 0..16)));
    assert_eq!(iter.next(), Some((Splice::Err, "{unbalanced", 17..28)));
    assert_eq!(iter.next(), None);

    assert_eq!(&input[0..16], "This will error ");
    assert_eq!(&input[17..28], "{unbalanced");
}

#[test]
fn test_multiple_delims_in_text() {
    let input = "text ${first} middle ${second} end";
    let mut iter = TestSplicer::new(input);

    assert_eq!(iter.next(), Some((Splice::Text, "text ", 0..5)));
    assert_eq!(iter.next(), Some((Splice::Delim, "{first}", 6..13)));
    assert_eq!(iter.next(), Some((Splice::Text, " middle ", 13..21)));
    assert_eq!(iter.next(), Some((Splice::Delim, "{second}", 22..30)));
    assert_eq!(iter.next(), Some((Splice::Text, " end", 30..34)));
    assert_eq!(iter.next(), None);

    assert_eq!(&input[0..5], "text ");
    assert_eq!(&input[6..13], "{first}");
    assert_eq!(&input[13..21], " middle ");
    assert_eq!(&input[22..30], "{second}");
    assert_eq!(&input[30..34], " end");
}

#[test]
fn test_hello() {
    let input = "my name is ${name}";
    let mut iter = TestSplicer::new(input);

    assert_eq!(iter.next(), Some((Splice::Text, "my name is ", 0..11)));
    assert_eq!(iter.next(), Some((Splice::Delim, "{name}", 12..18)));
    assert_eq!(iter.next(), None);
}
