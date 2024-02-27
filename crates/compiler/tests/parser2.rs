use compiler::lexer::Lexer;
use compiler::parser2::Parser;

#[ignore]
#[test]
fn test0() {
    let source = "def f(a:i32): i32 = 1+1*1;";
    let lexer = Lexer::new(0, source);
    let _output = Parser::new(source, lexer).parse(Parser::program);
}

#[ignore]
#[test]
fn test1() {
    let source = "def f(a:i32): i32 = 1+1*1;";
    let lexer = Lexer::new(0, source);
    let _output = Parser::new(source, lexer).parse(Parser::program);
}

#[ignore]
#[test]
fn test2() {
    let source = "def f(a:i32): i32 = 1+1*1;";
    let lexer = Lexer::new(0, source);
    let _output = Parser::new(source, lexer).parse(Parser::program);
}
