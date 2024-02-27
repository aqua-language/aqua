#[test]
fn test_source0() {
    let mut sources = compiler::diag::Sources::new();
    assert_eq!(sources.add("file0", "val x = 0;"), 0);
    assert_eq!(sources.add("file1", "def f() = 1;"), 1);
}
