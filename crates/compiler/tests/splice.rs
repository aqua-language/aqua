use compiler::ast::parse::splice::Splice;
use compiler::ast::parse::splice::SpliceIterator;

#[test]
fn test_basic_extraction() {
    let input = "a b c ${x+y} d e f ${z}";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(iter.next(), Some(Splice::Text("a b c ", 0..6)));
    assert_eq!(iter.next(), Some(Splice::Delim("{x+y}", 7..12)));
    assert_eq!(iter.next(), Some(Splice::Text(" d e f ", 12..19)));
    assert_eq!(iter.next(), Some(Splice::Delim("{z}", 20..23)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_starts_with_delim() {
    let input = "${start} and then some text";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(iter.next(), Some(Splice::Delim("{start}", 1..8)));
    assert_eq!(
        iter.next(),
        Some(Splice::Text(" and then some text", 8..27))
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn test_ends_with_delim() {
    let input = "Some text before ${end}";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(iter.next(), Some(Splice::Text("Some text before ", 0..17)));
    assert_eq!(iter.next(), Some(Splice::Delim("{end}", 18..23)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_only_text() {
    let input = "Just some plain text without delimiters";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(
        iter.next(),
        Some(Splice::Text(
            "Just some plain text without delimiters",
            0..39
        ))
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn test_only_delim() {
    let input = "${only_delim}";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(iter.next(), Some(Splice::Delim("{only_delim}", 1..13)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_unbalanced_opening_delim() {
    let input = "This will error ${unbalanced";
    let mut iter = SpliceIterator::new(input);

    // Attempt to iterate should trigger the panic
    assert_eq!(iter.next(), Some(Splice::Text("This will error ", 0..16)));
    assert_eq!(iter.next(), Some(Splice::Err("{unbalanced", 17..28)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_multiple_delims_in_text() {
    let input = "text ${first} middle ${second} end";
    let mut iter = SpliceIterator::new(input);

    assert_eq!(iter.next(), Some(Splice::Text("text ", 0..5)));
    assert_eq!(iter.next(), Some(Splice::Delim("{first}", 6..13)));
    assert_eq!(iter.next(), Some(Splice::Text(" middle ", 13..21)));
    assert_eq!(iter.next(), Some(Splice::Delim("{second}", 22..30)));
    assert_eq!(iter.next(), Some(Splice::Text(" end", 30..34)));
    assert_eq!(iter.next(), None);
}
