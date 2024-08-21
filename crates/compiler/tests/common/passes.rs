#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::builtins::Value;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::Compiler;
use compiler::Recovered;


#[macro_export]
macro_rules! check {
    ($a:expr, $msg:literal) => {{
        let msg = indoc::indoc!($msg);
        assert!($a.msg == msg, "{}", common::passes::diff($a.msg, msg.to_string()));
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
        assert!($a.msg == msg, "{}", common::passes::diff($a.msg, msg.to_string()));
    }};
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

pub fn parse<T>(
    comp: &mut Compiler,
    name: impl ToString,
    input: &str,
    f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>) -> T,
) -> Result<T, Recovered<T>> {
    let input: Rc<str> = Rc::from(input);
    let id = comp.sources.add(name, input.clone());
    let mut lexer = Lexer::new(id, input.as_ref());
    let mut parser = Parser::new(&input, &mut lexer);
    let result = f(&mut parser);
    comp.add_report(&mut parser.report);
    comp.add_report(&mut lexer.report);
    comp.recover(result)
}

pub fn desugar(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.parse(name, input, |parser| parser.parse(Parser::program).unwrap())?;
    let result = comp.desugar.desugar(&program);
    comp.recover(result)
}

pub fn resolve(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.desugar(name, input)?;
    let result = comp.resolve.resolve(&program);
    comp.report.merge(&mut comp.resolve.report);
    comp.recover(result)
}

pub fn flatten(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let program = comp.resolve(name, input)?;
    let program = comp.flatten.flatten(&program);
    comp.recover(program)
}

pub fn lift(comp: &mut Compiler, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
    let program = comp.resolve(name, input)?;
    let program = comp.lift.lift(&program);
    comp.recover(program)
}

pub fn infer(comp: &mut Compiler, name: &str, input: &str) -> Result<Program, Recovered<Program>> {
    let result = comp.resolve(name, input)?;
    let result = comp.infer.infer(&result);
    comp.report.merge(&mut comp.infer.report);
    comp.recover(result)
}

pub fn monomorphise(
    comp: &mut Compiler,
    name: &str,
    input: &str,
) -> Result<Program, Recovered<Program>> {
    let result = comp.infer(name, input)?;
    let result = comp.monomorphise.monomorphise(&result);
    comp.recover(result)
}

pub fn interpret(comp: &mut Compiler, name: &str, input: &str) -> Result<Value, Recovered<Value>> {
    let mut result = comp.infer(name, input).unwrap();
    let stmt = result.stmts.pop().unwrap();
    let expr = stmt.as_expr().unwrap();
    comp.interpret.interpret(&result);
    let value = comp.interpret.expr(expr);
    comp.report.merge(&mut comp.interpret.report);
    comp.recover(value)
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

pub fn parse_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().parse("test", input, |p| p.parse(Parser::program).unwrap())
}

pub fn desugar_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().desugar("test", input)
}

pub fn querycomp_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().querycomp("test", input)
}

pub fn resolve_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().resolve("test", input)
}

pub fn flatten_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().flatten("test", input)
}

pub fn lift_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().lift("test", input)
}

pub fn infer_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().infer("test", input)
}

pub fn monomorphise_program(input: &str) -> Result<Program, Recovered<Program>> {
    Compiler::default().init().monomorphise("test", input)
}

pub fn interpret_program(input: &str) -> Result<Value, Recovered<Value>> {
    Compiler::default().init().interpret("test", input)
}

macro_rules! parse_program {
    ($code:literal) => {
        common::passes::parse_program(indoc::indoc!($code))
    };
}

macro_rules! parse_expr {
    ($code:literal) => {
        common::passes::parse_expr(indoc::indoc!($code))
    };
    ($code:expr) => {
        common::passes::parse_stmt($code)
    };
}

macro_rules! parse_type {
    ($code:literal) => {
        common::passes::parse_type(indoc::indoc!($code))
    };
}

macro_rules! parse_stmt {
    ($code:literal) => {
        common::passes::parse_stmt(indoc::indoc!($code))
    };
}

macro_rules! parse_pat {
    ($code:literal) => {
        common::passes::parse_pat(indoc::indoc!($code))
    };
}

macro_rules! desugar_program {
    ($code:literal) => {
        common::passes::desugar_program(indoc::indoc!($code))
    };
}

macro_rules! resolve_program {
    ($code:literal) => {
        common::passes::resolve_program(indoc::indoc!($code))
    };
}

macro_rules! lift_program {
    ($code:literal) => {
        common::passes::lift_program(indoc::indoc!($code))
    };
}

macro_rules! infer_program {
    ($code:literal) => {
        common::passes::infer_program(indoc::indoc!($code))
    };
}

macro_rules! monomorphise_program {
    ($code:literal) => {
        common::passes::monomorphise_program(indoc::indoc!($code))
    };
}

macro_rules! interpret_program {
    ($code:literal) => {
        common::passes::interpret_program(indoc::indoc!($code))
    };
}

macro_rules! flatten_program {
    ($code:literal) => {
        common::passes::flatten_program(indoc::indoc!($code))
    };
}

macro_rules! querycomp_program {
    ($code:literal) => {
        common::passes::querycomp_program(indoc::indoc!($code))
    };
}
