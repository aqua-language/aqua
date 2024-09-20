use compiler::source::SourceId;

#[test]
fn test_source0() {
    assert_eq!(SourceId::new("file0", "val x = 0;").inner(), 0);
    assert_eq!(SourceId::new("file1", "def f() = 1;").inner(), 1);
}
