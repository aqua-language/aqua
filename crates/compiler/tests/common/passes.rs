#![allow(unused)]

use std::rc::Rc;

use compiler::ast::Expr;
use compiler::ast::Pat;
use compiler::ast::Program;
use compiler::ast::Stmt;
use compiler::ast::Type;
use compiler::builtins::value::Value;
use compiler::diag::Report;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::pass::Pass as _;
use compiler::source::Cache;
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

struct Tester {
    compiler: Compiler,
}

impl Tester {
    pub fn new() -> Self {
        Self {
            compiler: Compiler::default(),
        }
    }

    pub fn init(&mut self) -> &mut Self {
        self.compiler.init();
        self
    }

    pub fn recover<T>(&mut self, result: T) -> Result<T, Recovered<T>> {
        if self.compiler.report.is_empty() {
            Ok(result)
        } else {
            Err(Recovered::new(
                result,
                trim(
                    &self
                        .compiler
                        .report
                        .string(&mut self.compiler.sources)
                        .unwrap(),
                ),
            ))
        }
    }

    pub fn parse<T>(
        &mut self,
        input: &str,
        f: impl for<'a> FnOnce(&mut Parser<'a, &mut Lexer<'a>>) -> T,
    ) -> Result<T, Recovered<T>> {
        let mut compiler = Compiler::default();
        let input: Rc<str> = Rc::from(input);
        let id = self.compiler.sources.add("test", input.clone());
        let mut lexer = Lexer::new(id, input.as_ref());
        let mut parser = Parser::new(&input, &mut lexer);
        let result = f(&mut parser);
        self.compiler.report.append(&mut parser.report);
        self.compiler.report.append(&mut lexer.report);
        self.recover(result)
    }

    pub fn desugar(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = parse(input)?;
        let result = self.compiler.desugar.run(&program);
        self.recover(result)
    }

    pub fn querycomp(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.desugar(input)?;
        let result = self.compiler.query.run(&program);
        self.recover(result)
    }

    pub fn resolve(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.querycomp(input)?;
        let result = self.compiler.resolve.run(&program);
        self.recover(result)
    }

    pub fn flatten(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.resolve(input)?;
        let result = self.compiler.flatten.run(&program);
        self.recover(result)
    }

    pub fn lift(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.resolve(input)?;
        let result = self.compiler.lift.run(&program);
        self.recover(result)
    }

    pub fn infer(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.resolve(input)?;
        let result = self.compiler.infer.run(&program);
        self.recover(result)
    }

    pub fn monomorphise(&mut self, input: &str) -> Result<Program, Recovered<Program>> {
        let program = self.infer(input)?;
        let result = self.compiler.monomorphise.run(&program);
        self.recover(result)
    }

    pub fn interpret(&mut self, input: &str) -> Result<Value, Recovered<Value>> {
        let mut result = self.monomorphise(input).unwrap();
        let last_stmt = result.stmts.pop().unwrap();
        let last_expr = last_stmt.as_expr().unwrap();
        self.compiler.interpret.interpret(&result);
        let value = self.compiler.interpret.eval_expr(last_expr);
        self.recover(value)
    }
}

pub fn parse(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().parse(input, |p| p.parse(Parser::program).unwrap())
}

pub fn parse_expr(input: &str) -> Result<Expr, Recovered<Expr>> {
    Tester::new().parse(input, |p| p.parse(Parser::expr).unwrap())
}

pub fn parse_stmt(input: &str) -> Result<Stmt, Recovered<Stmt>> {
    Tester::new().parse(input, |p| p.parse(Parser::stmt).unwrap())
}

pub fn parse_type(input: &str) -> Result<Type, Recovered<Type>> {
    Tester::new().parse(input, |p| p.parse(Parser::ty).unwrap())
}

pub fn parse_pat(input: &str) -> Result<Pat, Recovered<Pat>> {
    Tester::new().parse(input, |p| p.parse(Parser::pat).unwrap())
}

pub fn desugar(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().desugar(input)
}

pub fn querycomp(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().querycomp(input)
}

pub fn resolve(input: impl AsRef<str>) -> Result<Program, Recovered<Program>> {
    Tester::new().init().resolve(input.as_ref())
}

pub fn flatten(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().init().flatten(input)
}

pub fn lift(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().init().lift(input)
}

pub fn infer(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().init().infer(input)
}

pub fn monomorphise(input: &str) -> Result<Program, Recovered<Program>> {
    Tester::new().init().monomorphise(input)
}

pub fn interpret(input: impl AsRef<str>) -> Result<Value, Recovered<Value>> {
    Tester::new().init().interpret(input.as_ref())
}
