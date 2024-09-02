#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::builtins::value::Value;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::Compiler;
use compiler::Recovered;

#[macro_export]
macro_rules! check {
    ($a:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        assert!(
            $a.msg == msg,
            "{}",
            common::passes::diff($a.msg, msg.to_string())
        );
    }};
    ($a:expr, $b:expr) => {
        assert!($a == $b, "{}", {
            let a_str = format!("{}", $a);
            let b_str = format!("{}", $b);
            if a_str != b_str {
                common::passes::diff(a_str, b_str)
            } else {
                let a_str = format!("{}", $a.verbose());
                let b_str = format!("{}", $b.verbose());
                if a_str != b_str {
                    common::passes::diff(a_str, b_str)
                } else {
                    let a_str = format!("{:#?}", $a);
                    let b_str = format!("{:#?}", $b);
                    common::passes::diff(a_str, b_str)
                }
            }
        });
    };
    ($a:expr, $b:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        check!($a.val, $b);
        assert!(
            $a.msg == msg,
            "{}",
            common::passes::diff($a.msg, msg.to_string())
        );
    }};
    (@value, $a:expr, $b:expr) => {{
        let a_str = format!("{:#?}", $a);
        let b_str = format!("{:#?}", $b);
        assert!($a == $b, "{}", common::passes::diff(a_str, b_str));
    }};
}

#[macro_export]
macro_rules! aqua {
    ($($code:tt)*) => {
        indoc::indoc!($($code)*)
    };
}

trait TestUtils {
    fn parse_expr(&self) -> Result<Value, Recovered<Value>>;
    fn parse_stmt(&self) -> Result<Stmt, Recovered<Stmt>>;
}

impl TestUtils for &'static str {
    fn parse_expr(&self) -> Result<Value, Recovered<Value>> {
        Compiler::default().init().interpret("test", self)
    }

    fn parse_stmt(&self) -> Result<Stmt, Recovered<Stmt>> {
        Compiler::default()
            .init()
            .parse("test", self, |p| p.parse(Parser::stmt).unwrap())
    }
}

pub fn diff(a: String, b: String) -> String {
    let mut output = String::new();
    let diff = similar::TextDiff::from_lines(&a, &b);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "A ",
            similar::ChangeTag::Insert => "B ",
            similar::ChangeTag::Equal => "  ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

pub fn parse_expr(input: &str) -> Result<Expr, Recovered<Expr>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::expr).unwrap())
}

pub fn parse_stmt(input: &str) -> Result<Stmt, Recovered<Stmt>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::stmt).unwrap())
}

pub fn parse_type(input: &str) -> Result<Type, Recovered<Type>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::ty).unwrap())
}

pub fn parse_pat(input: &str) -> Result<Pat, Recovered<Pat>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::pat).unwrap())
}

pub fn parse(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::program).unwrap())
}

pub fn desugar(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().desugar("test", input)
}

pub fn querycomp(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().querycomp("test", input)
}

pub fn resolve(input: impl AsRef<str>) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().resolve("test", input.as_ref())
}

pub fn flatten(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().flatten("test", input)
}

pub fn lift(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().lift("test", input)
}

pub fn infer(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().infer("test", input)
}

pub fn monomorphise(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().monomorphise("test", input)
}

pub fn interpret(input: impl AsRef<str>) -> Result<Value, Recovered<Value>> {
    Compiler::default().init().interpret("test", input.as_ref())
}

pub fn codegen(input: &str) -> String {
    Compiler::default().init().codegen("test", input)
}
