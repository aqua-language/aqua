#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Ast;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::builtins::value::Value;
use compiler::diag::Report;
use compiler::pass;
use compiler::pass::Pass;
use compiler::pass::Pass as _;
use compiler::syntax::lexer::Lexer;
use compiler::syntax::parser::Parser;
use compiler::syntax::source::Cache;
use compiler::syntax::span::Span;
use compiler::Compiler;

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
    (@value; $a:expr, $b:expr) => {{
        let a_str = format!("{:#?}", $a);
        let b_str = format!("{:#?}", $b);
        assert!($a == $b, "{}", common::passes::diff(a_str, b_str));
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

pub struct Recovered<T> {
    pub val: T,
    pub msg: String,
}

impl<T> Recovered<T> {
    pub fn new(value: T, report: String) -> Self {
        Self {
            val: value,
            msg: report,
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Debug for Recovered<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.val)?;
        write!(f, "\n{}", &self.msg)
    }
}

pub fn trim(s: &str) -> String {
    // Trim space right before \n on each line
    s.trim_end()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

struct Tester(Compiler);

impl Tester {
    pub fn new() -> Self {
        Self(Compiler::default())
    }

    pub fn recover<T>(&mut self, result: T) -> Result<T, Recovered<T>> {
        if self.0.report.is_empty() {
            Ok(result)
        } else {
            Err(Recovered::new(
                result,
                trim(&self.0.report.string(&mut self.0.sources).unwrap()),
            ))
        }
    }

    pub fn init(&mut self) -> &mut Self {
        self.0.init();
        self
    }

    pub fn run<T>(
        &mut self,
        s: &str,
        f: impl FnOnce(&mut Compiler, Ast) -> T,
    ) -> Result<T, Recovered<T>> {
        let program = self.0.parse("test", s);
        let result = f(&mut self.0, program);
        self.recover(result)
    }

    pub fn parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>) -> T,
    ) -> Result<T, Recovered<T>> {
        let input: Rc<str> = Rc::from(input);
        let id = self.0.sources.add("test", input.clone());
        let mut lexer = Lexer::new(id, &input);
        let mut parser = Parser::new(&input, &mut lexer);
        let result = f(&mut parser);
        self.0.report.append(&mut parser.report);
        self.0.report.append(&mut lexer.report);
        self.recover(result)
    }
}

pub fn parse(input: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().parse(input, |p| p.parse(Parser::program).unwrap())
}

pub fn parse_expr(s: &str) -> Result<Expr, Recovered<Expr>> {
    Tester::new().parse(s, |p| p.parse(Parser::expr).unwrap())
}

pub fn parse_stmt(s: &str) -> Result<Stmt, Recovered<Stmt>> {
    Tester::new().parse(s, |p| p.parse(Parser::stmt).unwrap())
}

pub fn parse_type(s: &str) -> Result<Type, Recovered<Type>> {
    Tester::new().parse(s, |p| p.parse(Parser::ty).unwrap())
}

pub fn parse_pat(s: &str) -> Result<Pat, Recovered<Pat>> {
    Tester::new().parse(s, |p| p.parse(Parser::pat).unwrap())
}

pub fn desugar(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, |c, p| c.desugar(&p))
}

pub fn querycomp(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, |c, p| c.query_desugar(&p))
}

pub fn resolve(s: impl AsRef<str>) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s.as_ref(), |c, p| c.resolve(&p))
}

pub fn flatten(input: &str) -> Result<Ast, Recovered<Ast>> {
    todo!()
    // Tester::new()
    //     .init()
    //     .run(input, |compiler, program| compiler.expand(&program))
}

pub fn lift(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, |c, p| c.infer(&p))
}

pub fn infer(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, |c, p| c.infer(&p))
}

pub fn monomorphise(s: &str) -> Result<Ast, Recovered<Ast>> {
    Tester::new().init().run(s, |c, p| c.monomorphise(&p))
}

pub fn interpret(s: impl AsRef<str>) -> Result<Value, Recovered<Value>> {
    Tester::new().init().run(s.as_ref(), |c, mut p| {
        let mut p = c.monomorphise(&p);
        let last_stmt = p.stmts.pop().unwrap();
        let last_expr = last_stmt.as_expr().unwrap();
        c.interpreter.interpret(&p);
        c.interpreter.eval_expr(last_expr)
    })
}
