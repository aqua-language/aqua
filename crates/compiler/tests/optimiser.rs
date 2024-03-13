use compiler::opt::Optimiser;
use egglog::ast::parse::ExprParser;

#[test]
fn test_rewrites() {
    let mut opt = Optimiser::new();
    opt.include("optimiser.egg").unwrap();
}

#[test]
fn test_parser() {
    let mut opt = Optimiser::new();
    let a = ExprParser::new()
        .parse(r#"(Op2 Add (Val (I64 6)) (Op2 Mul (Val (I64 2)) (Val (I64 3))))"#)
        .unwrap();
    let b = ExprParser::new().parse("(Val (I64 12))").unwrap();
    assert!(opt.check(a, b));
}
