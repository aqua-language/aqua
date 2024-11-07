#[test]
fn test_source0() {
    let mut sources = compiler::source::Cache::new();
    assert_eq!(sources.add("file0", "val x = 0;").inner(), 0);
    assert_eq!(sources.add("file1", "def f() = 1;").inner(), 1);
}
